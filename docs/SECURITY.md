# Sécurité — Toolé

## Objectifs

Le système doit protéger contre :
- l'interception des données
- la modification des fichiers
- les connexions non autorisées
- la corruption des transferts

---

## TLS obligatoire

Le transfert est sécurisé avec **TLS**.

TLS fournit :
- le chiffrement
- l'authentification
- la protection contre les attaques MITM
- une sécurité standard moderne

### Fonctionnement

1. Le sender génère un **certificat temporaire**
2. Le receiver se connecte
3. Handshake TLS
4. Validation du certificat
5. Acceptation utilisateur
6. Session sécurisée

---

## Vérification d'intégrité

### CRC32

Utilisé pour la validation rapide des chunks. Avantages : très rapide, faible coût CPU.

### SHA-256

Utilisé pour la validation finale du fichier. Avantages : très fiable, sécurisé, pratiquement impossible à falsifier.

---

> Revoir l'architecture : [ARCHITECTURE.md](ARCHITECTURE.md) · Voir la roadmap : [ROADMAP.md](ROADMAP.md)
