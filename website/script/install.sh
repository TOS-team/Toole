#!/bin/bash
set -euo pipefail

REPO="TOS-team/Toole"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
APPIMAGE_DIR="${APPIMAGE_DIR:-$HOME/.local/bin}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[Toolé]${NC} $1"; }
warn() { echo -e "${YELLOW}[Toolé]${NC} $1"; }
err() { echo -e "${RED}[Toolé]${NC} $1" >&2; }

detect_os() {
  if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "linux"
  elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "macos"
  elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" ]]; then
    echo "windows"
  elif command -v wsl.exe &>/dev/null; then
    echo "wsl"
  else echo "unknown"; fi
}

detect_pkg_manager() {
  if command -v apt &>/dev/null; then
    echo "apt"
  elif command -v pacman &>/dev/null; then
    echo "pacman"
  elif command -v dnf &>/dev/null; then
    echo "dnf"
  elif command -v yum &>/dev/null; then
    echo "yum"
  elif command -v zypper &>/dev/null; then
    echo "zypper"
  else echo "none"; fi
}

get_latest_version() {
  curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" |
    grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/'
}

install_apt() {
  info "Détection APT (Debian/Ubuntu)..."
  if ! command -v curl &>/dev/null || ! command -v gpg &>/dev/null; then
    warn "Installation des dépendances..."
    sudo apt-get update && sudo apt-get install -y curl gnupg
  fi
  info "Ajout de la clé GPG..."
  sudo mkdir -p /usr/share/keyrings
  curl -fsSL "https://$REPO.github.io/Toole/apt/pubkey.gpg" |
    sudo gpg --dearmor -o /usr/share/keyrings/toole.gpg 2>/dev/null || true
  info "Ajout du dépôt APT..."
  echo "deb [signed-by=/usr/share/keyrings/toole.gpg] https://TOS-team.github.io/Toole/apt stable main" |
    sudo tee /etc/apt/sources.list.d/toole.list >/dev/null
  sudo apt-get update
  info "Installation de Toolé..."
  sudo apt-get install -y toole
  info "✅ Toolé installé ! Lance avec : toole"
}

install_pacman() {
  info "Détection Arch Linux / Manjaro..."
  if command -v yay &>/dev/null; then
    yay -S --noconfirm toole-bin
  elif command -v paru &>/dev/null; then
    paru -S --noconfirm toole-bin
  else
    warn "yay/paru non trouvé. Fallback AppImage..."
    install_appimage
    return
  fi
  info "✅ Toolé installé !"
}

install_rpm() {
  info "Détection RPM-based..."
  local PKG_MGR=$(detect_pkg_manager)
  local LATEST=$(get_latest_version)
  local URL="https://github.com/$REPO/releases/download/v${LATEST}/toole_desktop_${LATEST}_amd64.rpm"
  info "Téléchargement du RPM..."
  curl -L -o /tmp/toole.rpm "$URL"
  if [ "$PKG_MGR" == "dnf" ]; then
    sudo dnf install -y /tmp/toole.rpm
  elif [ "$PKG_MGR" == "yum" ]; then
    sudo yum install -y /tmp/toole.rpm
  elif [ "$PKG_MGR" == "zypper" ]; then sudo zypper install -y /tmp/toole.rpm; fi
  rm -f /tmp/toole.rpm
  info "✅ Toolé installé !"
}

install_appimage() {
  info "Installation via AppImage (fallback)..."
  local LATEST=$(get_latest_version)
  local URL="https://github.com/$REPO/releases/download/v${LATEST}/toole_desktop_${LATEST}_amd64.AppImage"
  local DEST="$APPIMAGE_DIR/toole"
  mkdir -p "$APPIMAGE_DIR"
  curl -L -o "$DEST" "$URL"
  chmod +x "$DEST"
  mkdir -p "$HOME/.local/share/applications"
  cat >"$HOME/.local/share/applications/toole.desktop" <<EOF
[Desktop Entry]
Name=Toolé
Comment=Transfert P2P local
Exec=$DEST
Icon=$DEST
Type=Application
Categories=Network;FileTransfer;
Terminal=false
EOF
  if [[ ":$PATH:" != *":$APPIMAGE_DIR:"* ]]; then
    echo 'export PATH="$HOME/.local/bin:$PATH"' >>"$HOME/.bashrc"
    warn "Ajoute ~/.local/bin à ton PATH, puis recharge : source ~/.bashrc"
  fi
  info "✅ Toolé installé ! Lance avec : $DEST"
}

install_macos() {
  info "Détection macOS..."
  if ! command -v brew &>/dev/null; then
    err "Homebrew n'est pas installé."
    echo '  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"'
    exit 1
  fi
  info "Ajout du tap TOS-team/toole..."
  brew tap TOS-team/toole
  info "Installation de Toolé..."
  brew install --cask toole
  info "✅ Toolé installé ! Lance depuis /Applications ou Spotlight (Cmd+Space → Toolé)"
  info "Si Gatekeeper bloque : sudo xattr -cr /Applications/Toolé.app"
}

install_wsl() {
  info "Détection WSL..."
  warn "Toolé est une app graphique. Sous WSL, utilise la version Windows native."
  info "Télécharge le .msi ici : https://github.com/$REPO/releases/latest"
  exit 0
}

main() {
  info "Installateur Toolé"
  local OS=$(detect_os)
  case "$OS" in
  linux)
    case "$(detect_pkg_manager)" in
    apt) install_apt ;;
    pacman) install_pacman ;;
    dnf | yum | zypper) install_rpm ;;
    *) install_appimage ;;
    esac
    ;;
  macos) install_macos ;;
  windows | wsl) install_wsl ;;
  *)
    err "OS non reconnu : $OSTYPE"
    info "Télécharge manuellement : https://github.com/$REPO/releases/latest"
    exit 1
    ;;
  esac
}

main "$@"
