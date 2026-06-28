# Chiffrement et intégrité — Toolé

## État actuel

**Aucun mécanisme de chiffrement ou de transfert n'est encore implémenté.**

La version actuelle (0.2.0) se limite à la **découverte UDP** des appareils sur le réseau local. Aucune donnée fichier n'est transférée.

---

## Plan pour le transfert

### QUIC + TLS 1.3 — Chiffrement intégré

Le transfert utilisera **QUIC via Quinn**. QUIC intègre **TLS 1.3 nativement** :
- **Chiffrement** : toutes les données sont chiffrées, pas de paquet en clair
- **Authentification** : handshake TLS 1.3 automatique à l'établissement de la connexion
- **Intégrité par paquet** : chaque paquet QUIC est authentifié (pas de modification possible)
- **Perfect Forward Secrecy (PFS)** : même si une clé privée est compromise, les sessions passées restent protégées
- **Multiplexage sécurisé** : chaque stream QUIC est indépendamment chiffré

### Fonctionnement prévu

1. Un pair agit comme **serveur QUIC** (écoute sur le port 5200)
2. L'autre pair initie une **connexion QUIC** (handshake TLS 1.3 automatique)
3. Chaque fichier = un **stream bidirectionnel** dédié
4. Tous les streams en parallèle — **aucune intervention utilisateur** pour le chiffrement

### Vérification d'intégrité

**SHA-256 (fichier final)**
- Hash du fichier complet calculé côté sender avant transfert
- Transmis dans le paquet `Metadata`
- Le receiver recalcule progressivement pendant la réception et compare à la fin
- Double protection : QUIC (intégrité paquet) + SHA-256 (intégrité fichier)

---

## Modèle de menace (planifié)

| Attaque | Protégé par | Niveau |
|---|---|---|
| Interception passive (écoute) | QUIC + TLS 1.3 | ✅ Fort |
| Modification d'un paquet | QUIC (authentification paquet) | ✅ Fort |
| Modification d'un fichier | SHA-256 final | ✅ Fort |
| Connexion non autorisée | TLS 1.3 + réseau local | ✅ Fort |
| MITM actif | TLS 1.3 | ✅ Fort |
| Corruption réseau | QUIC (contrôle congestion + renvoi) | ✅ Fort |
| Reverse du fichier reçu sur disque | ❌ Non traité (hors scope V1) | — |

---

## Notes V1

- Les fichiers reçus seront stockés en clair sur le disque (pas de chiffrement au repos)
- Pas de révocation de certificat
- Pas de validation d'empreinte par l'utilisateur (TLS automatique via QUIC)

---

> [Protocole réseau](protocol.md) | Lire ensuite : [Roadmap](roadmap.md)
