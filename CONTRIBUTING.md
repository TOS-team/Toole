# Contribuer à Toolé

## Commencer

1. Lire la [documentation](docs/) pour comprendre le projet :
   - [PRD](docs/PRD.md) — le besoin produit
   - [SRS](docs/SRS.md) — les exigences logicielles
   - [Architecture](docs/architecture.md) — comment le code est organisé
   - [Diagrammes](docs/diagram.md) — flux réseau et machine d'état

2. Builder le projet :

```bash
# Installer les dependances Python
python3 -m venv .venv
.venv/bin/pip install -r requirements.txt

# Compiler le coeur C et le bridge UI
cmake -S . -B build
cmake --build build

# Lancer l'interface
.venv/bin/python src/ui/app.py
```

3. Lancer les tests :

```bash
cmake -S tests -B build/tests
cmake --build build/tests
ctest --test-dir build/tests
```

## Axes majeurs de contribution

### 1. Réécriture en Rust
Le coeur réseau (discovery, contrôle TCP, failover) est actuellement en C. Une réécriture en Rust apporterait de la sécurité mémoire et faciliterait la maintenance cross-platform. L'architecture (3 couches : discovery UDP, contrôle TCP, transfert fichier) resterait la même ; seule l'implémentation changerait. Les headers dans `includes/` servent de contrat d'interface.

### 2. Chiffrement des transferts
Actuellement les fichiers sont transférés en clair avec une simple vérification CRC32. Un axe prioritaire est d'ajouter du chiffrement :
- Négociation de clé à la connexion (ex: ECDH)
- Chiffrement symétrique du flux (ex: ChaCha20-Poly1305 ou AES-GCM)
- Signature des métadonnées pour éviter le tampering

### 3. Autres fonctionnalités
- Envoi de dossiers complets
- File d'attente de transferts
- Améliorations diverses de l'interface

## Procédure

1. Fork le projet
2. Créer une branche : `git checkout -b feature/ma-fonctionnalite`
3. Coder en suivant les conventions existantes
4. Vérifier que les tests passent
5. Ouvrir une Pull Request

Le style de code est libre mais le C suit le standard C11 et les commentaires d'explication sont les bienvenus.

## Générer une release

### Linux
```bash
VERSION=v1.0.0
pip install pyinstaller
cmake -S . -B build && cmake --build build
pyinstaller --onefile --name toole \
  --add-data "$(pwd)/assets/logo.png:assets" \
  --add-data "$(pwd)/assets/logo.ico:assets" \
  --add-data "$(pwd)/build/libtoole_bridge.so:." \
  --distpath dist src/ui/app.py
mkdir -p releases/toole-$VERSION-linux
cp dist/toole releases/toole-$VERSION-linux/
tar czf releases/toole-$VERSION-linux.tar.gz -C releases toole-$VERSION-linux
rm -rf releases/toole-$VERSION-linux
```

Le binaire standalone `dist/toole` contient tout le nécessaire (Python, bridge C, icônes).

### Windows (quand le backend sera synchronisé)
```bash
pip install pyinstaller
cmake -S . -B build && cmake --build build
pyinstaller --onefile --name Toole \
  --add-data "assets/logo.png;assets" \
  --add-data "assets/logo.ico;assets" \
  --add-data "build/toole_bridge.dll;." \
  --distpath dist src/ui/app.py
# copier dist/Toole.exe dans releases/
```
