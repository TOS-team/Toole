$ErrorActionPreference = "Stop"

$repo = "TOS-team/Toole"

Write-Host "📡 Recherche de la dernière release de Toolé..." -ForegroundColor Cyan
$release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest"
$tag = $release.tag_name
$version = $tag.TrimStart('v')

$fileName = "Toolé_${version}_x64-setup.exe"
$downloadUrl = "https://github.com/$repo/releases/download/$tag/$fileName"
$tempPath = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), $fileName)

Write-Host "⬇️ Téléchargement de $fileName ($tag)..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempPath

Write-Host "🚀 Lancement de l'installateur Windows..." -ForegroundColor Green
Start-Process -FilePath $tempPath -Wait

Remove-Item -Path $tempPath -ErrorAction SilentlyContinue
Write-Host "✅ Installation terminée !" -ForegroundColor Green
