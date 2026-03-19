$ErrorActionPreference = "Stop"

$Repo = "FabianTrafada/coco"
$InstallDir = "$env:USERPROFILE\.coco\bin"

Write-Host ""
Write-Host "  Installing coco 🥥"
Write-Host ""

# Detect architecture
$Arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "aarch64" } else { "x86_64" }
Write-Host "  Detected: windows/$Arch"

# Get latest release tag
Write-Host "  Fetching latest version..."
$Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Tag = $Release.tag_name

if (-not $Tag) {
  Write-Host "  x Could not fetch latest release." -ForegroundColor Red
  exit 1
}

Write-Host "  Latest version: $Tag"

# Build download URL
$Artifact = "coco-windows-$Arch.exe"
$Url = "https://github.com/$Repo/releases/download/$Tag/$Artifact"

# Download
Write-Host "  Downloading $Artifact..."
if (-not (Test-Path $InstallDir)) {
  New-Item -ItemType Directory -Path $InstallDir | Out-Null
}
Invoke-WebRequest -Uri $Url -OutFile "$InstallDir\coco.exe"

# Add to PATH if not already there
$CurrentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
  [Environment]::SetEnvironmentVariable("PATH", "$CurrentPath;$InstallDir", "User")
  Write-Host "  v Added $InstallDir to PATH" -ForegroundColor Green
}

Write-Host ""
Write-Host "  v coco $Tag installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "  Restart your terminal, then run 'coco --help' to get started."
Write-Host ""