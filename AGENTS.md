# Toolé — transfert de fichiers P2P sur réseau local

Toolé détecte les appareils voisins par UDP broadcast et transfère des fichiers par QUIC (TLS 1.3 intégré, sans serveur central). Backend Rust + frontend Vue 3, app Tauri desktop. Commentaires du code et messages de commit en **français**.

## Règles d'or (à lire avant tout)

1. **Ne repasse jamais les hooks `beforeDevCommand`/`beforeBuildCommand` en `sh -c '…'`**. Tauri les exécute avec `cwd` = le dossier parent du `dist` (`desktop-app/`) : sur Windows le hook tourne via `cmd /S /C` où `sh` n'existe pas (build cassé). Le seul formé fiable partout est `cd ui && npm run …`.
2. **La fenêtre est dupliquée volontairement** dans `tauri.conf.json` **et** `tauri.linux.conf.json`, `tauri.macos.conf.json`, `tauri.windows.conf.json`. Toute modif de fenêtre (taille, min, resizable…) doit être répercutée sur **les 4 fichiers** (macOS ajoute `titleBarStyle: "Overlay"` + `hiddenTitle: true`).
3. **ACL Tauri (capabilities)** : tout appel à une API/plugin Tauri depuis le front (window, dialog, process, updater…) doit avoir sa permission dans `capabilities/default.json`, sinon erreur de permission au runtime. Les commandes IPC custom ne passent **pas** par les capabilities.
4. **Nouvelle commande Tauri** : ajoute-la dans `desktop-app/src-tauri/src/commands.rs` **et** dans `invoke_handler` de `lib.rs` (généré par `generate_handler!`). Sinon l'invoke renvoie une erreur.
5. **Les 3 versions doivent rester identiques** : `desktop-app/src-tauri/tauri.conf.json`, `desktop-app/src-tauri/Cargo.toml`, `desktop-app/ui/package.json`. Sinon le job CI `check-versions` bloque la release.
6. **Updater** : ne pas toucher `plugins.updater` (pubkey minisign + endpoint GitHub) par mégarde. La release n'existe qu'avec un tag `v*` + secrets `TAURI_SIGNING_PRIVATE_KEY(_PASSWORD)`.
7. **Tests réseau** : ils bindent les ports UDP `58199`/`58200` et partagent un `PORT_LOCK` global. Ne retire pas la sérialisation ; les tests e2e se sautent quand `CI` est défini.
8. **Intégrité** : pas de SHA-256 applicatif — la fiabilité repose sur QUIC/TLS 1.3. Ne réintroduis pas de hash « pour la sécurité » sans le valider.

## Structure

