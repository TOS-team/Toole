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

// ID device : {hostname}-{suffixe base32}. Généré une seule fois puis persisté
// pour rester unique même si deux OS partagent le même hostname.
pub fn device_id() -> String {
    let Some(proj_dirs) = directories::ProjectDirs::from("com", "Tiligre Open Space", "Toole") else {
        return current_hostname();
    };
    let dir = proj_dirs.data_dir();
    if std::fs::create_dir_all(dir).is_err() {
        return current_hostname();
    }

    let file = dir.join("device_id");
    if let Ok(existing) = std::fs::read_to_string(&file) {
        if !existing.trim().is_empty() {
            return existing.trim().to_string();
        }
    }

    let id = format!("{}-{}", current_hostname(), short_suffix());
    let _ = std::fs::write(&file, &id);
    id
}

// 5 caractères base32 sans chiffres ambigus (0 O 1 I L)
fn short_suffix() -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTVWXYZ23456789";
    let mut out = String::new();
    for byte in uuid::Uuid::new_v4().as_bytes().iter().take(4) {
        out.push(ALPHABET[(*byte >> 3) as usize & 0x1F] as char);
        out.push(ALPHABET[*byte as usize & 0x1F] as char);
    }
    out.truncate(5);
    out
}