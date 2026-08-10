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
  if (window.matchMedia && window.matchMedia("(prefers-color-scheme: light)").matches) {
    applySystemTheme("light");
  } else {
    applySystemTheme("dark");
  }
}

app.mount("#app");