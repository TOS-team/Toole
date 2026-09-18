#!/usr/bin/env bash

# ==============================================================================
# Script d'installation — Toolé
# Transfert de fichiers P2P chiffré (Rust + Tauri v2 + QUIC/TLS 1.3)
# ==============================================================================
GITHUB_REPO="TOS-team/Toole"
APP_NAME="Toole"               # ⚠️ À vérifier : nom exact du binaire/.app produit par la CI Tauri
VERSION="${1:-latest}"         # Usage : ./install-toole.sh v1.2.0  (sinon = dernière release)
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}=== Installation de $APP_NAME ===${NC}"

if ! command -v curl &> /dev/null; then
    echo -e "${RED}Erreur : 'curl' est requis pour ce script.${NC}"
    exit 1
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS_ID="${ID:-unknown}"
else
    OS_ID="unknown"
fi

# --- Suggestion des canaux déjà publiés pour Toolé, avant le fallback GitHub ---
if [[ "$OS" == "Darwin" ]] && command -v brew &> /dev/null; then
    echo -e "${YELLOW}Homebrew détecté. Installation recommandée :${NC}"
    echo -e "  brew tap tos-team/toole && brew install toole"
    echo -e "${YELLOW}(Le script continue avec le téléchargement direct du .dmg si tu préfères.)${NC}"
fi

if [[ "$OS_ID" == "arch" || "$OS_ID" == "manjaro" || "$OS_ID" == "endeavouros" ]] && command -v yay &> /dev/null; then
    echo -e "${YELLOW}AUR détecté. Installation recommandée :${NC}"
    echo -e "  yay -S toole"
    echo -e "${YELLOW}(Le script continue avec le téléchargement direct du .deb/.AppImage si tu préfères.)${NC}"
fi

# --- Récupération de la release depuis GitHub ---
if [[ "$VERSION" == "latest" ]]; then
    RELEASE_API="https://api.github.com/repos/$GITHUB_REPO/releases/latest"
else
    RELEASE_API="https://api.github.com/repos/$GITHUB_REPO/releases/tags/$VERSION"
fi

echo -e "Recherche de la release ($VERSION) sur GitHub..."
RELEASE_JSON="$(curl -fsSL "$RELEASE_API")" || {
    echo -e "${RED}Erreur : impossible de récupérer la release ($VERSION) pour $GITHUB_REPO.${NC}"
    exit 1
}
DOWNLOAD_URLS="$(echo "$RELEASE_JSON" | grep "browser_download_url")"

if [ -z "$DOWNLOAD_URLS" ]; then
    echo -e "${RED}Erreur : aucun asset trouvé pour cette release.${NC}"
    exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
cd "$TMP_DIR"

# Vérifie le SHA-256 du fichier téléchargé, si un fichier de somme est publié
# dans la release (cohérent avec la vérification SHA-256 déjà utilisée dans
# le protocole de transfert de Toolé).
verify_checksum() {
    local file_name="$1"
    local checksum_url
    checksum_url=$(echo "$DOWNLOAD_URLS" | grep -i "${file_name}.sha256\"" | head -n 1 | cut -d '"' -f 4)
    if [ -n "$checksum_url" ]; then
        curl -fsSL -o "${file_name}.sha256" "$checksum_url"
        echo -e "Vérification de l'intégrité (SHA-256)..."
        local expected
        expected=$(awk '{print $1}' "${file_name}.sha256")
        if command -v sha256sum &> /dev/null; then
            echo "$expected  $file_name" | sha256sum -c - \
                && echo -e "${GREEN}Intégrité vérifiée.${NC}" \
                || { echo -e "${RED}Échec de la vérification SHA-256.${NC}"; exit 1; }
        elif command -v shasum &> /dev/null; then
            echo "$expected  $file_name" | shasum -a 256 -c - \
                && echo -e "${GREEN}Intégrité vérifiée.${NC}" \
                || { echo -e "${RED}Échec de la vérification SHA-256.${NC}"; exit 1; }
        fi
    fi
}

