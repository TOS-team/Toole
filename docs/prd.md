# PRD (Product Requirement Document)

## 1) Contexte

Le transfert de fichiers hors ligne entre ordinateurs reste souvent lent ou compliqué (clé USB, manipulations manuelles, dépendance Internet). **Toolé** veut rendre ça simple avec un transfert pair-à-pair sur réseau local.

## 2) Personnes concernées

- **Noaga** : étudiant qui veut partager rapidement des documents entre machines sur le même réseau.
- **Tinbnooma** : profil cybersécurité qui veut transférer des fichiers sensibles sans support physique.

## 3) Inventaire des fonctionnalités

- **F-001** : Découverte automatique des appareils sur le LAN via UDP broadcast.
- **F-002** : Transfert de fichier sécurisé avec TLS + SHA-256.
- **F-003** : Interface simple (Send / Receive).

## 4) Spécifications détaillées

### F-001 — Découverte automatique

- Le sender diffuse sa présence via UDP broadcast sur le réseau local.
- Les receivers écoutent les broadcasts et répondent avec leur nom d'appareil.
- L'utilisateur voit la liste des appareils disponibles.

### F-002 — Transfert sécurisé

- Connexion TCP établie entre sender et receiver.
- Chiffrement TLS 1.3 obligatoire, automatique (aucune validation utilisateur).
- Vérification d'intégrité : SHA-256 fichier final.
- Accusé de réception (Ack) par chunk pour fiabilité.
- Transfert par chunks de 1 Mo sans charger le fichier en RAM.

### F-003 — Interface simple

- Deux boutons principaux : Send / Receive.
- Affichage de la progression en temps réel avec vitesse.
- Liste des appareils détectés avec choix du sender.

## 5) Exigences non fonctionnelles

- Découverte rapide (broadcast toutes les 2s, timeout 10s).
- Transfert fiable sans corruption de fichier (SHA-256).
- Faible consommation mémoire (streaming).
- Support Linux, Windows, macOS.

---

> Lire ensuite : [SRS — exigences logicielles](srs.md)
