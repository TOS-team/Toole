// tests des utilitaires (utils.rs du core)
//
// je couvre :
//   - short_suffix : le suffixe aléatoire en base32 Crockford (5 caractères,
//     alphabet sans chiffres ambigus)
//   - device_id : le format {hostname}-{suffixe}, stable et persisté
//   - local_ip : une IP v4 non-loopback (ou 127.0.0.1 en secours)

use toole_core::utils::{device_id, local_ip, short_suffix};

// alphabet base32 Crockford : je l'utilise pour vérifier chaque caractère
const CROCKFORD: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

#[test]
fn should_generer_un_suffixe_de_5_caracteres_crockford() {
    // je vérifie le format exact : 5 caractères, tous dans l'alphabet
    let suffix = short_suffix();
    assert_eq!(suffix.len(), 5, "le suffixe doit faire 5 caractères");
    for c in suffix.chars() {
        assert!(
            CROCKFORD.contains(&(c as u8)),
            "caractère '{c}' hors alphabet Crockford"
        );
    }
}

#[test]
fn should_generer_des_suffixes_differents() {
    // deux appels successifs ne doivent pas (quasi) jamais coïncider : je
    // vérifie que sur un échantillon de 50, il y a plus de 10 valeurs uniques
    let mut seen = std::collections::HashSet::new();
    for _ in 0..50 {
        seen.insert(short_suffix());
    }
    assert!(seen.len() > 10, "les suffixes doivent varier, j'ai {seen:?}");
}

#[test]
fn should_device_id_suivre_le_format_hostname_suffixe() {
    // format attendu : {hostname}-{5 caractères Crockford}
    let id = device_id();
    assert!(!id.is_empty(), "device_id ne doit pas être vide");
    let (host, suffix) = id
        .rsplit_once('-')
        .unwrap_or_else(|| panic!("device_id doit contenir un '-': {id}"));
    assert_eq!(suffix.len(), 5, "le suffixe doit faire 5 caractères dans {id}");
    for c in suffix.chars() {
        assert!(
            CROCKFORD.contains(&(c as u8)),
            "caractère '{c}' hors alphabet Crockford dans {id}"
        );
    }
    assert!(
        !host.is_empty(),
        "le hostname ne doit pas être vide dans {id}"
    );
}

#[test]
fn should_device_id_etre_stable_sur_deux_appels() {
    // l'ID est persisté sur disque : deux appels successifs doivent renvoyer
    // exactement la même valeur (c'est ce qui garantit l'unicité multi-OS)
    let a = device_id();
    let b = device_id();
    assert_eq!(a, b, "device_id doit être stable entre deux appels");
}

#[test]
fn should_local_ip_renvoyer_une_ip_valide() {
    // je vérifie qu'on obtient une adresse parseable (v4 ou v6), et jamais
    // une chaîne vide
    let ip = local_ip();
    assert!(!ip.is_empty(), "local_ip ne doit pas être vide");
    let parsed: std::net::IpAddr = ip.parse().unwrap_or_else(|_| {
        panic!("local_ip doit être une IP valide, j'ai {ip:?}");
    });
    assert!(!parsed.is_unspecified(), "local_ip ne doit pas être 0.0.0.0");
}
