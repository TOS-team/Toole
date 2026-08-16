use crate::file_certif::{certificat, PinnedServerVerifier};
use crate::{ToolError, UI};
use quinn::{ClientConfig, Connection, Endpoint, RecvStream, SendStream, ServerConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Error as IoError;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub transfer_id: String,
    pub rel_path: String,
    pub size: u64,
    pub is_dir: bool,
}

/// en-tête de lot envoyé par l'émetteur sur le premier flux de la connexion :
/// le récepteur connaît le transfer_id, le total du lot, l'émetteur et la
/// liste des fichiers à l'avance, afin d'afficher la demande d'acceptation et
/// la même progression globale que l'émetteur dès le premier fichier. sender
/// et files sont tolérants (serde default) pour rester compatibles avec un
/// ancien émetteur qui n'envoie pas ces champs.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BatchHeader {
    pub transfer_id: String,
    pub total_bytes: u64,
    #[serde(default)]
    pub sender: String,
    #[serde(default)]
    pub files: Vec<String>,
}

const CHUNK_SIZE: usize = 1_048_576; // 1 Mo
pub const ACK: u8 = 0x01;
const COMPLETE: u8 = 0x02;
/// réponse du récepteur pour refuser le transfert à la demande d'acceptation
pub const REFUSE: u8 = 0x03;
/// code de fermeture QUIC d'une fin normale de transfert
pub const CLOSE_OK: u32 = 0;
/// code de fermeture QUIC d'une annulation : le pair sait distinguer
/// « annulé » d'une simple erreur réseau
pub const CLOSE_CANCEL: u32 = 1;
/// délai d'attente de la décision du destinataire (émetteur et récepteur)
pub const DECISION_TIMEOUT: Duration = Duration::from_secs(30);
const PORT: u16 = 58200;

/// registre des demandes d'acceptation en attente : le récepteur y dépose un
/// canal oneshot sous le transfer_id, et la commande respond_transfer le
/// résout quand l'utilisateur clique sur accepter/refuser. auto_accept permet
/// aux tests de sauter la validation (comportement historique).
pub struct DecisionBoard {
    pending: Mutex<HashMap<String, oneshot::Sender<bool>>>,
    auto_accept: AtomicBool,
}

impl DecisionBoard {
    pub fn new() -> Self {
        DecisionBoard {
            pending: Mutex::new(HashMap::new()),
            auto_accept: AtomicBool::new(false),
        }
    }

    pub fn set_auto_accept(&self, v: bool) {
        self.auto_accept.store(v, Ordering::Relaxed);
    }

    pub fn auto_accept(&self) -> bool {
        self.auto_accept.load(Ordering::Relaxed)
    }

    /// je dépose un canal d'attente pour ce transfert et je rends la partie
    /// réceptrice ; l'app résout la décision via `resolve`.
    pub fn register(&self, transfer_id: &str) -> oneshot::Receiver<bool> {
        let (tx, rx) = oneshot::channel();
        self.pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(transfer_id.to_string(), tx);
        rx
    }

    /// je résous la décision de l'utilisateur ; false si le transfert est
    /// inconnu ou déjà traité
    pub fn resolve(&self, transfer_id: &str, accepted: bool) -> bool {
        let tx = self
            .pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(transfer_id);
        match tx {
            Some(tx) => {
                let _ = tx.send(accepted);
                true
            }
            None => false,
        }
    }

    /// je retire une demande en attente sans la résoudre (timeout, annulation
    /// ou arrêt de l'app) : le canal oneshot resterait sinon orphelin dans la
    /// map et un respond_transfer tardif résoudrait un canal mort
    pub fn remove(&self, transfer_id: &str) {
        self.pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(transfer_id);
    }

    /// je vérifie si un transfert attend encore une décision (utile en test)
    pub fn has_pending(&self, transfer_id: &str) -> bool {
        self.pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains_key(transfer_id)
    }
}

