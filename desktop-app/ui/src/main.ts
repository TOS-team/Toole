// point d'entrée de l'app : je monte Vue, j'applique le thème et le halo
// persistés, et je m'abonne au thème système de la fenêtre Tauri
import { createApp } from "vue";
import { createPinia } from "pinia";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettingsStore } from "./stores/settings";
import App from "./App.vue";
import "./style.css";

const pinia = createPinia();
const app = createApp(App);

app.use(pinia);

const settings = useSettingsStore(pinia);
settings.applyTheme();
settings.applyGlow();

function applySystemTheme(theme: "dark" | "light") {
  settings.setSystemTheme(theme);
}

try {
  // en environnement Tauri je suis le thème de la fenêtre et ses changements
  const appWindow = getCurrentWindow();
  appWindow
    .theme()
    .then((theme) => {
      if (theme) applySystemTheme(theme);
    })
    .catch(() => applySystemTheme("dark"));
  appWindow.onThemeChanged(({ payload: theme }) => applySystemTheme(theme)).catch(() => {
    applySystemTheme("dark");
  });
} catch {
  // fallback navigateur (dev) : je lis prefers-color-scheme
  if (window.matchMedia && window.matchMedia("(prefers-color-scheme: light)").matches) {
    applySystemTheme("light");
  } else {
    applySystemTheme("dark");
  }
}

app.mount("#app");