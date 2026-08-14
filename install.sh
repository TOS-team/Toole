#!/bin/sh
set -e

# ==========================================
# CONFIGURATION
# ==========================================
REPO="TOS-team/Toole"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"

echo "╔════════════════════════════════════════╗"
echo "║         Installation de Toolé          ║"
echo "║  Transfert P2P chiffré sur réseau local║"
echo "╚════════════════════════════════════════╝"
echo ""

# Vérification des dépendances
check_dependencies() {
  echo "🔍 Vérification des dépendances..."
  MISSING_DEPS=""

  if ! command -v curl >/dev/null 2>&1; then
    MISSING_DEPS="$MISSING_DEPS curl"
  fi

  if ! command -v sudo >/dev/null 2>&1; then
    MISSING_DEPS="$MISSING_DEPS sudo"
  fi

  if [ -n "$MISSING_DEPS" ]; then
    echo "❌ Dépendances manquantes:$MISSING_DEPS"
    echo "   Installez-les d'abord puis relancez le script."
    exit 1
  fi
}

check_dependencies

echo "📡 Recherche de la dernière version de Toolé..."

# On récupère le tag de la dernière version (ex: v2.0.3)
LATEST_TAG=$(curl -s "$API_URL" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
  echo "❌ Impossible de récupérer la dernière version."
  exit 1
fi

echo "🚀 Version trouvée : $LATEST_TAG"
echo ""

# ==========================================
# PRÉPARATION
# ==========================================
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
OS="$(uname -s)"

# 🚀 NOUVELLE MÉTHODE : On extrait l'URL exacte générée par GitHub
download_asset() {
  EXT="$1"
  LOCAL_NAME="$2"

  # Cherche la ligne "browser_download_url" qui se termine par l'extension voulue
  URL=$(curl -s "$API_URL" | grep '"browser_download_url":' | grep "\.${EXT}\"" | cut -d '"' -f 4 | head -n 1)

  if [ -z "$URL" ]; then
    echo "❌ Impossible de trouver un fichier d'installation (.${EXT}) pour la version ${LATEST_TAG}."
    exit 1
  fi

  echo "⬇️  Téléchargement de $(basename "$URL")..."
  curl -fsSL "$URL" -o "${TMP_DIR}/${LOCAL_NAME}"
}

# ==========================================
# DÉTECTION OS ET INSTALLATION
# ==========================================
case "$OS" in
Linux*)
  echo "🐧 Système détecté : Linux"

  # 1. CAS UBUNTU / DEBIAN / MINT
  if command -v dpkg >/dev/null 2>&1; then
    download_asset "deb" "toole.deb"
    echo "📦 Installation du paquet .deb..."
    sudo dpkg -i "${TMP_DIR}/toole.deb" || sudo apt-get install -f -y

  # 2. CAS FEDORA / REDHAT / CENTOS
  elif command -v rpm >/dev/null 2>&1; then
    download_asset "rpm" "toole.rpm"
    echo "📦 Installation du paquet .rpm..."
    sudo rpm -i "${TMP_DIR}/toole.rpm"

  # 3. CAS ARCH LINUX / AUTRES (Extraction manuelle)
  else
    echo "⚙️  Distribution générique/Arch détectée : installation manuelle du binaire..."
    download_asset "deb" "toole.deb"

    cd "$TMP_DIR"
    echo "📦 Extraction de l'archive..."

    # Priorité à bsdtar puis ar
    if command -v bsdtar >/dev/null 2>&1; then
      bsdtar -xf "toole.deb"
      bsdtar -xf data.tar.*
    elif command -v ar >/dev/null 2>&1; then
      ar x "toole.deb"
      tar -xf data.tar.*
    else
      echo "❌ Utilitaires d'extraction (ar ou bsdtar) introuvables."
      exit 1
    fi

    echo "🚚 Déplacement des fichiers dans le système..."
    sudo cp -r usr/* /usr/
    sudo chmod +x /usr/bin/toole 2>/dev/null || true
  fi
  ;;

Darwin*)
  echo "🍎 Système détecté : macOS"
  download_asset "dmg" "toole.dmg"

  echo "📦 Montage du fichier .dmg..."
  MOUNT_DIR=$(mktemp -d)

  if ! hdiutil attach "${TMP_DIR}/toole.dmg" -mountpoint "$MOUNT_DIR" -quiet; then
    echo "❌ Échec du montage du .dmg"
    exit 1
  fi

  echo "🚚 Installation dans /Applications..."
  APP_PATH=$(find "$MOUNT_DIR" -name "*.app" -maxdepth 1 | head -n 1)

  if [ -z "$APP_PATH" ]; then
    echo "❌ Aucune application .app trouvée dans le .dmg"
    hdiutil detach "$MOUNT_DIR" -quiet
    exit 1
  fi

  APP_NAME=$(basename "$APP_PATH")
  sudo cp -R "$APP_PATH" /Applications/

  hdiutil detach "$MOUNT_DIR" -quiet

  echo "🛡️  Application du correctif Gatekeeper..."
  sudo xattr -cr "/Applications/$APP_NAME"

  echo "ℹ️  Note : La mise à jour automatique est désactivée sur macOS"
  ;;

*)
  echo "❌ Système d'exploitation non supporté par ce script : $OS"
  exit 1
  ;;
esac

echo ""
echo "✅ Toolé a été installé avec succès !"
echo ""

# Vérification finale
if command -v toole >/dev/null 2>&1; then
  echo "🚀 Pour lancer Toolé, tapez simplement :"
  echo "   toole"
else
  echo "📱 Toolé est installé. Cherchez-le dans vos applications."
fi
