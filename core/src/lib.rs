pub mod error;
pub mod utils;
pub mod discovery;

#[derive(Debug,Clone)]
pub struct Peer{
    pub hostname:String,
    pub addr:String,
}