impl Default for DecisionBoard {
    fn default() -> Self {
        Self::new()
    }
}

/// je détecte si une erreur de lecture quinn correspond à une annulation
/// distante : soit un flux reset par l'émetteur (arrêt utilisateur), soit une
/// connexion fermée avec le code CLOSE_CANCEL
pub fn is_remote_cancel(e: &quinn::ReadExactError) -> bool {
    match e {
        quinn::ReadExactError::ReadError(quinn::ReadError::Reset(_)) => true,
        quinn::ReadExactError::ReadError(quinn::ReadError::ConnectionLost(
            quinn::ConnectionError::ApplicationClosed(a),
        )) => a.error_code.into_inner() == CLOSE_CANCEL as u64,
        _ => false,
    }
}

/// convertit une erreur de lecture quinn en ToolError, en distinguant
/// l'annulation distante (RemoteCancel) des autres erreurs réseau
pub fn quinn_to_err(e: quinn::ReadExactError) -> ToolError {
    if is_remote_cancel(&e) {
        ToolError::RemoteCancel
    } else {
        e.into()
    }
}

/// convertit une erreur d'écriture quinn en ToolError, en distinguant
/// l'annulation distante (connexion fermée avec CLOSE_CANCEL) des erreurs
/// réseau : sans ça, un write_all qui échoue quand le récepteur annule serait
/// signalé comme une erreur au lieu d'une annulation
pub fn write_quinn_to_err(e: quinn::WriteError) -> ToolError {
    match e {
        quinn::WriteError::ConnectionLost(quinn::ConnectionError::ApplicationClosed(a))
            if a.error_code.into_inner() == CLOSE_CANCEL as u64 =>
        {
            ToolError::RemoteCancel
        }
        e => e.into(),
    }
}

/// limite les emissions de progression UI a ~20/s : le webview n'a pas besoin
/// de 190 evenements/s et chaque IPC coûte cher en boucle serree
struct UiThrottle {
    last: Instant,
}

impl UiThrottle {
    fn new() -> Self {
        UiThrottle {
            last: Instant::now(),
        }
    }

    /// true si au moins 50ms se sont ecoulees depuis la derniere emission
    fn ready(&mut self) -> bool {
        if self.last.elapsed() >= Duration::from_millis(50) {
            self.last = Instant::now();
            true
        } else {
            false
        }
    }
}

pub fn io_err<E: std::fmt::Display>(e: E) -> ToolError {
    IoError::other(e.to_string()).into()
}

pub async fn make_server_endpoint() -> Result<Endpoint, ToolError> {
    let (cert_pem, key_pem) = certificat().await?;
    let mut cert_reader = std::io::BufReader::new(cert_pem.as_bytes());
    let mut key_reader = std::io::BufReader::new(key_pem.as_bytes());

    let certs = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;

    let key = match rustls_pemfile::private_key(&mut key_reader)? {
        Some(v) => v,
        None => return Err(ToolError::ParseKeyError),
    };

    let server_config = ServerConfig::with_single_cert(certs, key)?;
    let mut server_config = server_config;
    server_config.transport_config(Arc::new(transport_config()));
    let bind_addr: SocketAddr = format!("0.0.0.0:{PORT}").parse().map_err(io_err)?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok(endpoint)
}

/// fenetres QUIC larges (defaut quinn = 1,25 Mo par stream, ca plafonne le debit)
/// initial_rtt garde au defaut quinn (333ms) : une valeur optimiste fait partir le
/// pacing trop vite et provoque des pertes puis l'effondrement de Cubic sur WiFi
/// j'ajoute aussi un idle timeout court + keepalive : une déconnexion soudaine
/// (app fermée, réseau coupé) est détectée en ~15 s au lieu des 30 s par défaut,
/// et le keepalive maintient la connexion vivante pendant la période silencieuse
/// de la demande d'acceptation (jusqu'à 30 s de réflexion utilisateur)
fn transport_config() -> quinn::TransportConfig {
    use quinn::VarInt;
    let mut t = quinn::TransportConfig::default();
    t.stream_receive_window(VarInt::from(8u32 * 1024 * 1024));
    t.receive_window(VarInt::from(32u32 * 1024 * 1024));
    t.send_window(32 * 1024 * 1024);
    t.max_idle_timeout(Some(
        quinn::IdleTimeout::try_from(Duration::from_secs(15)).unwrap(),
    ));
    t.keep_alive_interval(Some(Duration::from_secs(3)));
    t
}

