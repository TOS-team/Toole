#Requires -Version 5.1

$REPO = "https://github.com/TOS-team/Toole"
$INSTALL_DIR = "$env:LOCALAPPDATA\Toolé"

function Info { param([string]$msg) Write-Host "[Toolé] $msg" -ForegroundColor Green }
function Warn { param([string]$msg) Write-Host "[Toolé] $msg" -ForegroundColor Yellow }
function Err  { param([string]$msg) Write-Host "[Toolé] $msg" -ForegroundColor Red }

# Détection architecture
$ARCH = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }

Info "Recherche de la dernière version..."

$release = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest"
$VERSION = $release.tag_name.TrimStart('v')
$ASSET = $release.assets | Where-Object { $_.name -like "*setup.exe" -or $_.name -like "*.msi" } | Select-Object -First 1

if (-not $ASSET) {
    Err "Aucun installateur trouvé pour Windows."
    exit 1
}

$URL = $ASSET.browser_download_url
$FILENAME = $ASSET.name
$TEMP = "$env:TEMP\$FILENAME"

Info "Téléchargement de $FILENAME..."
Invoke-WebRequest -Uri $URL -OutFile $TEMP -UseBasicParsing

if ($FILENAME -like "*.msi") {
    Info "Installation silencieuse MSI..."
    Start-Process -FilePath "msiexec.exe" -ArgumentList "/i", "`"$TEMP`"", "/quiet", "/norestart" -Wait
} else {
    Info "Lancement de l'installateur..."
    Start-Process -FilePath $TEMP -Wait
}

Remove-Item $TEMP -ErrorAction SilentlyContinue
Info "✅ Toolé installé !"
Info "Lance Toolé depuis le menu Démarrer."
