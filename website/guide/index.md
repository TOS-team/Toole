# Guide Toolé

Toolé est un outil de **transfert de fichiers P2P sur réseau local** : il détecte
automatiquement les appareils voisins et envoie des fichiers sans passer par
Internet, de façon chiffrée (QUIC/TLS 1.3).

## Sommaire

| Document | Contenu |
|---|---|
| [Prise en main](#prise-en-main) | installation, premier envoi, destination des fichiers |
| [Utilisation complète](utilisation.md) | découverte, envoi, réception, historique, paramètres, raccourcis |
| [Dépannage](depannage.md) | problèmes fréquents et leurs solutions |
| [Pour les développeurs](developpeur.md) | comment Toolé fonctionne en interne |

## Prise en main

1. **Installez Toolé** sur au moins deux machines du même réseau (Wi-Fi ou
   filaire). Les machines doivent être sur le même sous-réseau.
2. **Lancez l'application** sur les deux machines. Le récepteur écoute
   automatiquement sur le port **58200** et les fichiers arrivent dans
   `Téléchargements/Toolé/`.
3. **Attendez** que les appareils apparaissent dans le panneau de droite
   (broadcast toutes les 3 s, timeout de 9 s).
4. **Ajoutez des fichiers** dans la zone de dépôt (glisser-déposer, clic pour
   parcourir, ou `Ctrl+V`).
5. **Cochez** un ou plusieurs appareils puis cliquez sur **Transférer**.

> Les captures d'écran sont dans [`images/`](images/) — [voir la liste](#captures-décran).

## Captures d'écran

*(à ajouter)*

| Capture | Description |
|---|---|
| `images/accueil.png` | Page d'accueil avec la zone de dépôt |
| `images/decouverte.png` | Appareils détectés dans le panneau latéral |
| `images/transfert.png` | Transfert en cours avec barres de progression |
| `images/historique.png` | Page historique |
| `images/parametres.png` | Page paramètres (thème + halo) |

---

> Lire ensuite : [Utilisation complète](utilisation.md)