pub fn make_client_endpoint(expected_fingerprint: Option<&str>) -> Result<Endpoint, ToolError> {
    // je vérifie l'identité du serveur par son empreinte épinglée (TOFU) :
    // None au premier contact (on épingle après le handshake), l'empreinte
    // attendue ensuite. Je ne vérifie plus « rien » : un cert différent de
    // celui épinglé fait échouer le handshake
    let verifier = Arc::new(PinnedServerVerifier {
        expected: expected_fingerprint.map(str::to_string),
    });
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_no_client_auth();

    let client_config = ClientConfig::new(Arc::new(
        quinn::crypto::rustls::QuicClientConfig::try_from(crypto).map_err(io_err)?,
    ));
    let mut client_config = client_config;
    client_config.transport_config(Arc::new(transport_config()));

    let bind_addr: SocketAddr = "0.0.0.0:0".parse().map_err(io_err)?;
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_config);
    Ok(endpoint)
}

/// reçoit un lot de fichiers sur cette connexion et remonte la liste des fichiers recus
#[allow(clippy::too_many_arguments)]
pub async fn handle_incoming_connection(
    connection: Connection,
    dest_dir: PathBuf,
    stop: Arc<AtomicBool>,
    files: Arc<Mutex<Vec<String>>>,
    bytes: Arc<AtomicU64>,
    ui: Arc<dyn UI>,
    transfer_id: Arc<Mutex<Option<String>>>,
    total: Arc<AtomicU64>,
    decisions: Arc<DecisionBoard>,
    registry: Arc<dyn crate::TransferRegistry>,
) -> Result<(), ToolError> {
    // je mémorise si un flux a échoué en cours de route (ex: annulation côté
    // émetteur) : dans ce cas je signalerai une erreur, pas une réception
    let had_error = Arc::new(AtomicBool::new(false));
    // je mémorise si le pair a annulé explicitement (reset ou code CLOSE_CANCEL)
    // pour signaler une annulation, pas une erreur
    let cancelled_by_peer = Arc::new(AtomicBool::new(false));
    let mut handles: Vec<JoinHandle<()>> = Vec::new();
    let mut header_received = false;
    loop {
        if stop.load(Ordering::Relaxed) {
            connection.close(CLOSE_CANCEL.into(), b"arret utilisateur");
            return Err(ToolError::Cancelled);
        }

        match tokio::time::timeout(Duration::from_secs(1), connection.accept_bi()).await {
            Ok(Ok((mut send, mut recv))) => {
                // le premier flux est l'en-tête de lot : il apporte transfer_id,
                // le total et la liste des fichiers, et demande la validation de
                // l'utilisateur (accepter / refuser) avant de recevoir
                if !header_received {
                    header_received = true;
                    let header: BatchHeader = read_json_line(&mut recv).await?;
                    {
                        let mut tid = transfer_id.lock().unwrap_or_else(|e| e.into_inner());
                        if tid.is_none() {
                            *tid = Some(header.transfer_id.clone());
                        }
                    }
                    total.store(header.total_bytes, Ordering::Relaxed);
                    // je m'enregistre pour que la croix de la carte puisse
                    // annuler cette réception depuis l'interface
                    registry.register(&header.transfer_id, stop.clone());

                    // je demande la décision à l'utilisateur et j'attends sa réponse
                    ui.transfert_incoming(
                        &header.transfer_id,
                        &header.sender,
                        header.total_bytes,
                        header.files.clone(),
                    );
                    let decision = await_decision(&decisions, &header.transfer_id, &stop, &connection).await;

                    match decision {
                        Decision::Accepted => {
                            send.write_all(&[ACK]).await.map_err(write_quinn_to_err)?;
                            send.finish()?;
                            ui.show_progress_bar(&header.transfer_id);
                        }
                        Decision::Refused | Decision::TimedOut => {
                            // je laisse le temps au REFUSE d'être lu avant toute
                            // fermeture : une fermeture immédiate envoie un
                            // CONNECTION_CLOSE qui peut éclipser la donnée de
                            // flux encore en transit, et l'émetteur notifierait
                            // « annulé » au lieu de « refusé ». C'est l'émetteur
                            // qui ferme, une fois le REFUSE lu.
                            if send.write_all(&[REFUSE]).await.is_err() {
                                // l'émetteur a déjà fermé (annulation pendant
                                // l'attente) : le refus n'a plus de sens
                                return Err(ToolError::RemoteCancel);
                            }
                            send.finish()?;
                            return Err(ToolError::Refused);
                        }
                        Decision::Cancelled => {
                            connection.close(CLOSE_CANCEL.into(), b"annulation utilisateur");
                            return Err(ToolError::RemoteCancel);
                        }
                        Decision::RemoteCancelled => {
                            return Err(ToolError::RemoteCancel);
                        }
                        Decision::RemoteError => {
                            return Err(io_err("l'emetteur a ferme pendant la decision"));
                        }
                    }
                    continue;
                }

                let dest_dir = dest_dir.clone();
                let files = files.clone();
                let bytes = bytes.clone();
                let ui = ui.clone();
                let transfer_id = transfer_id.clone();
                let total = total.clone();
                let had_error = had_error.clone();
                let cancelled_by_peer = cancelled_by_peer.clone();
                handles.push(tokio::spawn(async move {
                    if let Err(e) = receive_one(
                        send,
                        recv,
                        &dest_dir,
                        files,
                        bytes,
                        ui,
                        transfer_id,
                        total,
                        cancelled_by_peer.clone(),
                    )
                    .await
                    {
                        match e {
                            ToolError::RemoteCancel => {
                                // l'émetteur a annulé explicitement : je le
                                // signale en annulation, pas en erreur
                                cancelled_by_peer.store(true, Ordering::Relaxed);
                            }
                            e => {
                                // je marque l'échec pour que la connexion soit
                                // signalée en erreur (pas comme un transfert reçu)
                                had_error.store(true, Ordering::Relaxed);
                                eprintln!("Erreur reception fichier: {e}");
                            }
                        }
                    }
                }));
            }
            Ok(Err(quinn::ConnectionError::ApplicationClosed(a)))
                if a.error_code.into_inner() == CLOSE_CANCEL as u64 =>
            {
                // le pair a fermé avec le code d'annulation (refus ou arrêt)
                for h in handles {
                    let _ = h.await;
                }
                return Err(ToolError::RemoteCancel);
            }
            Ok(Err(quinn::ConnectionError::ApplicationClosed(_)))
            | Ok(Err(quinn::ConnectionError::LocallyClosed)) => {
                // j'attends la fin des tâches de flux : elles posent
                // `had_error` / `cancelled_by_peer` si un fichier a échoué ou a
                // été annulé, et elles terminent vite une fois la connexion fermée
                for h in handles {
                    let _ = h.await;
                }
                if cancelled_by_peer.load(Ordering::Relaxed) {
                    return Err(ToolError::RemoteCancel);
                }
                // si un flux a échoué (ex: annulation côté émetteur), je
                // signale une erreur plutôt qu'une réception
                if had_error.load(Ordering::Relaxed) {
                    return Err(io_err("connexion fermee avec un flux en echec"));
                }
                // contrôle de complétude : si l'émetteur a disparu en cours de
                // route, je refuse de notifier une réception alors que des
                // octets manquent
                let done = bytes.load(Ordering::Relaxed);
                let expected = total.load(Ordering::Relaxed);
                if done < expected {
                    return Err(io_err("connexion interrompue : reception incomplete"));
                }
                return Ok(());
            }
            Ok(Err(e)) => {
                // perte de connexion (ex: pair qui disparaît brutalement) :
                // j'attends la fin des flux pour laisser le nettoyage des
                // fichiers partiels se terminer, puis je signale l'erreur
                for h in handles {
                    let _ = h.await;
                }
                return Err(e.into());
            }
            Err(_) => continue, // timeout : on revérifie `stop`
        }
    }
}

