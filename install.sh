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
    echo "🔍 Vérification des dépendances de base..."
    MISSING_DEPS=""
    
    if ! command -v curl >/dev/null 2>&1; then
        MISSING_DEPS="$MISSING_DEPS curl"
    fi
    
    if ! command -v sudo >/dev/null 2>&1; then
        MISSING_DEPS="$MISSING_DEPS sudo"
    fi
    
    if [ -n "$MISSING_DEPS" ]; then
        echo "❌ Dépendances manquantes:$MISSING_DEPS"
        echo "   Installez-les d'abord :"
        echo "   Debian/Ubuntu : sudo apt install curl sudo"
        echo "   Fedora/RHEL  : sudo dnf install curl sudo"
        exit 1
    fi
}

check_dependencies

echo "📡 Recherche de la dernière version de Toolé..."

# Récupération de la dernière version
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

# Fonction de téléchargement améliorée
download_asset() {
    PATTERN="$1"  # Pattern de recherche (ex: "amd64.deb")
    LOCAL_NAME="$2"
    
    echo "🔍 Recherche de l'asset avec le pattern: $PATTERN"
    
    # Récupérer toutes les URLs de la release
    ASSETS_JSON=$(curl -s "$API_URL" | grep '"browser_download_url":')
    
    # Trouver l'URL qui correspond au pattern
    URL=$(echo "$ASSETS_JSON" | grep "$PATTERN" | grep -v '\.sig"' | cut -d '"' -f 4 | head -n 1)
    
    if [ -z "$URL" ]; then
        echo "❌ Asset non trouvé avec le pattern: $PATTERN"
        echo "   Assets disponibles :"
        echo "$ASSETS_JSON" | cut -d '"' -f 4 | while read -r asset_url; do
            echo "   - $(basename "$asset_url")"
        done
        exit 1
    fi
    
    echo "⬇️  Téléchargement de $(basename "$URL")..."
    
    # Téléchargement avec retry
    MAX_RETRIES=3
    RETRY_COUNT=0
    
    while [ $RETRY_COUNT -lt $MAX_RETRIES ]; do
        if curl -fsSL --retry 3 --retry-delay 2 "$URL" -o "${TMP_DIR}/${LOCAL_NAME}"; then
            echo "✅ Téléchargement réussi"
            return 0
        else
            RETRY_COUNT=$((RETRY_COUNT + 1))
            if [ $RETRY_COUNT -lt $MAX_RETRIES ]; then
                echo "⚠️  Échec du téléchargement, tentative $RETRY_COUNT/$MAX_RETRIES..."
                sleep 2
            fi
        fi
    done
    
    echo "❌ Échec du téléchargement après $MAX_RETRIES tentatives"
    exit 1
}

# Installation des dépendances pour Tauri/WebKitGTK
install_tauri_dependencies() {
    echo "📦 Installation des dépendances requises pour Toolé (WebKitGTK)..."
    
    # Debian/Ubuntu
    if command -v apt-get >/dev/null 2>&1; then
        echo "🐧 Installation des dépendances Debian/Ubuntu..."
        sudo apt-get update
        sudo apt-get install -y \
            libwebkit2gtk-4.1-0 \
            libgtk-3-0 \
            libayatana-appindicator3-1 \
            librsvg2-common \
            libssl3 \
            libnotify4 \
            libsecret-1-0 \
            || sudo apt-get install -y \
            libwebkit2gtk-4.0-37 \
            libgtk-3-0 \
            libappindicator3-1 \
            librsvg2-common \
            libssl3 \
            libnotify4 \
            libsecret-1-0
    
    # Fedora/RHEL
    elif command -v dnf >/dev/null 2>&1; then
        echo "🐧 Installation des dépendances Fedora..."
        sudo dnf install -y \
            webkit2gtk4.1 \
            gtk3 \
            libappindicator-gtk3 \
            librsvg2 \
            openssl \
            libnotify \
            libsecret \
            || sudo dnf install -y \
            webkit2gtk3 \
            gtk3 \
            libappindicator-gtk3 \
            librsvg2 \
            openssl \
            libnotify \
            libsecret
    
    # Arch Linux
    elif command -v pacman >/dev/null 2>&1; then
        echo "🐧 Installation des dépendances Arch Linux..."
        sudo pacman -S --noconfirm \
            webkit2gtk-4.1 \
            webkit2gtk \
            gtk3 \
            libappindicator-gtk3 \
            librsvg \
            openssl \
            libnotify \
            libsecret
    fi
}

