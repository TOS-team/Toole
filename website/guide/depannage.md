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
- **Ajoutez l'autre appareil manuellement** : dans le panneau Appareils, cliquez
  sur **« Ajouter un appareil par IP »** puis saisissez son **IPv4 locale**
  (ex. `192.168.1.42`). Toolé s'y connecte directement, sans dépendre du
  broadcast. Fonctionne pour `10.x`, `172.16–31.x`, `192.168.x` et
  `169.254.x` (link-local). L'appareil porte l'étiquette **« manuel »** : vous
  pouvez le retirer à tout moment avec son bouton 🗑.

> L'appareil disparaît aussi si **l'application est fermée** ou que le timeout
> de 9 s expire sans réponse.

## Pare-feu

> Depuis la **v2.1**, `install.sh` ouvre automatiquement les ports
> **58199/58200** (ufw si actif, sinon firewalld) et l'installeur Windows ajoute
> la règle `Toolé UDP` quand il est lancé avec les droits admin. Si un pare-feu
> bloque encore, Toolé affiche **une bannière** dans l'application avec les
> commandes à exécuter.

### Windows

1. `Pare-feu Windows Defender` → **Autoriser une application**.
2. Ajoutez `Toolé.exe` (ou `toole`).
3. Cochez les profils **Privé** et **Domaine**.

Ou en ligne de commande (admin) :

```powershell
netsh advfirewall firewall add rule name="Toolé UDP" dir=in action=allow protocol=UDP localport=58199,58200 profile=private,domain
```

### Linux

```bash
sudo ufw allow 58199/udp
sudo ufw allow 58200/udp
```

Ou avec firewalld :

```bash
sudo firewall-cmd --permanent --add-port=58199/udp --add-port=58200/udp
sudo firewall-cmd --reload
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

## « The signature was created with a different key »

Lors d'une mise à jour automatique, l'app vérifie la signature du fichier
téléchargé avec la clé publique embarquée. Cette erreur signifie que la clé
de signature des releases (dans les secrets GitHub) ne correspond pas à la
clé embarquée dans l'app. Cela arrive si le couple de clés a été régénéré
sans mettre à jour `plugins.updater.pubkey` dans
`desktop-app/src-tauri/tauri.conf.json`.

La solution : aligner `pubkey` avec la clé privée `TAURI_SIGNING_PRIVATE_KEY`
des secrets, reconstruire la release, puis **réinstaller une fois** l'app de
base à la main (`.deb`/`.rpm`) — l'installation existante porte encore
l'ancienne clé et refusera toute mise à jour.

---

> [Sommaire](index.md) · Besoin de comprendre le fonctionnement ? →
> [Pour les développeurs](developpeur.md)
