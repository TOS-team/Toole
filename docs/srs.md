# Software Requirement Specifications (SRS)

## 1) Introduction

Toolé est un système P2P qui permet de **détecter des appareils sur le réseau local** par UDP broadcast. Le transfert de fichiers est en phase de conception.

> Besoin produit : [prd.md](prd.md)

## 2) Description générale du système

### Contraintes de conception

- Backend en **Rust** avec Tokio async, séparé en workspace `core/` (biblio pure) + `desktop-app/` (Tauri).
- Frontend **HTML / CSS / JS vanilla** (pas de framework).
- Découverte en UDP broadcast sur le réseau local (port 58199).
- Architecture symétrique (pair-à-pair) : chaque instance est à la fois émettrice et réceptrice.
- Communication frontend↔backend par **polling** (pas d'events Tauri).

## 3) Exigences fonctionnelles

### Découverte (F-001)

- **E-001** : L'application diffuse périodiquement `TOOLE_DISCOVERY` en UDP broadcast toutes les 3s sur le port 58199.
- **E-002** : L'application écoute sur le port 58199 et répond `TOOLE_HERE:<hostname>` à l'expéditeur.
- **E-003** : L'application ignore son propre hostname et sa propre IP pour ne pas s'ajouter elle-même.
- **E-004** : Un pair est retiré de la liste après 9s sans réponse.
- **E-005** : Le frontend affiche la liste des pairs découverts, mise à jour toutes les 2s par polling.

### Démarrage et arrêt (F-002)

- **E-006** : La découverte démarre automatiquement au lancement de l'application.
- **E-007** : La découverte s'arrête automatiquement à la fermeture de la fenêtre (via `on_window_event` et `beforeunload`).

### Interface (F-003)

- **E-008** : Le nom de la machine hôte est affiché dans l'en-tête.
- **E-009** : La liste des appareils détectés s'affiche avec nom et adresse IP.
- **E-010** : Chaque pair peut être sélectionné individuellement par clic.
- **E-011** : Une case "tout sélectionner" permet de sélectionner/désélectionner tous les pairs.
- **E-012** : Un message "Aucun appareil détecté" s'affiche quand la liste est vide.
- **E-013** : Une modale "À propos" avec version et description.

## 4) Exigences non fonctionnelles

- Découverte rapide : broadcast toutes les 3s, timeout 9s.
- Architecture modulaire : `core/` (bibliothèque pure) + `desktop-app/` (Tauri).
- Interface glassmorphism dark, responsive.

---

> [PRD — vision produit](prd.md) | Lire ensuite : [Architecture technique](architecture.md)