- `core/` — bibliothèque Rust pure (sans Tauri). Modules : `discovery` (UDP broadcast, port 58199), `transfer` (framing des flux QUIC, `DecisionBoard` pour la demande d'acceptation), `sender` (émetteur, 2 fichiers max en parallèle), `receiver` (récepteur QUIC port 58200 → `Downloads/Toolé`), `file_certif` (certificat auto-signé persisté), `utils` (`device_id`), `error` (`ToolError`), `lib` (trait `UI` à 12 méthodes + trait `TransferRegistry`), `examples/bench.rs` (bench loopback)
- `desktop-app/src-tauri/` — app Tauri : `commands.rs` (9 commandes `#[tauri::command]`, `AppUI` implémente le trait `UI` via events `tool://…`), `lib.rs` (builder, plugins, récepteur au setup), `capabilities/default.json` (ACL)
- `desktop-app/ui/` — Vue 3 + Pinia + Tailwind v4 + Vite. Stores : `peers`, `files`, `transfers`, `settings`, `updater`. `tauri.ts` wrappe `invoke`
- `tests/rust/` — tests d'intégration cargo (`-p toole_tests`) : discovery, protocol, transfer_*, utils
- `tests/frontend/` — vitest + jsdom, avec mocks Tauri (alias vers `tests/frontend/src/mocks/`)
- `docs/docs/` — SRS, PRD, architecture, protocol, crypto, roadmap. `website/` = vitrine + guide Docsify (déployé sur Firebase à chaque push main)

## Commandes

| Commande | Depuis | Rôle |
|---|---|---|
| `cargo test -p toole_tests` | racine | tous les tests d'intégration Rust |
| `cargo check -p app` | racine | type-check Rust de l'app Tauri |
| `cargo run -p toole_core` | racine | démo CLI standalone de la découverte |
| `cargo run -p toole_core --example bench -- 512` | racine | bench de la voie de données (Mo) |
| `npm run build` | `desktop-app/ui/` | type-check + build frontend (`vue-tsc --noEmit && vite build`) |
| `npm test` | `tests/frontend/` | tests frontend vitest |
| `cargo tauri dev` | `desktop-app/` | lancer l'app en dev (vite port 1420, HMR 1421) |
| `npm run dev` | `desktop-app/ui/` | vite seul (port 1420) |

Pas de linter configuré : fais au minimum `cargo check -p app` et `npm run build` avant de finir. **Ne commit pas de modif qui casse le build ou les tests.**

## Conventions

- Commentaires Rust et JS/TS en **français, lowercase, première personne « je … »** (ex. `// je regroupe ici :`). CSS : `/* SECTION */` en majuscules.
- Rust `snake_case`, structs `PascalCase`, constantes `UPPER_SNAKE_CASE`. TS `camelCase` ; composants `.vue` `PascalCase` (TitleBar.vue, HomePage.vue).
- Commits **conventional commits en français** : `feat(core): …`, `fix(ui): …`, `refactor(app): …`, `test: …`, `docs(readme): …`, `bench: …`, `chore: …`. Ex. : `feat(discovery): add UDP broadcast peer discovery`.

## Points clés du code à préserver

- **Framing des flux** : `len` u32 BE + data (chunk 1 Mo max), pas d'ack par chunk (fiabilité QUIC), marqueur `COMPLETE 0x02`, ack final `0x01`.
- **Handshake / décision** : le 1er flux porte le `BatchHeader` (`{ transfer_id, total_bytes, sender, files }` en JSON + `\n`, `sender`/`files` avec `#[serde(default)]`) ; le récepteur présente la demande à l'utilisateur puis répond `ACK 0x01` (accepter) ou `REFUSE 0x03` (refuser) avant tout envoi de fichier. Délai de décision : 30 s (`DECISION_TIMEOUT`).
- **Metadata** : `{ transfer_id, rel_path, size, is_dir }` en JSON + `\n`.
- **Progression UI** limitée à ~20 événements/s (`UiThrottle`) pour ne pas saturer le pont frontend.
- **Codes de fermeture QUIC** : `CLOSE_OK 0` (fin normale), `CLOSE_CANCEL 1` (annulation/refus). Le récepteur distingue annulation (reset ou `CLOSE_CANCEL` → `transfert_cancel`) d'une erreur réseau (autre code/perte → `transfert_error`).
- **Annulation croisée** : les deux côtés peuvent annuler (croix de la carte) ; le récepteur supprime les fichiers partiels, jamais de réception tronquée.
- **Déconnexion soudaine** : `max_idle_timeout 15 s` + `keep_alive_interval 3 s` dans `transport_config` ; détection de la perte en ~15 s + contrôle de complétude `done < expected`.
- **Découverte** : broadcast `TOOLE_HERE:<device_id>` (hostname + suffixe base32 stable), toutes les 3 s, timeout pair 9 s.
- **Linux** : `WEBKIT_DISABLE_DMABUF_RENDERER=1` dans `main.rs` (workaround NVIDIA) + `GIO_MODULE_DIR=/dev/null` **uniquement sous AppImage** (`APPIMAGE`/`APPDIR` posés par le runtime) : les libs embarquées (GLib 2.72, build Ubuntu 22.04) crash WebKitWebProcess avec les modules GIO système (GLib ≥ 2.76) → fenêtre qui ne s'ouvre pas. Ne pas retirer ni étendre à tout Linux.
- `Cargo.lock` est tracké à la racine (ne pas l'ignorer malgré `.gitignore`).

## Attention aussi à

- `capabilities/default.json` commence à `core:default` ; ajoute les permissions spécifiques par-dessus, ne remplace pas tout par `core:default` si un plugin est utilisé.
- Le récepteur démarre au `setup` et écrit dans `Downloads/Toolé`. Un changement de dossier cible implique de nouvelles commandes + capabilities.
- CSP : `style-src 'unsafe-inline'` et `assetProtocol.scope: ["**"]` sont voulus (Tailwind + asset protocol) — documenter avant de resserrer.
- La fenêtre `main` est `decorations: false`, `transparent: true` : les tests visuels dépendent de la couche CSS (sidebar, fenêtre, `useDragRegion` sur la barre de titre).