/// motif de sortie de l'attente de décision : je distingue le refus explicite
/// de l'utilisateur du timeout et de l'annulation pour des notifications UI
/// cohérentes entre émetteur et récepteur
enum Decision {
    Accepted,
    Refused,
    TimedOut,
    Cancelled,
    /// l'émetteur a fermé la connexion pendant l'attente (annulation ou perte)
    RemoteCancelled,
    RemoteError,
}

/// attend la décision de l'utilisateur (ou auto-accepte en test). Je rends le
/// motif exact pour que l'app distingue refus, timeout et annulation, et je
/// surveille aussi la connexion : si l'émetteur annule pendant l'attente, la
/// carte du récepteur ne doit pas rester bloquée 30 s en « en attente ».
async fn await_decision(
    decisions: &DecisionBoard,
    transfer_id: &str,
    stop: &Arc<AtomicBool>,
    connection: &Connection,
) -> Decision {
    if decisions.auto_accept() {
        return Decision::Accepted;
    }
let mut rx = decisions.register(transfer_id);
        let deadline = Instant::now() + DECISION_TIMEOUT;
        let mut closed = Box::pin(connection.closed());
        loop {
            if stop.load(Ordering::Relaxed) {
                decisions.remove(transfer_id);
                return Decision::Cancelled;
            }
            if Instant::now() >= deadline {
                // le délai est écoulé : je retire la demande en attente pour ne
                // pas laisser un canal orphelin résolvable par un clic tardif
                decisions.remove(transfer_id);
                return Decision::TimedOut;
            }
            tokio::select! {
                r = &mut rx => {
                    match r {
                        Ok(true) => return Decision::Accepted,
                        Ok(false) => return Decision::Refused,
                        // canal fermé (app qui s'éteint) : la demande ne peut
                        // plus être résolue, je la retire du registre
                        Err(_) => {
                            decisions.remove(transfer_id);
                            return Decision::Cancelled;
                        }
                    }
                }
                err = &mut closed => {
                    // l'émetteur a fermé pendant l'attente : je retire la
                    // demande et je rends la main sans attendre le timeout
                    decisions.remove(transfer_id);
                    let is_cancel = matches!(
                        &err,
                        quinn::ConnectionError::ApplicationClosed(a)
                            if a.error_code.into_inner() == CLOSE_CANCEL as u64
                    );
                    if is_cancel {
                        return Decision::RemoteCancelled;
                    }
                    return Decision::RemoteError;
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => continue,
            }
        }
    }

#[allow(clippy::too_many_arguments)]
async fn receive_one(
    mut send: SendStream,
    mut recv: RecvStream,
    dest_dir: &Path,
    files: Arc<Mutex<Vec<String>>>,
    bytes: Arc<AtomicU64>,
    ui: Arc<dyn UI>,
    transfer_id: Arc<Mutex<Option<String>>>,
    total: Arc<AtomicU64>,
    cancelled_by_peer: Arc<AtomicBool>,
) -> Result<(), ToolError> {
    let metadata = read_json_line::<Metadata>(&mut recv).await?;
    let full_path = dest_dir.join(&metadata.rel_path);
    let name = Path::new(&metadata.rel_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| metadata.rel_path.clone());

    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // l'id du transfert vient de l'en-tête de lot ; en secours (protocole
    // ancien) je le prends du metadata, sans redéclencher la barre de
    // progression qui a été signalée par l'en-tête
    {
        let mut tid = transfer_id.lock().unwrap_or_else(|e| e.into_inner());
        if tid.is_none() {
            *tid = Some(metadata.transfer_id.clone());
        }
    }
    let tid = transfer_id
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_default();

    if metadata.is_dir {
        fs::create_dir_all(&full_path).await?;
        send.write_all(&[ACK]).await.map_err(write_quinn_to_err)?;
        send.finish()?;
        files.lock().unwrap().push(name);
        return Ok(());
    }

    // Ack métadonnées
    send.write_all(&[ACK]).await.map_err(write_quinn_to_err)?;

    let mut out_file = fs::File::create(&full_path).await?;
    let mut received: u64 = 0;
    let mut throttle = UiThrottle::new();

    let res: Result<(), ToolError> = async {
        while received < metadata.size {
            let mut len_buf = [0u8; 4];
            recv.read_exact(&mut len_buf).await.map_err(quinn_to_err)?;
            let len = u32::from_be_bytes(len_buf) as usize;

            let mut data = vec![0u8; len];
            recv.read_exact(&mut data).await.map_err(quinn_to_err)?;

            out_file.write_all(&data).await?;
            received += len as u64;

            // progression cote receveur, aluminee sur celle de l'emetteur
            let done = bytes.fetch_add(len as u64, Ordering::Relaxed) + len as u64;
            if throttle.ready() || received == metadata.size {
                ui.update_progress_bar(&tid, done, total.load(Ordering::Relaxed));
                ui.file_progress_bar(&tid, &name, received, metadata.size);
            }
        }

        out_file.flush().await?;

        // Marqueur de complétion, puis ack final
        let mut complete = [0u8; 1];
        recv.read_exact(&mut complete).await.map_err(quinn_to_err)?;
        if complete[0] != COMPLETE {
            return Err(io_err(
                "fin de fichier inattendue (protocole desynchronise)",
            ));
        }
        send.write_all(&[ACK]).await.map_err(write_quinn_to_err)?;

        send.finish()?;
        Ok(())
    }
    .await;

    match &res {
        Ok(()) => {
            files.lock().unwrap().push(name);
        }
        Err(ToolError::RemoteCancel) => {
            // l'émetteur a annulé : pas de nettoyage nécessaire côté récepteur
            // (je ne laisse pas de réception tronquée : je signale l'annulation)
            cancelled_by_peer.store(true, Ordering::Relaxed);
        }
        Err(_) => {
            // je supprime le fichier partiel : jamais de réception tronquée
            // dans le dossier de destination
            let _ = fs::remove_file(&full_path).await;
        }
    }
    res
}

pub async fn read_json_line<T: for<'de> Deserialize<'de>>(
    recv: &mut RecvStream,
) -> Result<T, ToolError> {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        recv.read_exact(&mut byte).await.map_err(quinn_to_err)?;
        if byte[0] == b'\n' {
            break;
        }
        buf.push(byte[0]);
    }

    serde_json::from_slice(&buf).map_err(io_err)
}

