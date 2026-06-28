# PRD (Product Requirement Document)

## 1) Contexte

Le transfert de fichiers hors ligne entre ordinateurs reste souvent lent ou compliqué (clé USB, manipulations manuelles, dépendance Internet). **Toolé** rend ça simple avec un transfert pair-à-pair sur réseau local : découverte automatique des appareils, sélection par glisser-déposer, transfert parallèle chiffré.

## 2) Personnes concernées

- **Noaga** : étudiant qui veut partager rapidement des documents entre machines sur le même réseau.
- **Tinbnooma** : profil cybersécurité qui veut transférer des fichiers sensibles sans support physique.

## 3) Fonctionnalités

- **F-001** : Découverte automatique des appareils sur le LAN via UDP broadcast.
- **F-002** : Démarrage et arrêt automatiques de la découverte.
- **F-003** : Interface liste des appareils avec sélection individuelle ou groupée.
- **F-004** : Transfert de fichiers par **QUIC** avec multiplexage et chiffrement TLS 1.3 intégré.
- **F-005** : Support de **plusieurs fichiers** en parallèle sur une même connexion QUIC.
- **F-006** : Support des **dossiers** (envoi récursif, conservation de l'arborescence).
- **F-007** : Glisser-déposer et sélecteur de fichiers/dossiers natif.
- **F-008** : Barre de progression avec vitesse et temps restant en temps réel.
- **F-009** : Bouton Annuler par fichier et global.
- **F-010** : Vérification d'intégrité SHA-256 à la réception.

## 4) Exigences non fonctionnelles

- Découverte rapide (broadcast toutes les 3s, timeout 9s).
- Faible consommation mémoire (streaming, pas de chargement complet en RAM).
- Chiffrement obligatoire (TLS 1.3 via QUIC).
- Support Linux, Windows, macOS.

---

> Lire ensuite : [SRS — exigences logicielles](srs.md)
