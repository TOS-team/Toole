import { defineStore } from "pinia"
import { ref, watch } from "vue"

export type ThemeMode = "auto" | "dark" | "light"

const KEY = "toole.settings"

export const GLOW_COLORS = [
  { id: "red", label: "Rouge", rgb: [255, 0, 51], accent: "#ff0033" },
  { id: "orange", label: "Orange", rgb: [255, 140, 0], accent: "#ff7a00" },
  { id: "yellow", label: "Jaune", rgb: [255, 200, 0], accent: "#d9a100" },
  { id: "green", label: "Vert", rgb: [0, 200, 83], accent: "#00a056" },
  { id: "cyan", label: "Cyan", rgb: [0, 220, 220], accent: "#0099a8" },
  { id: "blue", label: "Bleu", rgb: [70, 130, 255], accent: "#2f6bff" },
  { id: "purple", label: "Violet", rgb: [150, 60, 255], accent: "#7c3aed" },
  { id: "pink", label: "Rose", rgb: [255, 60, 150], accent: "#e6397a" },
] as const

export type GlowColorId = (typeof GLOW_COLORS)[number]["id"]

function loadSettings() {
  try {
    const raw = localStorage.getItem(KEY)
    if (raw) {
      const parsed = JSON.parse(raw)
      let glow = 50
      if (typeof parsed.glow === "number") {
        glow = parsed.glow > 2 ? parsed.glow : parsed.glow * 50
      }
      const glowColor = GLOW_COLORS.some((c) => c.id === parsed.glowColor)
        ? (parsed.glowColor as GlowColorId)
        : "red"
      return {
        theme: parsed.theme === "dark" || parsed.theme === "light" ? parsed.theme : "auto",
        glow,
        glowColor,
      }
    }
  } catch {
    /* silencieux : valeurs par défaut sinon */
  }
  return { theme: "auto" as ThemeMode, glow: 50, glowColor: "red" as GlowColorId }
}

export const useSettingsStore = defineStore("settings", () => {
  const saved = loadSettings()
  const theme = ref<ThemeMode>(saved.theme)
  const glow = ref<number>(saved.glow)
  const glowColor = ref<GlowColorId>(saved.glowColor)
  const systemTheme = ref<"dark" | "light">("dark")

  function applyTheme() {
    const effective: "dark" | "light" =
      theme.value === "auto" ? systemTheme.value : theme.value
    document.documentElement.dataset.theme = effective
  }

  function applyGlow() {
    const color = GLOW_COLORS.find((c) => c.id === glowColor.value)
    const [r, g, b] = color?.rgb ?? [255, 0, 51]
    document.documentElement.style.setProperty("--glow-opacity", String(glow.value / 50))
    document.documentElement.style.setProperty("--glow-color", `${r}, ${g}, ${b}`)
    if (color) {
      document.documentElement.style.setProperty("--color-primary", color.accent)
    }
  }

  function setSystemTheme(t: "dark" | "light") {
    systemTheme.value = t
  }

  watch([theme, systemTheme], applyTheme)
  watch([glow, glowColor], applyGlow)

  watch(
    [theme, glow, glowColor],
    () => {
      try {
        localStorage.setItem(
          KEY,
          JSON.stringify({
            theme: theme.value,
            glow: glow.value,
            glowColor: glowColor.value,
          }),
        )
      } catch {
        /* stockage indisponible : on ignore */
      }
    },
    { deep: true },
  )

  return { theme, glow, glowColor, applyTheme, applyGlow, setSystemTheme }
})