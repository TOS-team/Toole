# Software Requirement Specifications (SRS)

## 1) Introduction

Toolé est un système P2P qui permet de **détecter des appareils** sur le réseau local par UDP broadcast et de **transférer des fichiers** par QUIC avec chiffrement TLS 1.3 intégré et vérification SHA-256.

> Besoin produit : [prd.md](prd.md)

## 2) Description générale du système

### Contraintes de conception

- Backend en **Rust** avec Tokio async, séparé en workspace `core/` (biblio pure) + `desktop-app/` (Tauri).
- Frontend **HTML / CSS / JS vanilla** (pas de framework).
- Découverte en UDP broadcast sur le réseau local (port 58199).
- Transfert par **QUIC** (port 5200) avec multiplexage de streams.
- Architecture symétrique (pair-à-pair) : chaque instance joue les deux rôles.
- Communication frontend↔backend par **polling** (découverte) et **events** (progression).

## 3) Exigences fonctionnelles

### Découverte (F-001)

- **E-001** : L'application diffuse périodiquement `TOOLE_DISCOVERY` en UDP broadcast toutes les 3s sur le port 58199.
- **E-002** : L'application écoute sur le port 58199 et répond `TOOLE_HERE:<hostname>` à l'expéditeur.
- **E-003** : L'application ignore son propre hostname et sa propre IP pour ne pas s'ajouter elle-même.
- **E-004** : Un pair est retiré de la liste après 9s sans réponse.
- **E-005** : Le frontend affiche la liste des pairs découverts, mise à jour toutes les 2s par polling.

### Démarrage et arrêt (F-002)

- **E-006** : La découverte démarre automatiquement au lancement de l'application.
- **E-007** : La découverte s'arrête automatiquement à la fermeture de la fenêtre.

### Interface découverte (F-003)

- **E-008** : Le nom de la machine hôte est affiché dans l'en-tête.
- **E-009** : La liste des appareils détectés s'affiche avec nom et adresse IP.
- **E-010** : Chaque pair peut être sélectionné individuellement par clic.
- **E-011** : Une case "tout sélectionner" permet de sélectionner/désélectionner tous les pairs.
- **E-012** : Un message "Aucun appareil détecté" s'affiche quand la liste est vide.

### Transfert de fichiers (F-004)

- **E-013** : Le transfert utilise **QUIC** via Quinn, avec multiplexage de streams.
- **E-014** : Plusieurs fichiers peuvent être transférés **en parallèle** sur une même connexion QUIC.
- **E-015** : Les **dossiers** sont transférés récursivement (un stream QUIC par fichier).
- **E-016** : Chaque fichier est découpé en chunks de 1 Mo avec Ack par chunk.
- **E-017** : L'intégrité est vérifiée par SHA-256 (fichier complet).
- **E-018** : Le chunk perdu est renvoyé (timeout 10s, 3 tentatives max).
- **E-019** : Les fichiers peuvent être ajoutés par **glisser-déposer** (fichiers et dossiers).
- **E-020** : Un **sélecteur de fichiers/dossiers natif** est disponible.
- **E-021** : Une **barre de progression** avec vitesse et temps restant est affichée pour chaque fichier.
- **E-022** : Un **bouton Annuler** permet d'interrompre le transfert (par fichier ou global).

## 4) Exigences non fonctionnelles

- Découverte rapide : broadcast toutes les 3s, timeout 9s.
- Architecture modulaire : `core/` (bibliothèque pure) + `desktop-app/` (Tauri).
- Interface glassmorphism dark, responsive.
- Transfert par **streaming mémoire** : pas de chargement complet en RAM.
- **Multiplexage** : pas de Head-of-Line blocking grâce à QUIC.
- Chiffrement obligatoire : TLS 1.3 intégré à QUIC, aucune donnée en clair.

---

> [PRD — vision produit](prd.md) | Lire ensuite : [Architecture technique](architecture.md)
