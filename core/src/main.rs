use toole_core::discovery;

#[tokio::main]
async fn main() {
    println!("Démarrage de Toolé Discovery...");
    if let Err(e) = discovery::start_discovery().await {
        eprintln!("Erreur : {}", e);
    }
}
