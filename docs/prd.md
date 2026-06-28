# PRD (Product Requirement Document)

## 1) Contexte

Le transfert de fichiers hors ligne entre ordinateurs reste souvent lent ou compliqué (clé USB, manipulations manuelles, dépendance Internet). **Toolé** veut rendre ça simple avec un transfert pair-à-pair sur réseau local.

**Version actuelle (0.2.0)** : Découverte automatique des appareils sur le LAN.

## 2) Personnes concernées

- **Noaga** : étudiant qui veut partager rapidement des documents entre machines sur le même réseau.
- **Tinbnooma** : profil cybersécurité qui veut transférer des fichiers sensibles sans support physique.

## 3) Fonctionnalités actuelles

- **F-001** : Découverte automatique des appareils sur le LAN via UDP broadcast.
- **F-002** : Démarrage et arrêt automatiques de la découverte.
- **F-003** : Interface liste des appareils avec sélection individuelle ou groupée.

## 4) Fonctionnalités prévues

- **F-004** : Transfert de fichier sécurisé avec TLS + SHA-256 (en conception).
- **F-005** : Barre de progression avec vitesse en temps réel.
- **F-006** : Mode Send / Receive avec sélecteur de fichier natif.

## 5) Exigences non fonctionnelles

- Découverte rapide (broadcast toutes les 3s, timeout 9s).
- Faible consommation mémoire (streaming pour le futur transfert).
- Support Linux, Windows, macOS.

---

> Lire ensuite : [SRS — exigences logicielles](srs.md)
