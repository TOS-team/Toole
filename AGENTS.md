# Session

Dernière session : extraction de `formatSize` dans `src/utils.ts`, refactor AboutModal (emit + ref au lieu de `document.getElementById`), 40 commits individuels fichier par fichier.

Structure : workspace Cargo avec `core/` (lib pure) + `desktop-app/src-tauri/` (app Tauri). Frontend Vue 3 + Pinia + Tailwind v4 + Vite.

Commandes utiles :
- `cargo tauri dev` — lancer l'app en dev
- `npm run build` — builder le frontend (vérification TS)
- `cargo check -p app` — vérifier Rust

Conventions :
- Commentaires Rust en français, lowercase, avec `je`
- Pas de commentaires dans le JS/TS
- CSS avec commentaires `/* SECTION */` en majuscules
