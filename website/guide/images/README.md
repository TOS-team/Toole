# Captures d'écran du guide

Ce dossier contient les captures d'écran utilisées dans le guide utilisateur.

## Comment les générer

Lancez l'application avec `cargo tauri dev` (depuis `desktop-app/`), puis faites
la capture de chaque écran avec l'outil de capture du système.

## Fichiers attendus

| Fichier | Contenu |
|---|---|
| `accueil.png` | Page d'accueil : zone de dépôt vide + appareils à droite |
| `decouverte.png` | Panneau droit avec des appareils détectés |
| `transfert.png` | Page Transferts : un transfert en cours avec barres |
| `historique.png` | Page Historique avec quelques entrées |
| `parametres.png` | Page Paramètres (thème + halo) |

## Conventions

- Format **PNG**, résolution native de la fenêtre (890×550 par défaut).
- Faire les captures sur fond **sombre** (thème par défaut) pour correspondre
  aux guides.
- Recadrer uniquement la fenêtre de l'application (pas l'écran entier).