# ==========================================
# DÉTECTION OS ET INSTALLATION
# ==========================================
case "$OS" in
    Linux*)
        echo "🐧 Système détecté : Linux"
        
        # Installation des dépendances Tauri/WebKitGTK
        install_tauri_dependencies
        
        # 1. CAS UBUNTU / DEBIAN / MINT
        if command -v dpkg >/dev/null 2>&1; then
            echo "📦 Distribution Debian/Ubuntu détectée"
            
            # Télécharger le .deb (tous les .deb sont amd64)
            download_asset "amd64\.deb" "toole.deb"
            
            echo "📦 Installation du paquet .deb..."
            if sudo dpkg -i "${TMP_DIR}/toole.deb" 2>/dev/null; then
                echo "✅ Paquet installé avec succès"
            else
                echo "⚠️  Dépendances manquantes, correction automatique..."
                sudo apt-get install -f -y
                # Réessayer l'installation
                sudo dpkg -i "${TMP_DIR}/toole.deb"
            fi

        # 2. CAS FEDORA / REDHAT / CENTOS
        elif command -v rpm >/dev/null 2>&1; then
            echo "📦 Distribution Fedora/RHEL détectée"
            
            # Télécharger le .rpm (tous les .rpm sont x86_64)
            download_asset "x86_64\.rpm" "toole.rpm"
            
            echo "📦 Installation du paquet .rpm..."
            
            # DNF gère mieux les dépendances
            if command -v dnf >/dev/null 2>&1; then
                sudo dnf install -y "${TMP_DIR}/toole.rpm"
            else
                # Fallback sur yum pour les anciennes versions
                sudo yum install -y "${TMP_DIR}/toole.rpm"
            fi

        # 3. CAS ARCH LINUX
        elif command -v pacman >/dev/null 2>&1; then
            echo "📦 Distribution Arch Linux détectée"
            
            # Télécharger le .deb et extraire manuellement
            download_asset "amd64\.deb" "toole.deb"
            
            cd "$TMP_DIR"
            echo "📦 Extraction du paquet .deb..."
            
            if command -v bsdtar >/dev/null 2>&1; then
                bsdtar -xf "toole.deb"
                bsdtar -xf data.tar.*
            elif command -v ar >/dev/null 2>&1; then
                ar x "toole.deb"
                tar -xf data.tar.*
            else
                echo "❌ Utilitaires d'extraction introuvables."
                echo "   Installez libarchive : sudo pacman -S libarchive"
                exit 1
            fi
            
            echo "🚚 Installation manuelle..."
            sudo cp -r usr/* /usr/
            sudo chmod +x /usr/bin/toole 2>/dev/null || true
            sudo chmod +x /usr/bin/Toolé 2>/dev/null || true
            
        else
            echo "❌ Distribution Linux non reconnue"
            echo "   Systèmes supportés : Debian/Ubuntu, Fedora/RHEL, Arch Linux"
            exit 1
        fi
        ;;

    Darwin*)
        echo "🍎 Système détecté : macOS"
        
        # Télécharger le .dmg universel
        download_asset "universal\.dmg" "toole.dmg"
        
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
        echo "❌ Système d'exploitation non supporté : $OS"
        exit 1
        ;;
esac

echo ""
echo "✅ Toolé a été installé avec succès !"
echo ""

# ==========================================
# PARE-FEU
# ==========================================
# j'ouvre les ports UDP de Toolé (58199 découverte, 58200 réception) si un
# pare-feu est actif : ufw (Debian/Ubuntu) et firewalld (Fedora/RHEL).
# Jamais bloquant pour l'installation : en cas d'échec, simple avertissement.
configure_firewall() {
    echo "🛡️  Vérification du pare-feu..."

    # ufw : je n'ajoute que si le pare-feu est actif, et seulement les règles
    # manquantes (ufw refuse les doublons)
    if command -v ufw >/dev/null 2>&1; then
        if sudo ufw status 2>/dev/null | grep -q "Status: active"; then
            echo "🛡️  ufw actif : ouverture des ports UDP 58199/58200..."
            if ! sudo ufw status 2>/dev/null | grep -q "58199/udp"; then
                sudo ufw allow 58199/udp >/dev/null 2>&1 || echo "⚠️  Impossible d'ouvrir le port 58199/udp"
            fi
            if ! sudo ufw status 2>/dev/null | grep -q "58200/udp"; then
                sudo ufw allow 58200/udp >/dev/null 2>&1 || echo "⚠️  Impossible d'ouvrir le port 58200/udp"
            fi
        else
            echo "ℹ️  ufw présent mais inactif : rien à configurer."
        fi
    fi

    # firewalld : les ajouts permanents sont idempotents
    if command -v firewall-cmd >/dev/null 2>&1; then
        if sudo firewall-cmd --state 2>/dev/null | grep -q "running"; then
            echo "🛡️  firewalld actif : ouverture des ports UDP 58199/58200..."
            sudo firewall-cmd --permanent --add-port=58199/udp >/dev/null 2>&1 || echo "⚠️  Impossible d'ouvrir le port 58199/udp"
            sudo firewall-cmd --permanent --add-port=58200/udp >/dev/null 2>&1 || echo "⚠️  Impossible d'ouvrir le port 58200/udp"
            sudo firewall-cmd --reload >/dev/null 2>&1 || echo "⚠️  Rechargement firewalld impossible"
        else
            echo "ℹ️  firewalld présent mais inactif : rien à configurer."
        fi
    fi

    echo "✅ Vérification du pare-feu terminée."
}

case "$(uname -s)" in
    Linux*) configure_firewall ;;
esac

# Vérification finale
echo "🔍 Vérification de l'installation..."

if command -v toole >/dev/null 2>&1; then
    echo "✅ Toolé est disponible dans le PATH"
    echo ""
    echo "🚀 Pour lancer Toolé, tapez :"
    echo "   toole"
elif command -v Toolé >/dev/null 2>&1; then
    echo "✅ Toolé est disponible dans le PATH"
    echo ""
    echo "🚀 Pour lancer Toolé, tapez :"
    echo "   Toolé"
else
    echo "⚠️  Toolé est installé mais pas dans le PATH"
    echo "   Vérifiez : /usr/bin/toole ou /usr/bin/Toolé"
    
    # Vérifier si le binaire existe dans /usr/bin
    if [ -f "/usr/bin/toole" ] || [ -f "/usr/bin/Toolé" ]; then
        echo "✅ Binaire trouvé dans /usr/bin/"
        echo ""
        echo "🚀 Pour lancer Toolé :"
        if [ -f "/usr/bin/toole" ]; then
            echo "   /usr/bin/toole"
        else
            echo "   /usr/bin/Toolé"
        fi
    fi
fi
