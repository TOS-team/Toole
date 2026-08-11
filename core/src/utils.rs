pub fn current_hostname() -> String {
    hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string()
}

// renvoie la première IP v4 non-loopback (la plus fiable pour le filtrage self)
pub fn local_ip() -> String {
    if let Ok(ifaces) = if_addrs::get_if_addrs() {
        for iface in ifaces {
            if iface.is_loopback() {
                continue;
            }
            if let if_addrs::IfAddr::V4(v4) = iface.addr {
                return v4.ip.to_string();
            }
        }
    }
    "127.0.0.1".to_string()
}