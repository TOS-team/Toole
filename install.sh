#!/bin/bash
# Script d'installation universel pour Toolé
# Transfert de fichiers P2P sur réseau local, chiffré (QUIC/TLS 1.3)

set -e # Arrêter en cas d'erreur

# Configuration
GITHUB_REPO="TOS-team/Toole"
APP_NAME="Toolé"
GITHUB_API="https://api.github.com/repos/${GITHUB_REPO}"

# Couleurs pour le terminal
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Fonctions d'affichage
print_banner() {
  echo -e "${CYAN}"
  echo "╔════════════════════════════════════════╗"
  echo "║         Installation de Toolé          ║"
  echo "║  Transfert P2P chiffré sur réseau local║"
  echo "╚════════════════════════════════════════╝"
  echo -e "${NC}"
}

info() {
  echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

error() {
  echo -e "${RED}[ERROR]${NC} $1"
  exit 1
}

warning() {
  echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Détection de l'OS
detect_os() {
  case "$(uname -s)" in
  Linux*) OS="linux" ;;
  Darwin*) OS="macos" ;;
  CYGWIN* | MINGW* | MSYS*) OS="windows" ;;
  *) error "OS non supporté: $(uname -s)" ;;
  esac
  info "Système d'exploitation détecté: $OS"
}

# Détection de l'architecture
detect_arch() {
  case "$(uname -m)" in
  x86_64 | amd64) ARCH="x64" ;;
  arm64 | aarch64) ARCH="arm64" ;;
  *) error "Architecture non supportée: $(uname -m)" ;;
  esac
  info "Architecture détectée: $ARCH"
}

# Vérification des dépendances
check_dependencies() {
  info "Vérification des dépendances..."

  for cmd in curl; do
    if ! command -v $cmd &>/dev/null; then
      error "$cmd est requis mais n'est pas installé. Veuillez l'installer d'abord."
    fi
  done

  # Vérifier wget comme alternative
  if ! command -v wget &>/dev/null && ! command -v curl &>/dev/null; then
    error "curl ou wget est requis pour l'installation"
  fi
}

