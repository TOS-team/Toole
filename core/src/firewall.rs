// analyse des pare-feux système pour l'aide au diagnostic : je détecte
// si un pare-feu est actif et si les ports UDP de Toolé (58199 découverte,
// 58200 réception) sont autorisés. Je ne fais que lire la configuration,
// jamais de modification : l'ouverture des ports se fait à l'installation
// (install.sh / NSIS) et les commandes à exécuter sont affichées à l'UI

use std::net::Ipv4Addr;

pub const DISCOVERY_PORT: u16 = 58199;
pub const RECEIVER_PORT: u16 = 58200;

// état consolidé renvoyé à l'interface
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FirewallStatus {
    pub os: String,
    pub active: bool,
    pub ports_open: bool,
    pub commands: Vec<String>,
}

// je déduis l'adresse du groupe multicast... non, rien ici : ce module est
// uniquement de la lecture de configuration. Je garde la structure minimale.

// je parse la sortie de `ufw status` : je vérifie que le pare-feu est actif
// et que les règles 58199/udp et 58200/udp sont présentes. La sortie a ce
// format (locale anglaise par défaut d'ufw) :
//   Status: active
//   To                         Action      From
//   --                         ------      ----
//   58199/udp                  ALLOW       Anywhere
//   58200/udp                  ALLOW       Anywhere (v6)
pub fn ufw_ports_open(status: &str) -> (bool, bool) {
    let active = status.lines().any(|l| l.trim().starts_with("Status: active"));
    let mut udp = false;
    let mut tcp = false;
    for l in status.lines() {
        let t = l.trim();
        if let Some(rest) = t.strip_prefix("58199/udp") {
            udp = rest.trim_start().starts_with("ALLOW");
        } else if let Some(rest) = t.strip_prefix("58200/udp") {
            tcp = rest.trim_start().starts_with("ALLOW");
        }
    }
    (active, udp && tcp)
}

// je parse la sortie de `firewall-cmd --list-ports` (format "58199/udp 58200/udp") :
// je renvoie true si les deux ports UDP sont présents
pub fn firewalld_ports_open(ports: &str) -> bool {
    let mut discovery = false;
    let mut receiver = false;
    for p in ports.split_whitespace() {
        if p == format!("{DISCOVERY_PORT}/udp") {
            discovery = true;
        } else if p == format!("{RECEIVER_PORT}/udp") {
            receiver = true;
        }
    }
    discovery && receiver
}

// commandes à proposer à l'utilisateur selon l'OS, avec les ports Toolé
pub fn commands_for(os: &str) -> Vec<String> {
    match os {
        "linux" => vec![
            format!("sudo ufw allow {DISCOVERY_PORT}/udp"),
            format!("sudo ufw allow {RECEIVER_PORT}/udp"),
        ],
        "windows" => vec![
            format!(
                "netsh advfirewall firewall add rule name=\"Toolé UDP\" dir=in action=allow protocol=UDP localport={DISCOVERY_PORT},{RECEIVER_PORT} profile=private,domain"
            ),
        ],
        _ => vec![],
    }
}

// je construis le statut consolidé pour Linux : je combine ufw et firewalld.
// Les deux peuvent coexister mais en pratique un seul est actif ; si aucun
// n'est présent, je considère le pare-feu inactif (pas d'iptables/nftables
// natif suivi ici).
pub fn linux_status(ufw_active: bool, ufw_open: bool, fw_active: bool, fw_open: bool) -> FirewallStatus {
    let active = ufw_active || fw_active;
    // sans pare-feu actif, les ports sont ouverts par défaut ; sinon je me
    // fie au pare-feu effectivement actif
    let ports_open = if !active {
        true
    } else if ufw_active {
        ufw_open
    } else {
        fw_open
    };
    FirewallStatus {
        os: "linux".to_string(),
        active,
        ports_open,
        commands: commands_for("linux"),
    }
}

// je valide qu'une adresse IPv4 est utilisable pour un pair manuel : je ne
// garde que les adresses unicast routables du LAN privé (RFC 1918) et je
// refuse loopback, multicast et adresses non spécifiées
pub fn is_manual_ipv4_allowed(ip: Ipv4Addr) -> bool {
    if ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() {
        return false;
    }
    let octets = ip.octets();
    matches!(
        octets,
        [10, _, _, _] | [172, 16..=31, _, _] | [192, 168, _, _] | [169, 254, _, _]
    )
}