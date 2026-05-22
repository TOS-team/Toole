# Guide Local (dev rapide)

Hello le BOP, ici on met les commandes minimales pour compiler le bridge C et lancer l'UI Python.

## 1) Installer les dependances Python

```bash
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt
```

## 2) Compiler le coeur C et le bridge UI (Linux)

```bash
cmake -S . -B build
cmake --build build
```

La compilation genere notamment:

```text
build/libtoole_bridge.so
```

C'est cette bibliotheque que `src/ui/bridge_ctypes.py` charge via `ctypes`.

## 3) Lancer l'UI Python

```bash
.venv/bin/python src/ui/app.py
```

## 4) Lancer les tests Linux + bridge Python

```bash
ctest --test-dir build --output-on-failure
```

La suite couvre actuellement:

- discovery UDP Linux
- controle TCP Linux
- transfert fichier Linux avec CRC32 et rejet de nom dangereux
- failover runtime Linux
- chargement Python `ctypes` de `build/libtoole_bridge.so` avec verification de version API

## 5) Capacites UI actuelles

- L'ecran Transfert affiche les fichiers recus depuis le dossier `received/`.
- L'UI maintient une liste d'evenements: pairs detectes/perdus, fichiers recus, envois termines ou partiels.
- Le bridge expose une progression d'envoi en octets via `toole_bridge_get_transfer_status`.
- La barre de progression UI affiche le fichier courant avec octets envoyes / total quand le backend remonte l'information.

## 6) Notes importantes

- Le mode principal actuel passe par l'UI Python + le bridge C.
- Le theme UI est centralise dans `src/ui/theme.py`.
- La partie Windows existe dans le code mais n'est pas la cible stabilisee de ce guide local.

---

>[Architecture](architecture.md)  
>[Diagrammes](diagram.md)  
>[SRS](SRS.md)
