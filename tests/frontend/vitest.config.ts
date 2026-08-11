// je configure vitest pour les tests du frontend de Toolé
//
// deux points clés :
//   - environnement jsdom (localStorage + événements DOM)
//   - mocks des imports Tauri via des aliases, pour isoler les stores
//     du vrai pont Tauri (invoke/listen)

import { fileURLToPath } from "url";
import { defineConfig } from "vitest/config";

const mocks = fileURLToPath(new URL("./src/mocks/", import.meta.url));

export default defineConfig({
  test: {
    environment: "jsdom",
    include: ["tests/**/*.test.ts"],
  },
  resolve: {
    alias: {
      // je remplace les imports du pont Tauri par des stubs locaux
      "@tauri-apps/api/core": `${mocks}/tauri-core.ts`,
      "@tauri-apps/api/event": `${mocks}/tauri-event.ts`,
      // je force pinia et vue vers les copies du projet UI : sinon chaque
      // côté (store et test) chargerait sa propre instance et l'état pinia
      // ne serait pas partagé
      pinia: `${fileURLToPath(new URL("../../desktop-app/ui/node_modules/pinia/dist/pinia.mjs", import.meta.url))}`,
      vue: `${fileURLToPath(new URL("../../desktop-app/ui/node_modules/vue/dist/vue.runtime.esm-bundler.js", import.meta.url))}`,
    },
  },
});