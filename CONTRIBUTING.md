# Contribuer à Toolé

## Commencer

1. Lire la documentation dans cet ordre :
   - [prd.md](docs/prd.md) — les exigences produit
   - [srs.md](docs/srs.md) — les spécifications logicielles
   - [architecture.md](docs/architecture.md) — architecture technique
   - [protocol.md](docs/protocol.md) — protocole réseau
   - [crypto.md](docs/crypto.md) — chiffrement et TLS
   - [roadmap.md](docs/roadmap.md) — roadmap

2. Configurer l'environnement :
   - Installer Rust : `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
   - Installer Tauri CLI : `cargo install tauri-cli`

3. Builder le projet :

```bash
cargo build
# ou pour l'app Tauri (depuis la racine du workspace) :
cargo tauri dev --manifest-path desktop-app/src-tauri/Cargo.toml
# ou depuis desktop-app/ :
cd desktop-app && cargo tauri dev
```

4. Lancer les tests :

```bash
cargo test
```

---

## Structure du projet

```
toole/
├── core/                       # Bibliothèque Rust pure
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # Trait UI + types
│       ├── error.rs            # ToolError
│       ├── utils.rs            # Fonctions utilitaires
│       ├── discovery.rs        # UDP broadcast
│       └── transfer.rs         # QUIC (à implémenter)
│
├── desktop-app/                # Application Tauri
│   ├── src-tauri/
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json
│   │   ├── capabilities/default.json
│   │   ├── build.rs
│   │   ├── icons/
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs          # Builder Tauri + events
│   │       └── commands.rs     # Commandes IPC
│   └── ui/                     # Frontend Vue 3
│       ├── index.html
│       ├── package.json
│       ├── vite.config.ts
│       ├── tsconfig.json
│       └── src/
│           ├── main.ts
│           ├── App.vue
│           ├── style.css
│           ├── types.ts
│           ├── tauri.ts
│           ├── utils.ts
│           ├── components/
│           │   ├── WelcomeHeader.vue
│           │   ├── FileDropZone.vue
│           │   ├── PeerList.vue
│           │   └── AboutModal.vue
│           └── stores/
│               ├── peers.ts
│               └── files.ts
│
├── docs/
├── assets/
├── README.md
├── LICENSE
└── CONTRIBUTING.md
```

---

## Conventions de commits

Utiliser le format suivant :

```
feat(discovery): add UDP broadcast peer discovery
```

---

## Pull Requests

Chaque fonctionnalité doit avoir :
- une branche dédiée
- une Pull Request
- une review

---

## Procédure

1. Fork le projet
2. Créer une branche : `git checkout -b feature/ma-fonctionnalite`
3. Coder en suivant les conventions
4. Vérifier que les tests passent : `cargo test`
5. Ouvrir une Pull Request

---

## Licence

En contribuant, vous acceptez que votre code soit distribué sous [GNU GPLv3](LICENSE).