# Récupération de la dernière version
get_latest_version() {
  info "Récupération de la dernière version..."

  if command -v curl &>/dev/null; then
    LATEST_VERSION=$(curl -s "${GITHUB_API}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
  else
    LATEST_VERSION=$(wget -qO- "${GITHUB_API}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
  fi

  if [ -z "$LATEST_VERSION" ]; then
    error "Impossible de récupérer la dernière version"
  fi

  info "Dernière version disponible: $LATEST_VERSION"
}

# Installation sur Linux
install_linux() {
  info "Installation sur Linux..."

  # Déterminer le gestionnaire de paquets
  if command -v apt-get &>/dev/null; then
    # Debian/Ubuntu - Installation du .deb
    DEB_FILE="Toolé_${LATEST_VERSION#v}_amd64.deb"
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${LATEST_VERSION}/${DEB_FILE}"

    info "Téléchargement du paquet .deb..."
    TMP_DIR=$(mktemp -d)

    if command -v curl &>/dev/null; then
      curl -L -o "${TMP_DIR}/${DEB_FILE}" "$DOWNLOAD_URL"
    else
      wget -O "${TMP_DIR}/${DEB_FILE}" "$DOWNLOAD_URL"
    fi

    info "Installation du paquet..."
    if [ "$EUID" -ne 0 ]; then
      sudo apt-get install -y "${TMP_DIR}/${DEB_FILE}"
    else
      apt-get install -y "${TMP_DIR}/${DEB_FILE}"
    fi

    rm -rf "$TMP_DIR"

  elif command -v dnf &>/dev/null || command -v yum &>/dev/null; then
    # Fedora/RHEL - Installation du .rpm
    RPM_FILE="Toolé-${LATEST_VERSION#v}-1.x86_64.rpm"
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${LATEST_VERSION}/${RPM_FILE}"

    info "Téléchargement du paquet .rpm..."
    TMP_DIR=$(mktemp -d)

    if command -v curl &>/dev/null; then
      curl -L -o "${TMP_DIR}/${RPM_FILE}" "$DOWNLOAD_URL"
    else
      wget -O "${TMP_DIR}/${RPM_FILE}" "$DOWNLOAD_URL"
    fi

    info "Installation du paquet..."
    if command -v dnf &>/dev/null; then
      if [ "$EUID" -ne 0 ]; then
        sudo dnf install -y "${TMP_DIR}/${RPM_FILE}"
      else
        dnf install -y "${TMP_DIR}/${RPM_FILE}"
      fi
    else
      if [ "$EUID" -ne 0 ]; then
        sudo yum install -y "${TMP_DIR}/${RPM_FILE}"
      else
        yum install -y "${TMP_DIR}/${RPM_FILE}"
      fi
    fi

    rm -rf "$TMP_DIR"

  else
    # Fallback - Téléchargement du binaire directement
    warning "Gestionnaire de paquets non détecté, installation du binaire directement..."

    INSTALL_DIR="/usr/local/bin"
    BINARY_FILE="toole-linux-${ARCH}"
    DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${LATEST_VERSION}/${BINARY_FILE}"

    info "Téléchargement du binaire..."
    if command -v curl &>/dev/null; then
      if [ "$EUID" -ne 0 ]; then
        sudo curl -L -o "${INSTALL_DIR}/toole" "$DOWNLOAD_URL"
      else
        curl -L -o "${INSTALL_DIR}/toole" "$DOWNLOAD_URL"
      fi
    else
      if [ "$EUID" -ne 0 ]; then
        sudo wget -O "${INSTALL_DIR}/toole" "$DOWNLOAD_URL"
      else
        wget -O "${INSTALL_DIR}/toole" "$DOWNLOAD_URL"
      fi
    fi

    # Rendre exécutable
    if [ "$EUID" -ne 0 ]; then
      sudo chmod +x "${INSTALL_DIR}/toole"
    else
      chmod +x "${INSTALL_DIR}/toole"
    fi
  fi
}

# Installation sur macOS
install_macos() {
  info "Installation sur macOS..."

  DMG_FILE="Toolé_${LATEST_VERSION#v}_universal.dmg"
  DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${LATEST_VERSION}/${DMG_FILE}"

  TMP_DIR=$(mktemp -d)

  info "Téléchargement du fichier .dmg..."
  curl -L -o "${TMP_DIR}/${DMG_FILE}" "$DOWNLOAD_URL"

  info "Montage du .dmg..."
  MOUNT_POINT=$(hdiutil attach "${TMP_DIR}/${DMG_FILE}" | grep Volumes | awk '{print $3}')

  if [ -z "$MOUNT_POINT" ]; then
    error "Impossible de monter le .dmg"
  fi

  info "Installation dans /Applications..."
  cp -R "${MOUNT_POINT}/Toolé.app" /Applications/

  info "Démontage du .dmg..."
  hdiutil detach "$MOUNT_POINT"

  rm -rf "$TMP_DIR"

  warning "Note: Build non notarié, Gatekeeper peut refuser l'ouverture."
  warning "Si nécessaire, exécutez: xattr -cr /Applications/Toolé.app"

  info "La mise à jour automatique est désactivée sur macOS."
}

# Installation sur Windows (via WSL ou Git Bash)
install_windows() {
  info "Installation sur Windows..."

  EXE_FILE="Toolé_${LATEST_VERSION#v}_x64-setup.exe"
  DOWNLOAD_URL="https://github.com/${GITHUB_REPO}/releases/download/${LATEST_VERSION}/${EXE_FILE}"

  info "Téléchargement de l'installateur Windows..."
  TMP_DIR=$(mktemp -d)

  curl -L -o "${TMP_DIR}/${EXE_FILE}" "$DOWNLOAD_URL"

  info "Lancement de l'installateur..."
  warning "L'installateur Windows va s'ouvrir dans une nouvelle fenêtre."
  warning "Suivez les instructions à l'écran pour terminer l'installation."

  if command -v cmd.exe &>/dev/null; then
    cmd.exe /c "start ${TMP_DIR}/${EXE_FILE}"
  elif command -v powershell.exe &>/dev/null; then
    powershell.exe -Command "Start-Process '${TMP_DIR}/${EXE_FILE}'"
  else
    error "Impossible de lancer l'installateur Windows"
  fi

  rm -rf "$TMP_DIR"
}

# Vérification de l'installation
verify_installation() {
  info "Vérification de l'installation..."

  if command -v toole &>/dev/null; then
    success "Toolé est installé et disponible dans le PATH"
    echo ""
    echo -e "${GREEN}Pour lancer Toolé, tapez simplement:${NC}"
    echo -e "${CYAN}  toole${NC}"
    echo ""
  else
    case "$OS" in
    linux)
      if command -v toole &>/dev/null; then
        success "Toolé est installé"
      else
        warning "Toolé est installé mais n'est pas dans le PATH"
        info "Vous pouvez le trouver avec: which toole ou dpkg -L toole"
      fi
      ;;
    macos)
      success "Toolé est installé dans /Applications"
      echo -e "${GREEN}Pour lancer Toolé:${NC}"
      echo -e "${CYAN}  open /Applications/Toolé.app${NC}"
      ;;
    windows)
      success "Toolé est installé sur Windows"
      echo -e "${GREEN}Pour lancer Toolé, cherchez-le dans le menu Démarrer${NC}"
      ;;
    esac
  fi
}

# Fonction principale
main() {
  print_banner
  echo ""

  # Détection du système
  detect_os
  detect_arch

  echo ""

  # Vérifier les dépendances
  check_dependencies

  # Récupérer la dernière version
  get_latest_version

  echo ""

  # Installer selon l'OS
  case "$OS" in
  linux)
    install_linux
    ;;
  macos)
    install_macos
    ;;
  windows)
    install_windows
    ;;
  esac

  echo ""

  # Vérifier l'installation
  verify_installation

  echo ""
  success "Installation terminée ! 🎉"
  echo ""
  echo -e "${CYAN}Toolé - Transfert de fichiers P2P sur réseau local, chiffré (QUIC/TLS 1.3)${NC}"
}

# Exécution
main "$@"
