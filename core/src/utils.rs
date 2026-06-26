use gethostname::gethostname;

//ic je recupère le nom de la machine hote
pub fn current_hostname()-> String{
    gethostname().to_string_lossy().to_string()
}

//je vais me servir de ce nom pour generer le SSID du hotspot
pub fn gen_ssid()-> String{
    format!("Toole-{}",current_hostname())
}