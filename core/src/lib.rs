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
use std::collections::HashMap;
use std::sync::{Arc,Mutex};
use std::sync::mpsc;


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
    // implementation de la progrssion
    fn show_progress_bar(&self,tranfert_id:&str);
    // mise a jour de la  bar de progression
    fn update_progress_bar(&self, transfer_id: &str, bytes_sent: u64, total_bytes: u64);
    // quand un transfert est anuler
    fn transfert_cancel(&self,tranfert_id:&str);
    // quand un transfert est terminer
    fn transfert_completed(&self,tranfert_id:&str);
    // quand il y une errer a un tranfert 
    fn tranfert_error(&self,tranfert_id:&str,error:&ToolError);

}

pub struct Transfer {
    pub cancel_handle: Mutex<HashMap<String,tokio::task::JoinHandle<()>>>,
}

impl Transfer {
    pub fn new() -> Self {
        Transfer {
            cancel_handle: Mutex::new(HashMap::new()),
        }
    }
}
