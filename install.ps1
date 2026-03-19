$ErrorActionPreference = "Stop"

$Repo = "FabianTrafada/coco"
$InstallDir = "$env:USERPROFILE\.coco\bin"

function status($msg)  { Write-Host ">>> $msg" }
function err($msg)     { Write-Host "ERROR: $msg" -ForegroundColor Red; exit 1 }

# Detect architecture
$Arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "aarch64" } else { "x86_64" }

# Get latest release tag
status "Fetching latest version..."
$Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Tag = $Release.tag_name
if (-not $Tag) { err "Could not fetch latest version. Check your internet connection." }

# Download
$Artifact = "coco-windows-$Arch.exe"
$Url = "https://github.com/$Repo/releases/download/$Tag/$Artifact"

status "Downloading coco $Tag..."
if (-not (Test-Path $InstallDir)) {
  New-Item -ItemType Directory -Path $InstallDir | Out-Null
}
Invoke-WebRequest -Uri $Url -OutFile "$InstallDir\coco.exe"

# Add to PATH
$CurrentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
  [Environment]::SetEnvironmentVariable("PATH", "$CurrentPath;$InstallDir", "User")
}

status "Install complete. Run 'coco --help' to get started."