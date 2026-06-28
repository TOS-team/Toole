# Chiffrement et intégrité — Toolé

## Objectifs

Le système doit protéger contre :
- l'interception des données (confidentialité)
- la modification des fichiers (intégrité)
- les connexions non autorisées (authentification)
- la corruption accidentelle des transferts

---

## QUIC + TLS 1.3 — Chiffrement intégré

Le transfert utilise **QUIC via Quinn**. QUIC intègre **TLS 1.3 nativement** :
- **Chiffrement** : toutes les données sont chiffrées, aucun paquet en clair sur le réseau
- **Authentification** : handshake TLS 1.3 automatique à l'établissement de la connexion
- **Intégrité par paquet** : chaque paquet QUIC est authentifié (aucune modification possible en transit)
- **Perfect Forward Secrecy (PFS)** : même si une clé privée est compromise ultérieurement, les sessions passées restent protégées
- **Multiplexage sécurisé** : chaque stream QUIC est indépendamment chiffré

### Fonctionnement

1. Un pair agit comme **serveur QUIC** (écoute sur le port 5200)
2. L'autre pair initie une **connexion QUIC** (handshake TLS 1.3 automatique)
3. Aucune intervention utilisateur requise pour le chiffrement
4. Les certificats sont auto-signés et générés à la volée (unique par session)

### Vérification d'intégrité

**SHA-256 (fichier final)**
- Hash du fichier complet calculé côté sender avant transfert
- Transmis dans le paquet `Metadata`
- Le receiver recalcule progressivement pendant la réception et compare à la fin
- **Double protection** : QUIC (intégrité paquet) + SHA-256 (intégrité fichier)

---

## Modèle de menace

| Attaque | Protégé par | Niveau |
|---|---|---|
| Interception passive (écoute) | QUIC + TLS 1.3 | ✅ Fort |
| Modification d'un paquet | QUIC (authentification paquet) | ✅ Fort |
| Modification d'un fichier | SHA-256 final | ✅ Fort |
| Connexion non autorisée | TLS 1.3 + réseau local | ✅ Fort |
| MITM actif | TLS 1.3 | ✅ Fort |
| Corruption réseau | QUIC (contrôle congestion + renvoi automatique) | ✅ Fort |
| Reverse du fichier reçu sur disque | ❌ Non traité (hors scope V1) | — |

---

## Notes V1

- Les fichiers reçus sont stockés en clair sur le disque (pas de chiffrement au repos)
- Pas de révocation de certificat
- Pas de validation d'empreinte par l'utilisateur (TLS automatique via QUIC)
- Les certificats auto-signés suffisent pour un réseau local de confiance

---

> [Protocole réseau](protocol.md) | Lire ensuite : [Roadmap](roadmap.md)
