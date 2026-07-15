// modules internes de Toolé
pub mod error;
pub use error::ToolError;
pub mod discovery;
pub mod transfer;
pub mod utils;
pub mod recever;
pub mod sender;
pub mod file_certif;
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
}
