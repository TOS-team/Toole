mod utils;

fn main() {
    println!("SSID : {}", utils::gen_ssid());
    println!("Nom : {}", utils::current_hostname());
}
