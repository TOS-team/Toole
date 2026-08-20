// tests du module firewall du core : parsage des sorties ufw / firewalld
// et validation des adresses IPv4 pour les pairs manuels. Aucun port réseau
// utilisé ici, donc pas besoin de PORT_LOCK.

use toole_core::firewall::{
    commands_for, firewalld_ports_open, is_manual_ipv4_allowed, linux_status, ufw_ports_open,
};
use toole_core::utils::manual_peer;

const UFW_ACTIF_SANS_TOOLE: &str = "Status: active
To                         Action      From
--                         ------      ----
22/tcp                     ALLOW       Anywhere
80/tcp                     ALLOW       Anywhere
";

const UFW_ACTIF_AVEC_TOOLE: &str = "Status: active
To                         Action      From
--                         ------      ----
22/tcp                     ALLOW       Anywhere
58199/udp                  ALLOW       Anywhere
58200/udp                  ALLOW       Anywhere (v6)
";

const UFW_INACTIF: &str = "Status: inactive
";

#[test]
fn should_parer_ufw_actif_avec_ports_toolé() {
    let (active, open) = ufw_ports_open(UFW_ACTIF_AVEC_TOOLE);
    assert!(active, "le pare-feu doit être détecté actif");
    assert!(open, "les deux ports UDP doivent être ouverts");
}

#[test]
fn should_detecter_ufw_actif_sans_les_ports() {
    let (active, open) = ufw_ports_open(UFW_ACTIF_SANS_TOOLE);
    assert!(active, "le pare-feu doit être détecté actif");
    assert!(!open, "les ports ne doivent pas être considérés ouverts");
}

#[test]
fn should_ignorer_ufw_inactif() {
    let (active, open) = ufw_ports_open(UFW_INACTIF);
    assert!(!active, "ufw inactif ne doit pas déclencher d'alerte");
    assert!(!open);
}

#[test]
fn should_parer_firewalld_ports() {
    assert!(
        firewalld_ports_open("58199/udp 58200/udp"),
        "les deux ports présents doivent être détectés"
    );
    assert!(
        !firewalld_ports_open("22/tcp 58199/udp"),
        "il manque 58200/udp"
    );
    assert!(!firewalld_ports_open(""), "liste vide : rien d'ouvert");
}

#[test]
fn should_combiner_ufw_et_firewalld() {
    // ufw actif sans ports + firewalld inactif → alerte nécessaire
    let s = linux_status(true, false, false, false);
    assert!(s.active);
    assert!(!s.ports_open);
    // firewalld actif avec ports → rien à signaler
    let s2 = linux_status(false, false, true, true);
    assert!(s2.active);
    assert!(s2.ports_open);
    // aucun pare-feu → rien à signaler
    let s3 = linux_status(false, false, false, false);
    assert!(!s3.active);
    assert!(s3.ports_open);
}

#[test]
fn should_proposer_des_commandes_par_os() {
    let ufw = commands_for("ufw");
    assert_eq!(ufw.len(), 2);
    assert!(ufw[0].contains("ufw allow 58199/udp"));
    assert!(ufw[1].contains("ufw allow 58200/udp"));
    let fw = commands_for("firewalld");
    assert_eq!(fw.len(), 3);
    assert!(fw[0].contains("firewall-cmd --permanent --add-port=58199/udp"));
    assert!(fw[1].contains("firewall-cmd --permanent --add-port=58200/udp"));
    assert!(fw[2].contains("--reload"));
    let windows = commands_for("windows");
    assert!(windows[0].contains("netsh advfirewall"));
    assert!(windows[0].contains("58199,58200"));
    assert!(commands_for("macos").is_empty());
    assert!(commands_for("linux").is_empty());
}

#[test]
fn should_choisir_les_commandes_du_pare_feu_actif() {
    // ufw actif sans ports → commandes ufw
    let s = linux_status(true, false, false, false);
    assert!(s.active);
    assert!(!s.ports_open);
    assert_eq!(s.commands.len(), 2);
    assert!(s.commands[0].contains("ufw"));
    // firewalld actif sans ports → commandes firewalld, jamais d'ufw
    let s2 = linux_status(false, false, true, false);
    assert!(s2.active);
    assert!(!s2.ports_open);
    assert_eq!(s2.commands.len(), 3);
    assert!(s2.commands[0].contains("firewall-cmd"));
    assert!(!s2.commands.iter().any(|c| c.contains("ufw")));
    // aucun pare-feu → rien à signaler ni à proposer
    let s3 = linux_status(false, false, false, false);
    assert!(!s3.active);
    assert!(s3.ports_open);
    assert!(s3.commands.is_empty());
}

#[test]
fn should_valider_les_ipv4_privées_pour_les_pairs_manuels() {
    assert!(is_manual_ipv4_allowed("192.168.1.42".parse().unwrap()));
    assert!(is_manual_ipv4_allowed("10.0.0.7".parse().unwrap()));
    assert!(is_manual_ipv4_allowed("172.16.4.4".parse().unwrap()));
    assert!(is_manual_ipv4_allowed("169.254.10.10".parse().unwrap()));
    assert!(!is_manual_ipv4_allowed("127.0.0.1".parse().unwrap()));
    assert!(!is_manual_ipv4_allowed("0.0.0.0".parse().unwrap()));
    assert!(!is_manual_ipv4_allowed("8.8.8.8".parse().unwrap()));
    assert!(!is_manual_ipv4_allowed("224.0.0.1".parse().unwrap()));
}

#[test]
fn should_creer_un_pair_manuel_avec_une_ip_valide() {
    let peer = manual_peer("192.168.1.42").expect("IPv4 privée valide");
    assert_eq!(peer.addr, "192.168.1.42");
    assert_eq!(peer.id, "manual-192.168.1.42");
    // l'espace autour de l'IP ne doit pas gêner
    let peer2 = manual_peer("  10.0.0.7 ").expect("IPv4 privée valide");
    assert_eq!(peer2.addr, "10.0.0.7");
}

#[test]
fn should_refuser_les_ip_non_privées() {
    assert!(manual_peer("8.8.8.8").is_none(), "IPv4 publique refusée");
    assert!(manual_peer("127.0.0.1").is_none(), "loopback refusé");
    assert!(manual_peer("::1").is_none(), "IPv6 refusé");
    assert!(manual_peer("192.168.1.999").is_none(), "IP invalide refusée");
    assert!(manual_peer("example.com").is_none(), "hostname refusé");
}