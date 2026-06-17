# Sécurité — Toolé

## Objectifs

Le système doit protéger contre :
- l'interception des données (confidentialité)
- la modification des fichiers (intégrité)
- les connexions non autorisées (authentification)
- la corruption accidentelle des transferts

---

## TLS 1.3 — Chiffrement obligatoire

Le transfert utilise **TLS 1.3 via rustls**. Il assure :
- **Chiffrement** : les données sont illisibles pour un tiers
- **Authentification** : le receiver s'authentifie via le handshake TLS
- **Intégrité par chunk** : TLS inclut un MAC qui détecte toute modification en transit
- **Perfect Forward Secrecy (PFS)** : même si la clé privée est compromise plus tard, les sessions passées restent protégées

### Fonctionnement

1. Le sender génère un **certificat auto-signé temporaire** (unique par session)
2. Le receiver initie une connexion TCP
3. Handshake TLS 1.3 automatique
4. La session sécurisée démarre — **aucune intervention utilisateur**

> Le certificat étant auto-signé, un attaquant MITM sur le même réseau ad hoc pourrait en théorie intercepter la connexion. Dans la pratique, le réseau ad hoc étant créé spécifiquement pour le transfert et utilisé uniquement entre les deux pairs, la surface d'attaque est quasi inexistante.

---

## Vérification d'intégrité

### SHA-256 (fichier final)

- Hash du fichier complet calculé côté sender avant transfert
- Transmis dans le paquet `Metadata`
- Le receiver recalcule progressivement pendant la réception et compare à la fin
- **Protège contre la modification intentionnelle** et la corruption non détectée par TLS

---

## Modèle de menace

| Attaque | Protégé par | Niveau |
|---|---|---|
| Interception passive (écoute) | TLS 1.3 | ✅ Fort |
| Modification d'un chunk | TLS 1.3 (MAC) + SHA-256 final | ✅ Fort |
| Connexion non autorisée | Réseau ad hoc isolé (pas d'IP routeable) | ✅ Fort |
| MITM actif | TLS + réseau ad hoc pair-à-pair | ✅ Négligeable |
| Corruption réseau | TLS 1.3 | ✅ Fort |
| Reverse du fichier reçu sur disque | ❌ Non traité (hors scope V1) | — |

---

## Notes V1

- Les fichiers reçus sont stockés en clair sur le disque (pas de chiffrement au repos)
- Pas de révocation de certificat
- Pas de validation d'empreinte par l'utilisateur (TLS automatique)

---

> Voir l'architecture : [architecture.md](architecture.md) · Voir le protocole : [protocol.md](protocol.md) · Voir la roadmap : [roadmap.md](roadmap.md)