pub fn collect_entries<'a>(
    root: &'a Path,
    current: &'a Path,
    out: &'a mut Vec<(PathBuf, String, bool)>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ToolError>> + Send + 'a>> {
    Box::pin(async move {
        let metadata = fs::metadata(current).await?;
        let base = root.parent().unwrap_or(root);

        if metadata.is_file() {
            let rel = current
                .strip_prefix(base)
                .unwrap_or(current)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((current.to_path_buf(), rel, false));
            return Ok(());
        }

        if metadata.is_dir() {
            let mut read_dir = fs::read_dir(current).await?;
            let mut has_children = false;

            while let Some(entry) = read_dir.next_entry().await? {
                has_children = true;
                collect_entries(root, &entry.path(), out).await?;
            }

            if !has_children {
                let rel = current
                    .strip_prefix(base)
                    .unwrap_or(current)
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((current.to_path_buf(), rel, true));
            }
        }

        Ok(())
    })
}

/// envoie un fichier ou dossier dans la connexion QUIC établie : les paquets
/// sont fiables grâce à QUIC (pas de retry applicatif), je copie le contenu en
/// blocs et je signale la progression via l'interface UI.
#[allow(clippy::too_many_arguments)]
pub async fn send_entry(
    connection: Connection,
    abs_path: PathBuf,
    rel_path: String,
    is_dir: bool,
    stop: Arc<AtomicBool>,
    ui: Arc<dyn UI>,
    transfer_id: String,
    total_bytes: u64,
    bytes_sent_counter: Arc<AtomicU64>,
) -> Result<(), ToolError> {
    let (mut send, mut recv) = connection.open_bi().await?;

    if is_dir {
        let metadata = Metadata {
            transfer_id: transfer_id.clone(),
            rel_path,
            size: 0,
            is_dir: true,
        };
        write_json_line(&mut send, &metadata).await?;
        let mut ack = [0u8; 1];
        recv.read_exact(&mut ack).await.map_err(quinn_to_err)?;
        if ack[0] != ACK {
            return Err(io_err("dossier rejete par le receveur"));
        }
        send.finish()?;
        return Ok(());
    }

    let file_name = Path::new(&rel_path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| rel_path.clone());

    let size = fs::metadata(&abs_path).await?.len();
    let metadata = Metadata {
        transfer_id: transfer_id.clone(),
        rel_path,
        size,
        is_dir: false,
    };
    write_json_line(&mut send, &metadata).await?;

    // Ack métadonnées
    let mut ack = [0u8; 1];
    recv.read_exact(&mut ack).await.map_err(quinn_to_err)?;
    if ack[0] != ACK {
        return Err(io_err("metadonnees rejetees par le receveur"));
    }

    // Chunks — pas d'ack par chunk : QUIC assure la fiabilite, on pipeline
    let mut file = fs::File::open(&abs_path).await?;
    let mut buf = vec![0u8; CHUNK_SIZE];
    let mut file_sent: u64 = 0;
    let mut throttle = UiThrottle::new();

    loop {
        if stop.load(Ordering::Relaxed) {
            let _ = send.reset(0u32.into());
            return Err(io_err("transfert annule par l'utilisateur"));
        }

        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        send.write_all(&(n as u32).to_be_bytes())
            .await
            .map_err(write_quinn_to_err)?;
        send.write_all(&buf[..n]).await.map_err(write_quinn_to_err)?;

        // Progression globale cumulee (lot) + per-fichier
        file_sent += n as u64;
        let total_sent = bytes_sent_counter.fetch_add(n as u64, Ordering::Relaxed) + n as u64;
        if throttle.ready() || file_sent == size {
            ui.update_progress_bar(&transfer_id, total_sent, total_bytes);
            ui.file_progress_bar(&transfer_id, &file_name, file_sent, size);
        }
    }

    // Marqueur de complétion
    send.write_all(&[COMPLETE]).await.map_err(write_quinn_to_err)?;

    // Ack final
    let mut final_ack = [0u8; 1];
    recv.read_exact(&mut final_ack).await.map_err(quinn_to_err)?;
    send.finish()?;

    if final_ack[0] != ACK {
        return Err(io_err("le receveur a rejete le fichier"));
    }

    Ok(())
}

pub async fn write_json_line<T: Serialize>(send: &mut SendStream, value: &T) -> Result<(), ToolError> {
    let mut encoded = serde_json::to_vec(value).map_err(io_err)?;
    encoded.push(b'\n');
    send.write_all(&encoded).await.map_err(write_quinn_to_err)?;
    Ok(())
}
