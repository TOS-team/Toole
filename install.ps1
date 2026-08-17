# Script d'installation PowerShell pour Toolé
# Transfert de fichiers P2P sur réseau local, chiffré (QUIC/TLS 1.3)

$ErrorActionPreference = "Stop"

# Configuration
$repo = "TOS-team/Toole"
$appName = "Toolé"

# Fonctions d'affichage
function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARNING] $Message" -ForegroundColor Yellow
}

function Write-ErrorMsg {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
    exit 1
}

# Bannière
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "     Installation de $appName" -ForegroundColor Cyan
Write-Host "  Transfert P2P chiffré sur réseau local" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Vérification des privilèges administrateur (optionnel)
function Test-Admin {
    $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Récupération de la dernière version
try {
    Write-Info "Recherche de la dernière version de $appName..."
    
    # Ajout d'un User-Agent pour éviter les limitations GitHub
    $headers = @{
        "User-Agent" = "Toole-Installer-Script"
        "Accept" = "application/vnd.github.v3+json"
    }
    
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest" -Headers $headers
    
    if (-not $release) {
        Write-ErrorMsg "Impossible de récupérer les informations de la release"
    }
    
    $tag = $release.tag_name
    $version = $tag.TrimStart('v')
    
    Write-Success "Version trouvée : $tag"
    Write-Host ""
    
} catch {
    if ($_.Exception.Response.StatusCode -eq 403) {
        Write-ErrorMsg "Limite de requêtes GitHub atteinte. Réessayez dans quelques minutes."
    } elseif ($_.Exception.Response.StatusCode -eq 404) {
        Write-ErrorMsg "Dépôt ou release introuvable : $repo"
    } else {
        Write-ErrorMsg "Erreur lors de la récupération de la version : $_"
    }
}

# Construction du nom du fichier
$fileName = "Toolé_${version}_x64-setup.exe"
$downloadUrl = "https://github.com/$repo/releases/download/$tag/$fileName"
$tempPath = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), $fileName)

# Vérification de l'architecture
if (-not [Environment]::Is64BitOperatingSystem) {
    Write-Warning "Système 32 bits détecté. Toolé nécessite un système 64 bits."
    $response = Read-Host "Voulez-vous continuer quand même ? (o/N)"
    if ($response -notin @('o', 'O', 'oui', 'Oui', 'OUI', 'y', 'Y', 'yes', 'Yes', 'YES')) {
        Write-ErrorMsg "Installation annulée"
    }
}

# Téléchargement
try {
    Write-Info "Téléchargement de $fileName ($tag)..."
    
    # Téléchargement avec barre de progression
    $progressPreference = 'SilentlyContinue'  # Désactiver la barre de progression pour plus de clarté
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempPath -ErrorAction Stop
    $progressPreference = 'Continue'
    
    # Vérification que le fichier a bien été téléchargé
    if (-not (Test-Path $tempPath)) {
        Write-ErrorMsg "Le fichier téléchargé est introuvable"
    }
    
    $fileSize = (Get-Item $tempPath).Length
    if ($fileSize -lt 1MB) {
        Write-Warning "Le fichier téléchargé semble trop petit ($([Math]::Round($fileSize/1KB, 2)) KB)"
        $response = Read-Host "Voulez-vous continuer ? (o/N)"
        if ($response -notin @('o', 'O', 'oui', 'Oui', 'OUI', 'y', 'Y', 'yes', 'Yes', 'YES')) {
            Remove-Item -Path $tempPath -ErrorAction SilentlyContinue
            Write-ErrorMsg "Installation annulée"
        }
    }
    
    Write-Success "Téléchargement terminé ($([Math]::Round($fileSize/1MB, 2)) MB)"
    Write-Host ""
    
} catch {
    Remove-Item -Path $tempPath -ErrorAction SilentlyContinue
    Write-ErrorMsg "Erreur lors du téléchargement : $_"
}

# Lancement de l'installateur
try {
    Write-Info "Lancement de l'installateur Windows..."
    Write-Warning "Suivez les instructions à l'écran pour terminer l'installation."
    Write-Host ""
    
    # Lancement de l'installateur
    $process = Start-Process -FilePath $tempPath -PassThru -Wait
    
    if ($process.ExitCode -ne 0 -and $process.ExitCode -ne 3010) {
        Write-Warning "L'installateur a retourné le code : $($process.ExitCode)"
        Write-Warning "Code 3010 = redémarrage requis (normal pour certaines installations)"
    }
    
    Write-Success "Installation terminée !"
    
} catch {
    Write-ErrorMsg "Erreur lors du lancement de l'installateur : $_"
} finally {
    # Nettoyage du fichier temporaire
    Write-Info "Nettoyage des fichiers temporaires..."
    Remove-Item -Path $tempPath -ErrorAction SilentlyContinue
}

# Vérification de l'installation
Write-Host ""
Write-Info "Vérification de l'installation..."

# Vérifier si Toolé est installé
$tooleInstalled = $false

# Vérifier dans le registre Windows
$registryPaths = @(
    "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
    "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*"
)

foreach ($path in $registryPaths) {
    try {
        $apps = Get-ItemProperty $path -ErrorAction SilentlyContinue | Where-Object { $_.DisplayName -like "*Toolé*" -or $_.DisplayName -like "*Toole*" }
        if ($apps) {
            $tooleInstalled = $true
            Write-Success "Toolé est installé sur votre système"
            if ($apps.InstallLocation) {
                Write-Info "Emplacement : $($apps.InstallLocation)"
            }
            break
        }
    } catch {
        # Ignorer les erreurs de lecture du registre
    }
}

if (-not $tooleInstalled) {
    Write-Warning "Impossible de vérifier automatiquement l'installation"
    Write-Info "Recherchez 'Toolé' dans le menu Démarrer"
}

Write-Host ""
Write-Success "Installation terminée ! 🎉"
Write-Host ""
Write-Host "Toolé - Transfert de fichiers P2P sur réseau local, chiffré (QUIC/TLS 1.3)" -ForegroundColor Cyan

# Option : Ajouter au PATH si nécessaire
$installDir = "$env:LOCALAPPDATA\Toolé"
if (Test-Path "$installDir\toole.exe") {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$installDir*") {
        Write-Info "Ajout de Toolé au PATH utilisateur..."
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
        Write-Success "Toolé ajouté au PATH"
        Write-Info "Redémarrez votre terminal pour utiliser la commande 'toole'"
    }
}
