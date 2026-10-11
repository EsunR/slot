param(
    [string]$Distro = 'Ubuntu'
)

$ErrorActionPreference = 'Stop'
& (Join-Path $PSScriptRoot 'build-device-wsl.ps1') -Distro $Distro -Rg35xxsp
if ($LASTEXITCODE -ne 0) {
    throw 'The RG35XXSP build failed.'
}

$repoWindows = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path.Replace('\', '/')
$repoLinux = & wsl.exe -d $Distro -- wslpath -a $repoWindows
if ($LASTEXITCODE -ne 0 -or -not $repoLinux) {
    throw "Cannot find the repository in WSL distro '$Distro'."
}
& wsl.exe -d $Distro --cd $repoLinux.Trim() -- bash tools/package-rg35xxsp-wsl.sh
if ($LASTEXITCODE -ne 0) {
    throw 'The RG35XXSP content tree could not be assembled.'
}
Write-Host "Content tree: $(Join-Path (Resolve-Path (Join-Path $PSScriptRoot '..')).Path 'target-rg35xxsp-sdcard')"
