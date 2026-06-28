pub mod error;
pub use error::ToolError;
pub mod utils;
pub mod discovery;
use serde::Serialize;

#[derive(Debug,Clone,Serialize)]
pub struct Peer{
    pub hostname:String,
    pub addr:String,
}

pub trait UI: Send + Sync {
    fn log(&self, msg: &str);
    fn peer_found(&self,peer:&Peer);
    fn peer_lost(&self,hostname:&str);
}