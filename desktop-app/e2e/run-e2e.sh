#!/usr/bin/env bash
# lance le test e2e WebDriver de Toolé : de la vraie UI (webview WebKitGTK)
# au backend Rust via le pont IPC Tauri réel.
#
# Prérequis :
#   - tauri-driver installé (cargo install tauri-driver --locked)
#   - WebKitWebDriver présent sur la machine (paquet webkit2gtk-driver)
#   - un serveur X/Wayland disponible (DISPLAY/WAYLAND_DISPLAY défini)
#
# L'app est compilée en debug (= mode dev Tauri, il charge la UI depuis le
# serveur Vite), je lance donc Vite puis wdio, et je nettoie à la sortie.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

# 1. l'app doit être compilée (reconstruit seulement si les sources changent)
(cd "$ROOT" && cargo build -p app)

# 2. je sers la UI avec Vite (port 1420) : le binaire debug s'y connecte
(cd "$ROOT/desktop-app/ui" && npm run dev) >/tmp/toole-e2e-vite.log 2>&1 &
VITE_PID=$!
trap 'kill "$VITE_PID" 2>/dev/null || true' EXIT

# j'attends que Vite réponde avant de lancer le test (pas de délai fixe)
for _ in $(seq 1 60); do
  if curl -sf http://localhost:1420 >/dev/null 2>&1; then break; fi
  sleep 1
done

# 3. le test WebDriver (tauri-driver est lancé et arrêté par wdio.conf.ts)
cd "$ROOT/desktop-app/e2e"
npx wdio run wdio.conf.ts "$@"