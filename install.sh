#!/bin/sh
set -e

# ==========================================
# CONFIGURATION
# ==========================================
REPO="TOS-team/Toole"

echo "📡 Recherche de la dernière version de Toolé..."
LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

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

# ==========================================
# DÉTECTION OS ET INSTALLATION
# ==========================================
case "$OS" in
Linux*)
  echo "🐧 Système détecté : $OS"

  # 1. CAS UBUNTU / DEBIAN / MINT / POP!_OS
  if command -v dpkg >/dev/null 2>&1; then
    FILE_NAME="Toolé_${VERSION}_amd64.deb"
    URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${FILE_NAME}"

    echo "⬇️ Téléchargement de $FILE_NAME..."
    curl -sSL "$URL" -o "${TMP_DIR}/${FILE_NAME}"

    echo "📦 Installation du paquet .deb..."
    sudo dpkg -i "${TMP_DIR}/${FILE_NAME}" || sudo apt-get install -f -y

  # 2. CAS FEDORA / REDHAT / CENTOS / ROCKY
  elif command -v rpm >/dev/null 2>&1; then
    FILE_NAME="Toolé-${VERSION}-1.x86_64.rpm"
    URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${FILE_NAME}"

    echo "⬇️ Téléchargement de $FILE_NAME..."
    curl -sSL "$URL" -o "${TMP_DIR}/${FILE_NAME}"

    echo "📦 Installation du paquet .rpm..."
    sudo rpm -i "${TMP_DIR}/${FILE_NAME}"

  # 3. CAS ARCH LINUX / MANJARO / AUTRES (Extraction manuelle du .deb)
  else
    echo "⚙️ Distribution générique/Arch détectée : installation manuelle du binaire..."
    FILE_NAME="Toolé_${VERSION}_amd64.deb"
    URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${FILE_NAME}"

    echo "⬇️ Téléchargement de $FILE_NAME..."
    curl -sSL "$URL" -o "${TMP_DIR}/${FILE_NAME}"

    cd "$TMP_DIR"

    # Extraction du .deb (fonctionne avec bsdtar sur Arch, ou ar sur d'autres)
    echo "📦 Extraction de l'archive..."
    if command -v bsdtar >/dev/null 2>&1; then
      bsdtar -xf "$FILE_NAME"
    else
      ar x "$FILE_NAME"
    fi

    # Extraction de l'archive interne data.tar.* (contient les fichiers système)
    tar -xf data.tar.*

    # Déplacement des fichiers extraits (ex: le dossier usr/bin et usr/share) vers la racine du système
    echo "🚚 Déplacement des fichiers dans le système..."
    sudo cp -r usr/* /usr/

    # Sécurité supplémentaire pour garantir que le binaire est exécutable
    sudo chmod +x /usr/bin/toole 2>/dev/null || true
  fi
  ;;

Darwin*)
  echo "🍎 Système détecté : macOS"
  FILE_NAME="Toolé_${VERSION}_universal.dmg"
  URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${FILE_NAME}"

  echo "⬇️ Téléchargement de $FILE_NAME..."
  curl -sSL "$URL" -o "${TMP_DIR}/${FILE_NAME}"

  echo "📦 Montage du fichier .dmg..."
  MOUNT_DIR=$(mktemp -d)
  hdiutil attach "${TMP_DIR}/${FILE_NAME}" -mountpoint "$MOUNT_DIR" -quiet

  echo "🚚 Installation dans /Applications..."
  sudo cp -R "$MOUNT_DIR/"*.app /Applications/

  # Trouver le nom exact de l'application (ex: Toolé.app)
  APP_NAME=$(ls "$MOUNT_DIR" | grep '\.app$' | head -n 1)

  # Démonter le .dmg proprement
  hdiutil detach "$MOUNT_DIR" -quiet

  if [ -n "$APP_NAME" ]; then
    echo "🛡️ Application du correctif Gatekeeper (autorisation Apple Silicon/Intel)..."
    sudo xattr -cr "/Applications/$APP_NAME"
  fi
  ;;

*)
  echo "❌ Système d'exploitation non supporté par ce script : $OS"
  echo "Pour Windows, utilisez le script PowerShell (install.ps1)."
  exit 1
  ;;
esac

echo "✅ Toolé a été installé avec succès !"
echo "execter la commande toole"
