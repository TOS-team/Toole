# PRD (Product Requirement Document)

## 1) Contexte

Le transfert de fichiers hors ligne entre ordinateurs reste souvent lent ou compliqué (clé USB, manipulations manuelles, dépendance Internet). **Toolé** veut rendre ça simple avec un transfert pair-à-pair en réseau local.

> Vue problème détaillée : [brouillon.md](brouillon.md)

## 2) Personnes concernées

- **Noaga** : étudiant qui veut partager rapidement des documents dans un environnement avec réseau faible.
- **Tinbnooma** : profil cybersécurité qui veut transférer des fichiers sensibles sans support physique.

## 3) Inventaire des fonctionnalités

- **F-001** : Découverte automatique des appareils sur le LAN.
- **F-002** : Transfert de fichier sécurisé et fiable avec TLS.
- **F-003** : Interface simple (Send / Receive).

## 4) Spécifications détaillées

### F-001 — Découverte automatique

- Le système détecte les appareils en LAN via UDP broadcast.
- Les senders diffusent leur présence, les receivers les découvrent.
- L'utilisateur voit la liste des appareils disponibles.

### F-002 — Transfert sécurisé

- Connexion TCP établie entre sender et receiver.
- Chiffrement TLS obligatoire avec certificat temporaire.
- Vérification d'intégrité : CRC32 par chunk + SHA-256 fichier final.
- Transfert par chunks sans charger le fichier en RAM.

### F-003 — Interface simple

- Deux boutons principaux : Send / Receive.
- Affichage de la progression en temps réel.
- Liste des appareils détectés avec choix du sender.

## 5) Exigences non fonctionnelles

- Découverte réseau rapide.
- Transfert fiable sans corruption de fichier.
- Faible consommation mémoire (streaming).
- Support Linux, Windows, macOS.

---

> [ARCHITECTURE.md](ARCHITECTURE.md) — [PROTOCOL.md](PROTOCOL.md) — [SECURITY.md](SECURITY.md)
