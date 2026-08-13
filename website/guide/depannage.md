# Dépannage

## « Aucun appareil détecté »

### Causes possibles

1. **L'autre application n'est pas ouverte** — Toolé doit être lancé sur les
   deux machines pour se voir.
2. **Réseaux isolés** — les machines doivent être sur le **même sous-réseau**.
   Un partage Wi-Fi de téléphone ou un réseau invité peuvent isoler les appareils.
3. **Réseau qui bloque le broadcast** — certains routeurs / points d'accès
   WiFi bloquent le broadcast UDP (port 58199). L'AP client en mode « client »
   ou « répéteur » filtre souvent les broadcasts.
4. **Pare-feu** — le pare-feu du système peut bloquer l'écoute sur les ports
   **58199** (découverte) et **58200** (transfert).

### Solutions

- Vérifiez que les deux machines sont sur le même réseau : `ping <ip-de-l'autre-machine>`.
- Cliquez sur l'icône **rafraîchir** dans le panneau Appareils.
- Autorisez Toolé dans le pare-feu (voir [Pare-feu](#pare-feu)).

> L'appareil disparaît aussi si **l'application est fermée** ou que le timeout
> de 9 s expire sans réponse.

## Pare-feu

### Windows

1. `Pare-feu Windows Defender` → **Autoriser une application**.
2. Ajoutez `Toolé.exe` (ou `toole`).
3. Cochez les profils **Privé** et **Domaine**.

### Linux

```bash
sudo ufw allow 58199/udp
sudo ufw allow 58200/udp
```

### macOS

Autorisez Toolé dans *Préférences Système → Réseau → Pare-feu → Options*.

## Informations Windows — Éditeur de l'application

Sur Windows, les propriétés du fichier (`Toolé.exe` ou l'installeur) affichent
comme **Éditeur** la valeur **Tiligré Open Space** (onglet *Détails* →
*Éditeur*).

> L'avertissement SmartScreen « **Éditeur inconnu** » au lancement peut
> néanmoins subsister : il ne disparaît qu'avec une **signature de code**
> (certificat Authenticode), qui n'est pas fournie par l'application.

## Le transfert échoue en cours de route

- **« Un ou plusieurs fichiers ont échoué »** : le récepteur a fermé la
  connexion (fermeture de l'app, perte du réseau). Vérifiez que l'application
  est toujours ouverte chez le destinataire et réessayez.
- **L'émetteur reste bloqué** : le récepteur est peut-être occupé par un autre
  transfert. Réessayez après la fin du transfert en cours.

## Le transfert est lent

### Wi-Fi

- Le WiFi est partagé et semi-duplex : le débit réel est environ la **moitié**
  du débit annoncé par la carte.
- Éloignez-vous des interférences (four, micro-ondes, murs épais).
- Utilisez le **5 GHz** si disponible.

### Fenêtres QUIC

Toolé configure des fenêtres QUIC larges (32 Mo) pour le réseau local. Un
réseau avec beaucoup de pertes peut toutefois réduire le débit : le contrôle de
congestion (Cubic) s'adapte automatiquement.

## Les fichiers arrivent dans le mauvais dossier

Les fichiers reçus sont écrits dans le **dossier Téléchargements du système**,
dans un sous-dossier `Toolé/`. Si l'utilisateur courant n'a pas de dossier
Téléchargements, Toolé utilise le dossier temporaire système.

## « Port occupé » à la découverte

Le message **« Erreur découverte »** s'affiche dans le panneau Appareils si le
port 58199 est déjà utilisé par un autre processus. Dans ce cas :

1. Fermez l'autre instance de Toolé (l'application ne supporte pas deux
   instances simultanées sur le même port).
2. Vérifiez qu'aucun autre programme n'écoute sur ce port :
   ```bash
   sudo lsof -i :58199
   ```

## Un fichier reçu semble corrompu ou incomplet

- **Annulation** : un transfert annulé laisse un fichier partiel dans
  `Téléchargements/Toolé/`. Supprimez-le et relancez le transfert.
- L'intégrité des transferts réussis est garantie par **QUIC/TLS 1.3** (chiffre
  chaque paquet et vérifie son authenticité) : une corruption en transit est
  détectée et le paquet est renvoyé automatiquement. Aucune vérification
  manuelle n'est nécessaire.

## L'application ne démarre pas

- **Linux** : les dépendances système (WebKitGTK) doivent être installées.
  Consultez les [prérequis Tauri](https://tauri.app/start/prerequisites/).
- **Certificat** : au premier lancement, Toolé génère un certificat TLS
  auto-signé stocké dans le dossier de données de l'application. Si la
  génération échoue (dossier non accessible en écriture), l'application peut
  ne pas démarrer.

## L'AppImage ne s'ouvre pas (Linux)

L'AppImage embarque les bibliothèques du poste de build (GLib 2.72, Ubuntu
22.04). Sur une distribution récente (GLib ≥ 2.76), les modules GIO du système
(gvfs…) exigent des symboles absents et font crasher WebKitWebProcess : la
fenêtre ne s'ouvre jamais.

Toolé détecte l'exécution sous AppImage (`APPIMAGE`/`APPDIR`) et neutralise
automatiquement ces modules (`GIO_MODULE_DIR=/dev/null`). Si la fenêtre
n'apparaît toujours pas :

1. Lancez l'AppImage depuis un terminal pour voir l'erreur exacte :
   ```bash
   ./Toolé_2.0.0_amd64.AppImage
   ```
2. Sur Wayland avec un GPU NVIDIA, ajoutez éventuellement :
   ```bash
   WEBKIT_DISABLE_COMPOSITING_MODE=1 ./Toolé_2.0.0_amd64.AppImage
   ```
3. En dernier recours, utilisez la **version archive** (`.tar.gz`) du même
   numéro de version : elle utilise les bibliothèques du système et n'a pas
   ces limitations.

---

> [Sommaire](index.md) · Besoin de comprendre le fonctionnement ? →
> [Pour les développeurs](developpeur.md)
