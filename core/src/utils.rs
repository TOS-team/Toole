use gethostname;

// je recupere le hostname de la machine
pub fn current_hostname()-> String{
    gethostname::gethostname().to_string_lossy().to_string()
}

// dans cette fonction je recupere l'IP Local de la machine
pub fn local_ip()->String{
    // en fait j'ouvre un socket UDP
    let socket = match std::net::UdpSocket::bind("0.0.0.0:0"){
        Ok(s) => s,
        Err(_) => return "127.0.0.1".to_string(),
    };
    // il faut activer le broadcast avant de connect à 255.255.255.255
    if socket.set_broadcast(true).is_err(){
        return "127.0.0.1".to_string();
    }
    // puis on tente la connexion
    if socket.connect("255.255.255.255:1").is_err(){
        return "127.0.0.1".to_string();
    }
    // enfin je recupere l'adresse locale du socket et je recupere mon ip qui servira à filtrer les beacons entrants
    match socket.local_addr() {
        Ok(addr) => addr.ip().to_string(),
        Err(_) => "127.0.0.1".to_string(),
    }
}