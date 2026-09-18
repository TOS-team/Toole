#!/usr/bin/env bash

# ==============================================================================
# Configuration : Remplace ces variables par tes informations !
# ==============================================================================
GITHUB_REPO="TonUtilisateur/TonDepot" # ex: "tauri-apps/tauri"
APP_NAME="TonApplication"             # Le nom exact de ton .app ou binaire
# ==============================================================================

set -e

# Couleurs pour les logs
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Installation de $APP_NAME ===${NC}"

# 1. Vérification des dépendances (curl)
if ! command -v curl &> /dev/null; then
    echo -e "${RED}Erreur : 'curl' est requis pour ce script.${NC}"
    exit 1
fi

# 2. Récupération de la dernière release depuis GitHub
echo -e "Recherche de la dernière version sur GitHub..."
LATEST_RELEASE_API="https://api.github.com/repos/$GITHUB_REPO/releases/latest"
# On utilise grep pour ne pas dépendre de 'jq' qui n'est pas toujours installé
DOWNLOAD_URLS=$(curl -s $LATEST_RELEASE_API | grep "browser_download_url")

if [ -z "$DOWNLOAD_URLS" ]; then
    echo -e "${RED}Erreur : Impossible de trouver la dernière release pour $GITHUB_REPO.${NC}"
    exit 1
fi

# 3. Détection de l'OS
OS="$(uname -s)"
ARCH="$(uname -m)"

TMP_DIR=$(mktemp -d)
cd $TMP_DIR

case "$OS" in
    Linux*)
        if [ "$ARCH" != "x86_64" ]; then
            echo -e "${RED}Attention : Les binaires Linux fournis sont pour x86_64. Ton architecture est $ARCH.${NC}"
        fi

        # Détection de la distribution
        if [ -f /etc/os-release ]; then
            . /etc/os-release
            OS_ID=$ID
        else
            OS_ID="unknown"
        fi

        if [[ "$OS_ID" == "ubuntu" || "$OS_ID" == "debian" || "$OS_ID" == "linuxmint" || "$OS_ID" == "pop" ]]; then
            echo -e "${BLUE}Système Debian/Ubuntu détecté.${NC}"
            ASSET_URL=$(echo "$DOWNLOAD_URLS" | grep -i "\.deb\"" | head -n 1 | cut -d '"' -f 4)
            FILE_NAME="${APP_NAME}.deb"
            
            echo -e "Téléchargement du paquet .deb..."
            curl -L -o "$FILE_NAME" "$ASSET_URL"
            
            echo -e "Installation (les droits sudo seront demandés)..."
            sudo apt-get install -y "./$FILE_NAME"
            
        elif [[ "$OS_ID" == "fedora" || "$OS_ID" == "rhel" || "$OS_ID" == "centos" ]]; then
            echo -e "${BLUE}Système Fedora/RHEL détecté.${NC}"
            ASSET_URL=$(echo "$DOWNLOAD_URLS" | grep -i "\.rpm\"" | head -n 1 | cut -d '"' -f 4)
            FILE_NAME="${APP_NAME}.rpm"
            
            echo -e "Téléchargement du paquet .rpm..."
            curl -L -o "$FILE_NAME" "$ASSET_URL"
            
            echo -e "Installation (les droits sudo seront demandés)..."
            sudo dnf install -y "./$FILE_NAME"
            
        else
            echo -e "${BLUE}Système Linux générique détecté (Utilisation de l'AppImage).${NC}"
            ASSET_URL=$(echo "$DOWNLOAD_URLS" | grep -i "\.AppImage\"" | head -n 1 | cut -d '"' -f 4)
            FILE_NAME="${APP_NAME}.AppImage"
            
            echo -e "Téléchargement de l'AppImage..."
            curl -L -o "$FILE_NAME" "$ASSET_URL"
            
            echo -e "Configuration des permissions et déplacement..."
            chmod +x "$FILE_NAME"
            mkdir -p ~/.local/bin
            mv "$FILE_NAME" ~/.local/bin/$APP_NAME
            echo -e "${GREEN}L'AppImage a été installée dans ~/.local/bin/${APP_NAME}${NC}"
            echo -e "Assure-toi que ~/.local/bin est dans ton \$PATH."
        fi
        ;;
        
    Darwin*)
        echo -e "${BLUE}Système macOS détecté.${NC}"
        # La CI build du 'universal-apple-darwin', un seul dmg suffit
        ASSET_URL=$(echo "$DOWNLOAD_URLS" | grep -i "\.dmg\"" | head -n 1 | cut -d '"' -f 4)
        FILE_NAME="${APP_NAME}.dmg"
        
        echo -e "Téléchargement du .dmg..."
        curl -L -o "$FILE_NAME" "$ASSET_URL"
        
        echo -e "Montage de l'image disque..."
        VOLUME=$(hdiutil attach -nobrowse "$FILE_NAME" | grep Volumes | awk -F '\t' '{print $3}')
        
        echo -e "Copie de l'application dans /Applications (les droits admin peuvent être demandés)..."
        # On utilise sudo au cas où l'utilisateur n'est pas admin de la machine
        sudo cp -R "$VOLUME/$APP_NAME.app" /Applications/
        
        echo -e "Démontage de l'image..."
        hdiutil detach "$VOLUME" -quiet
        ;;
        
    *)
        echo -e "${RED}Système d'exploitation non supporté : $OS${NC}"
        exit 1
        ;;
esac

# Nettoyage
cd ~
rm -rf "$TMP_DIR"

echo -e "${GREEN}=== Installation terminée avec succès ! ===${NC}"
echo -e "Tu peux maintenant lancer $APP_NAME."
