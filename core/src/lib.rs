// modules internes de Toolé
pub mod error;
pub use error::ToolError;
pub mod discovery;
pub mod file_certif;
pub mod receiver;
pub mod sender;
pub mod transfer;
pub mod utils;
use serde::Serialize;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

// ici je defini la structure d'un pair sur le reseau
// chaque pair a un id jolie et unique (hostname-suffixe) et une addresse IP
#[derive(Debug, Clone, Serialize)]
pub struct Peer {
    pub id: String,
    pub addr: String,
}

// trait UI pour communiquer avec l'interface utilisateur
// chaque implementation (Tauri, Null, etc.) doit fournir ces methodes
pub trait UI: Send + Sync {
    // je log un message dans l'interface
    fn log(&self, msg: &str);

    // je signale qu'un nouveau pair est trouve
    fn peer_found(&self, peer: &Peer);

    // je signale qu'un pair a ete perdu
    fn peer_lost(&self, hostname: &str);

    // affichage initial de la barre de progression pour ce transfert
    fn show_progress_bar(&self, transfer_id: &str);

    // mise a jour de la progression (octets envoyes / total)
    fn update_progress_bar(&self, transfer_id: &str, bytes_sent: u64, total_bytes: u64);

    // progression d'un fichier individuel dans un lot (nom de fichier + octets)
    fn file_progress_bar(
        &self,
        transfer_id: &str,
        file_name: &str,
        file_bytes_sent: u64,
        file_total_bytes: u64,
    );

    // quand un transfert entrant demande la validation de l'utilisateur :
    // le recepteur affiche la demande (accepter / refuser) avec les infos
    // du lot (emetteur, taille totale, liste des fichiers)
    fn transfert_incoming(
        &self,
        transfer_id: &str,
        sender: &str,
        total_bytes: u64,
        files: Vec<String>,
    );

    // quand un transfert est refuse par le destinataire
    fn transfert_refused(&self, transfer_id: &str);

    // quand un transfert est annule par l'utilisateur
    fn transfert_cancel(&self, transfer_id: &str);

    // quand un transfert est termine avec succes
    fn transfert_completed(&self, transfer_id: &str);

    // quand des fichiers ont ete recus par ce device
    fn transfert_received(&self, transfer_id: &str, peer: &str, bytes: u64, files: Vec<String>);

    // quand une erreur survient pendant un transfert
    fn transfert_error(&self, transfer_id: &str, error: &ToolError);
}

// registre des transferts actifs : le récepteur s'y enregistre dès qu'un lot
// est identifié (transfer_id connu) pour que l'app puisse annuler la réception
// en cours depuis l'interface. Le stop est le drapeau d'arrêt de la connexion.
pub trait TransferRegistry: Send + Sync {
    fn register(&self, transfer_id: &str, stop: Arc<AtomicBool>);
    fn unregister(&self, transfer_id: &str);
}
