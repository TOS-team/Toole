# Toolé — transfert de fichiers P2P sur réseau local

Toolé détecte les appareils voisins par UDP broadcast et transfère des fichiers par QUIC (TLS 1.3 intégré). Backend Rust + frontend Vue 3, app Tauri pour desktop.

## Structure

- `core/` — bibliothèque Rust pure, sans Tauri. Modules : `discovery` (UDP 58199), `transfer` (endpoints/streams), `sender` (émetteur, 2 fichiers max en parallèle), `recever` (récepteur QUIC port 58200 → `Téléchargements/Toolé`), `file_certif`, `utils` (device_id), `error`, `lib` (trait `UI` à 10 méthodes)
- `desktop-app/src-tauri/` — app Tauri : `commands.rs` (AppUI implémente le trait `UI` via événements `tool://*`, commandes invoke), `lib.rs` (builder, récepteur au setup)
- `desktop-app/ui/` — Vue 3 + Pinia + Tailwind v4 + Vite. Stores : `peers` (polling 2s), `files`, `transfers` (historique localStorage, 200 entrées), `settings` (thème/halo)
- `tests/rust/` — workspace de tests cargo (15 tests)
- `tests/frontend/` — vitest + jsdom (29 tests)
- `docs/docs/` — SRS, PRD, architecture, protocol, crypto, roadmap

## Commandes

| Commande | Depuis | Rôle |
|---|---|---|
| `cargo test -p toole_tests` | racine | tous les tests Rust |
| `npm test` | `tests/frontend/` | tests frontend vitest |
| `npx tsc --noEmit` | `tests/frontend/` | type-check des tests frontend |
| `npm run build` | `desktop-app/ui/` | build frontend (vue-tsc + vite) |
| `cargo check -p app` | racine | vérifier Rust de l'app |
| `cargo run -p toole_core` | racine | tester la découverte UDP standalone |
| `cargo tauri dev` | `desktop-app/` | lancer l'app en dev |

## Conventions

- Commentaires Rust et JS/TS en français, lowercase, avec `je`
- CSS avec commentaires `/* SECTION */` en majuscules
- `tests/rust` = workspace cargo séparé (`-p toole_tests`), `tests/frontend` = vitest avec mocks Tauri (alias vers `tests/frontend/src/mocks/`)

## Points clés du code

- **Pas de SHA-256 applicatif** : l'intégrité repose sur QUIC/TLS 1.3 (retiré pour la perf)
- **Chunks pipelinés** : `len` u32 BE + data (1 Mo max), pas d'ack par chunk, marqueur `COMPLETE` `0x02`, ack final `0x01`
- **Metadata** : `{ transfer_id, rel_path, size, is_dir }` (JSON + `\n`)
- **Progression UI** limitée à ~20 événements/s (`UiThrottle`) pour ne pas saturer le pont frontend
- **Annulation** : stop flag + reset du stream ; le récepteur signale une erreur, jamais une réception tronquée
- **Découverte** : `TOOLE_HERE:<device_id>` (hostname + suffixe crockford stable), broadcast 3s, timeout 9s
- **Récepteur** démarre au setup de l'app dans `Downloads/Toolé`