case "$OS" in
    Linux*)
        if [[ "$ARCH" != "x86_64" && "$ARCH" != "aarch64" ]]; then
            echo -e "${RED}Attention : les binaires Linux sont fournis pour x86_64/aarch64. Ton architecture est $ARCH.${NC}"
        fi

        if [[ "$OS_ID" == "ubuntu" || "$OS_ID" == "debian" || "$OS_ID" == "linuxmint" || "$OS_ID" == "pop" ]]; then
            echo -e "${BLUE}Système Debian/Ubuntu détecté.${NC}"
            ASSET_URL=$(echo "$DOWNLOAD_URLS" | grep -i "\.deb\"" | head -n 1 | cut -d '"' -f 4)
            FILE_NAME="${APP_NAME}.deb"

            echo -e "Téléchargement du paquet .deb..."
            curl -fL -o "$FILE_NAME" "$ASSET_URL"
            verify_checksum "$FILE_NAME"

            echo -e "Installation (les droits sudo seront demandés)..."
            sudo apt-get install -y "./$FILE_NAME"

        elif [[ "$OS_ID" == "fedora" || "$OS_ID" == "rhel" || "$OS_ID" == "centos" ]]; then
            echo -e "${BLUE}Système Fedora/RHEL détecté.${NC}"
            ASSET_URL=$(echo "$DOWNLOAD_URLS" | grep -i "\.rpm\"" | head -n 1 | cut -d '"' -f 4)
            FILE_NAME="${APP_NAME}.rpm"

            echo -e "Téléchargement du paquet .rpm..."
            curl -fL -o "$FILE_NAME" "$ASSET_URL"
            verify_checksum "$FILE_NAME"

            echo -e "Installation (les droits sudo seront demandés)..."
            sudo dnf install -y "./$FILE_NAME"

        else
            echo -e "${BLUE}Système Linux générique détecté (utilisation de l'AppImage).${NC}"
            ASSET_URL=$(echo "$DOWNLOAD_URLS" | grep -i "\.AppImage\"" | head -n 1 | cut -d '"' -f 4)
            FILE_NAME="${APP_NAME}.AppImage"

            echo -e "Téléchargement de l'AppImage..."
            curl -fL -o "$FILE_NAME" "$ASSET_URL"
            verify_checksum "$FILE_NAME"

            echo -e "Configuration des permissions et déplacement..."
            chmod +x "$FILE_NAME"
            mkdir -p ~/.local/bin
            mv "$FILE_NAME" ~/.local/bin/"$APP_NAME"
            echo -e "${GREEN}L'AppImage a été installée dans ~/.local/bin/${APP_NAME}${NC}"
            echo -e "Assure-toi que ~/.local/bin est dans ton \$PATH."
        fi
        ;;

    Darwin*)
        echo -e "${BLUE}Système macOS détecté.${NC}"
        ASSET_URL=$(echo "$DOWNLOAD_URLS" | grep -i "\.dmg\"" | head -n 1 | cut -d '"' -f 4)
        FILE_NAME="${APP_NAME}.dmg"

        echo -e "Téléchargement du .dmg..."
        curl -fL -o "$FILE_NAME" "$ASSET_URL"
        verify_checksum "$FILE_NAME"

        echo -e "Montage de l'image disque..."
        VOLUME=$(hdiutil attach -nobrowse "$FILE_NAME" | grep Volumes | awk -F '\t' '{print $NF}')

        echo -e "Copie de l'application dans /Applications (les droits admin peuvent être demandés)..."
        sudo cp -R "$VOLUME/$APP_NAME.app" /Applications/

        echo -e "Démontage de l'image..."
        hdiutil detach "$VOLUME" -quiet
        ;;

    MINGW*|MSYS*|CYGWIN*)
        echo -e "${YELLOW}Windows détecté (environnement $OS).${NC}"
        ASSET_URL=$(echo "$DOWNLOAD_URLS" | grep -i "\.msi\"" | head -n 1 | cut -d '"' -f 4)
        if [ -z "$ASSET_URL" ]; then
            ASSET_URL=$(echo "$DOWNLOAD_URLS" | grep -i "\.exe\"" | head -n 1 | cut -d '"' -f 4)
        fi
        FILE_NAME="${APP_NAME}-setup.msi"

        echo -e "Téléchargement de l'installeur Windows..."
        curl -fL -o "$FILE_NAME" "$ASSET_URL"
        verify_checksum "$FILE_NAME"

        echo -e "${YELLOW}Lancement de l'installeur (une fenêtre va s'ouvrir)...${NC}"
        cmd.exe /c start "" "$(pwd -W 2>/dev/null || pwd)/$FILE_NAME"
        ;;

    *)
        echo -e "${RED}Système d'exploitation non supporté : $OS${NC}"
        exit 1
        ;;
esac

echo -e "${GREEN}=== Installation terminée avec succès ! ===${NC}"
echo -e "Tu peux maintenant lancer $APP_NAME."
