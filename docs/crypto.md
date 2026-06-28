# Chiffrement et intégrité — Toolé

## État actuel

**Aucun mécanisme de chiffrement ou de transfert n'est encore implémenté.**

La version actuelle (0.2.0) se limite à la **découverte UDP** des appareils sur le réseau local. Aucune donnée fichier n'est transférée.

---

## Plan pour le transfert

### TLS 1.3 — Chiffrement obligatoire

Le transfert utilisera **TLS 1.3 via rustls** pour assurer :
- **Chiffrement** : les données sont illisibles pour un tiers
- **Authentification** : le receiver s'authentifie via le handshake TLS
- **Intégrité par chunk** : TLS inclut un MAC qui détecte toute modification en transit
- **Perfect Forward Secrecy (PFS)** : même si la clé privée est compromise plus tard, les sessions passées restent protégées

### Fonctionnement prévu

1. Le sender génère un **certificat auto-signé temporaire** (unique par session)
2. Le receiver initie une connexion TCP
3. Handshake TLS 1.3 automatique
4. La session sécurisée démarre — **aucune intervention utilisateur**

### Vérification d'intégrité

**SHA-256 (fichier final)**
- Hash du fichier complet calculé côté sender avant transfert
- Transmis dans le paquet `Metadata`
- Le receiver recalcule progressivement pendant la réception et compare à la fin

---

## Modèle de menace (planifié)

| Attaque | Protégé par | Niveau |
|---|---|---|
| Interception passive (écoute) | TLS 1.3 | ✅ Fort |
| Modification d'un chunk | TLS 1.3 (MAC) + SHA-256 final | ✅ Fort |
| Connexion non autorisée | TLS + réseau local | ✅ Fort |
| MITM actif | TLS 1.3 | ✅ Fort |
| Corruption réseau | TLS 1.3 | ✅ Fort |
| Reverse du fichier reçu sur disque | ❌ Non traité (hors scope V1) | — |

---

## Notes V1

- Les fichiers reçus seront stockés en clair sur le disque (pas de chiffrement au repos)
- Pas de révocation de certificat
- Pas de validation d'empreinte par l'utilisateur (TLS automatique)

---

> [Protocole réseau](protocol.md) | Lire ensuite : [Roadmap](roadmap.md)
