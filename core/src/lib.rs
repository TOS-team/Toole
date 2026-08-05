// modules internes de Toolé
pub mod error;
pub use error::ToolError;
pub mod discovery;
pub mod file_certif;
pub mod recever;
pub mod sender;
pub mod transfer;
pub mod utils;
use serde::Serialize;

// ici je defini la structure d'un pair sur le reseau
// chaque pair a un hostname et une addresse IP
#[derive(Debug, Clone, Serialize)]
pub struct Peer {
    pub hostname: String,
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

    // quand un transfert est annule par l'utilisateur
    fn transfert_cancel(&self, transfer_id: &str);

    // quand un transfert est termine avec succes
    fn transfert_completed(&self, transfer_id: &str);

    // quand une erreur survient pendant un transfert
    fn tranfert_error(&self, transfer_id: &str, error: &ToolError);
}
