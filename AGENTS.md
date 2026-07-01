# Session

Dernière session : extraction de `formatSize` dans `src/utils.ts`, refactor AboutModal (emit + ref au lieu de `document.getElementById`), 40 commits individuels fichier par fichier.

Structure : workspace Cargo avec `core/` (lib pure) + `desktop-app/src-tauri/` (app Tauri). Frontend Vue 3 + Pinia + Tailwind v4 + Vite.

Commandes utiles :
- `cargo tauri dev` — lancer l'app en dev (depuis `desktop-app/`)
- `npm run build` — builder le frontend (depuis `desktop-app/ui/`)
- `cargo check -p app` — vérifier Rust
- `cargo run -p toole_core` — tester la découverte UDP standalone

Conventions :
- Commentaires Rust en français, lowercase, avec `je`
- Pas de commentaires dans le JS/TS
- CSS avec commentaires `/* SECTION */` en majuscules
