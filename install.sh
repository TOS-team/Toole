#!/bin/sh
set -e

# ==========================================
# CONFIGURATION
# ==========================================
REPO="TOS-team/Toole"

echo "📡 Recherche de la dernière version de Toolé..."
# Ajout de -f pour éviter les faux téléchargements
LATEST_TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
  echo "❌ Impossible de récupérer la dernière version."
  exit 1
fi

# Extraction du numéro de version sans le 'v' (ex: 2.0.3)
VERSION=$(echo "$LATEST_TAG" | sed 's/^v//')
echo "🚀 Version trouvée : $LATEST_TAG"

# ==========================================
# PRÉPARATION
# ==========================================
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT
OS="$(uname -s)"

# Fonction pour télécharger et gérer l'encodage de l'accent "é"
download_asset() {
  FILE_NAME="$1"
  # Remplace "é" par "%C3%A9" pour l'URL GitHub
  URL_NAME=$(echo "$FILE_NAME" | sed 's/é/%C3%A9/g')
  URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${URL_NAME}"

  echo "⬇️ Téléchargement de $FILE_NAME..."
  # -f force l'arrêt si l'URL renvoie une erreur 404
  curl -fsSL "$URL" -o "${TMP_DIR}/${FILE_NAME}"
}

# ==========================================
# DÉTECTION OS ET INSTALLATION
# ==========================================
case "$OS" in
Linux*)
  echo "🐧 Système détecté : Linux"

  # 1. CAS UBUNTU / DEBIAN / MINT / POP!_OS
  if command -v dpkg >/dev/null 2>&1; then
    download_asset "Toolé_${VERSION}_amd64.deb"
    echo "📦 Installation du paquet .deb..."
    sudo dpkg -i "${TMP_DIR}/Toolé_${VERSION}_amd64.deb" || sudo apt-get install -f -y

  # 2. CAS FEDORA / REDHAT / CENTOS / ROCKY
  elif command -v rpm >/dev/null 2>&1; then
    download_asset "Toolé-${VERSION}-1.x86_64.rpm"
    echo "📦 Installation du paquet .rpm..."
    sudo rpm -i "${TMP_DIR}/Toolé-${VERSION}-1.x86_64.rpm"

  # 3. CAS ARCH LINUX / MANJARO / AUTRES (Extraction manuelle du .deb)
  else
    echo "⚙️ Distribution générique/Arch détectée : installation manuelle du binaire..."
    download_asset "Toolé_${VERSION}_amd64.deb"

    cd "$TMP_DIR"
    echo "📦 Extraction de l'archive..."

    # Priorité à 'ar' (l'outil standard pour les .deb) puis 'bsdtar'
    if command -v ar >/dev/null 2>&1; then
      ar x "Toolé_${VERSION}_amd64.deb"
    elif command -v bsdtar >/dev/null 2>&1; then
      bsdtar -xf "Toolé_${VERSION}_amd64.deb"
    else
      echo "❌ Utilitaires d'extraction (ar ou bsdtar) introuvables sur ce système."
      exit 1
    fi

    # Extraction de l'archive interne data.tar.* (contient les fichiers)
    tar -xf data.tar.*

    echo "🚚 Déplacement des fichiers dans le système..."
    sudo cp -r usr/* /usr/
    sudo chmod +x /usr/bin/toole 2>/dev/null || true
  fi
  ;;

Darwin*)
  echo "🍎 Système détecté : macOS"
  download_asset "Toolé_${VERSION}_universal.dmg"

  echo "📦 Montage du fichier .dmg..."
  MOUNT_DIR=$(mktemp -d)
  hdiutil attach "${TMP_DIR}/Toolé_${VERSION}_universal.dmg" -mountpoint "$MOUNT_DIR" -quiet

  echo "🚚 Installation dans /Applications..."
  sudo cp -R "$MOUNT_DIR/"*.app /Applications/

  # Trouver le nom exact de l'application
  APP_NAME=$(ls "$MOUNT_DIR" | grep '\.app$' | head -n 1)
  hdiutil detach "$MOUNT_DIR" -quiet

  if [ -n "$APP_NAME" ]; then
    echo "🛡️ Application du correctif Gatekeeper (autorisation Apple Silicon/Intel)..."
    sudo xattr -cr "/Applications/$APP_NAME"
  fi
  ;;

*)
  echo "❌ Système d'exploitation non supporté par ce script : $OS"
  exit 1
  ;;
esac

echo "✅ Toolé a été installé avec succès !"
