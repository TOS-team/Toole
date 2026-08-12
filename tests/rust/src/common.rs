// helpers partages par tous les tests du crate toole_tests
//
// je regroupe ici :
//   - MockUI : une implementation du trait toole_core::UI qui enregistre
//     chaque evenement dans un état partagé, pour pouvoir asserter dessus
//   - PORT_LOCK : un mutex global pour sérialiser les tests de transfert qui
//     bindent le port UDP 58200 (les tests cargo tournent en parallèle sinon)
//   - helpers de fichiers temp / répertoires uniques

use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use toole_core::{Peer, ToolError, UI};

/// état collecté par MockUI pendant un test
#[derive(Debug, Default)]
pub struct UiState {
    pub log_messages: Vec<String>,
    pub peers_found: Vec<Peer>,
    pub peers_lost: Vec<String>,
    pub progress_bars: Vec<u64>,
    pub file_progress: Vec<(String, u64, u64)>,
    pub completed: Vec<String>,
    pub cancelled: Vec<String>,
    pub errors: Vec<String>,
    pub received: Vec<(String, String, u64, Vec<String>)>,
}

/// implementation factice du trait UI : je mémorise les événements au lieu
/// de toucher à une vraie interface, pour pouvoir les vérifier dans les tests
pub struct MockUI {
    pub state: Arc<Mutex<UiState>>,
    pub progress_count: Arc<AtomicUsize>,
}

impl MockUI {
    pub fn new() -> Self {
        MockUI {
            state: Arc::new(Mutex::new(UiState::default())),
            progress_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// nombre de fois où update_progress_bar a été appelé (je vérifie que le
    /// throttle 50ms ne spamme pas le webview)
    pub fn progress_events(&self) -> usize {
        self.progress_count.load(Ordering::Relaxed)
    }

    pub fn is_completed(&self, transfer_id: &str) -> bool {
        self.state
            .lock()
            .unwrap()
            .completed
            .iter()
            .any(|id| id == transfer_id)
    }

    pub fn is_cancelled(&self, transfer_id: &str) -> bool {
        self.state
            .lock()
            .unwrap()
            .cancelled
            .iter()
            .any(|id| id == transfer_id)
    }
}

impl Default for MockUI {
    fn default() -> Self {
        Self::new()
    }
}

/// attend qu'un message contenant `needle` apparaisse dans les logs de l'UI.
/// Remplace l'ancien délai fixe : on ne se connecte à un service réseau qu'une
/// fois qu'il a signalé être prêt (jonction robuste sous charge CPU).
pub async fn wait_for_log(ui: &MockUI, needle: &str, timeout: Duration) {
    wait_until(
        || {
            let state = ui.state.lock().unwrap();
            state.log_messages.iter().any(|m| m.contains(needle))
        },
        timeout,
        &format!("le message de log {needle:?}"),
    )
    .await;
}

/// attend que l'UI ait détecté au moins un pair (découverte terminée).
pub async fn wait_for_peer(ui: &MockUI, timeout: Duration) {
    wait_until(
        || !ui.state.lock().unwrap().peers_found.is_empty(),
        timeout,
        "un pair détecté",
    )
    .await;
}

/// boucle d'attente active : vérifie `pred` jusqu'à ce qu'elle soit vraie ou
/// que le timeout soit atteint, au lieu de compter sur un délai fixe.
async fn wait_until(mut pred: impl FnMut() -> bool, timeout: Duration, what: &str) {
    let deadline = std::time::Instant::now() + timeout;
    while !pred() {
        assert!(
            std::time::Instant::now() < deadline,
            "timeout en attendant {what}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

impl UI for MockUI {
    fn log(&self, msg: &str) {
        self.state
            .lock()
            .unwrap()
            .log_messages
            .push(msg.to_string());
    }

    fn peer_found(&self, peer: &Peer) {
        self.state.lock().unwrap().peers_found.push(peer.clone());
    }

    fn peer_lost(&self, hostname: &str) {
        self.state
            .lock()
            .unwrap()
            .peers_lost
            .push(hostname.to_string());
    }

    fn show_progress_bar(&self, _transfer_id: &str) {}

    fn update_progress_bar(&self, _transfer_id: &str, _bytes_sent: u64, _total_bytes: u64) {
        self.progress_count.fetch_add(1, Ordering::Relaxed);
    }

    fn file_progress_bar(
        &self,
        _transfer_id: &str,
        file_name: &str,
        file_bytes_sent: u64,
        file_total_bytes: u64,
    ) {
        self.state.lock().unwrap().file_progress.push((
            file_name.to_string(),
            file_bytes_sent,
            file_total_bytes,
        ));
    }

    fn transfert_cancel(&self, transfer_id: &str) {
        self.state
            .lock()
            .unwrap()
            .cancelled
            .push(transfer_id.to_string());
    }

    fn transfert_completed(&self, transfer_id: &str) {
        self.state
            .lock()
            .unwrap()
            .completed
            .push(transfer_id.to_string());
    }

    fn transfert_received(&self, transfer_id: &str, peer: &str, bytes: u64, files: Vec<String>) {
        self.state.lock().unwrap().received.push((
            transfer_id.to_string(),
            peer.to_string(),
            bytes,
            files,
        ));
    }

    fn transfert_error(&self, _transfer_id: &str, error: &ToolError) {
        self.state.lock().unwrap().errors.push(error.to_string());
    }
}

/// verrou global : les tests e2e bindent le port UDP 58200 (récepteur), et
/// cargo les lance en parallèle par défaut. Je les sérialise pour éviter les
/// conflits de port, au prix d'une exécution séquentielle (accepté).
/// Mutex async plutôt que std pour ne pas bloquer la boucle d'événements.
pub static PORT_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// crée un répertoire temp unique par test (préfixe donné) et le retourne,
/// pour que chaque test isole ses fichiers sans marcher sur les autres.
/// TempDir est supprimé automatiquement à la fin du test (même en cas de
/// panic) : j'évite ainsi d'accumuler des Go dans /tmp entre les runs, ce qui
/// avait fini par faire échouer les tests (quota disque dépassé).
pub fn temp_dir(prefix: &str) -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix(&format!("toole_test_{prefix}_"))
        .tempdir()
        .expect("je dois pouvoir créer le répertoire temp")
}

/// écrit un fichier de `size` octets (contenu pseudo-aléatoire mais
/// déterministe via un LCG) à l'emplacement donné, et renvoie le contenu
pub fn write_random_file(path: &Path, size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    let mut x: u32 = 0x1234_5678;
    for _ in 0..size {
        // LCG simple : contenu reproductible pour comparer source/destination
        x = x.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        data.push((x >> 24) as u8);
    }
    std::fs::write(path, &data).expect("je dois pouvoir écrire le fichier temp");
    data
}

/// je vérifie que deux fichiers sont strictement identiques octet à octet
pub fn files_equal(a: &Path, b: &Path) -> bool {
    let a = std::fs::read(a).expect("je dois pouvoir lire a");
    let b = std::fs::read(b).expect("je dois pouvoir lire b");
    a == b
}

/// je marque le test comme « à faire » : les tests e2e de réseau peuvent
/// être indisponibles sur certaines machines CI, je les isole ici
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
}

/// petit drapeau partagé pour arrêter proprement les tâches longues
pub fn shared_stop() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}
