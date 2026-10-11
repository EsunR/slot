param(
    [string]$Distro = 'Ubuntu',
    [switch]$Rg35xxsp
)

$ErrorActionPreference = 'Stop'
$repoWindows = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path.Replace('\', '/')
$repoLinux = & wsl.exe -d $Distro -- wslpath -a $repoWindows
if ($LASTEXITCODE -ne 0 -or -not $repoLinux) {
    throw "Cannot find the repository in WSL distro '$Distro'."
}
$repoLinux = $repoLinux.Trim()

$targetDir = if ($Rg35xxsp) { 'target-rg35xxsp' } else { 'target-device' }
$features = if ($Rg35xxsp) { 'device,rg35xxsp' } else { 'device' }
$build = 'source "$HOME/.cargo/env" && CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build --release --target aarch64-unknown-linux-gnu --target-dir ' + $targetDir + ' -p slot --no-default-features --features ' + $features
& wsl.exe -d $Distro --cd $repoLinux -- bash -lc $build
if ($LASTEXITCODE -ne 0) {
    throw 'The aarch64 device build failed.'
}

$binary = "$targetDir/aarch64-unknown-linux-gnu/release/slot"
& wsl.exe -d $Distro --cd $repoLinux -- python3 verify_slot.py $binary
if ($LASTEXITCODE -ne 0) {
    throw 'The aarch64 binary verification failed.'
}

& wsl.exe -d $Distro --cd $repoLinux -- python3 check_glibc.py $binary
if ($LASTEXITCODE -ne 0) {
    throw 'The glibc inspection failed.'
}

$binaryWindows = (Resolve-Path (Join-Path $PSScriptRoot "..\$($binary.Replace('/', '\'))")).Path
Write-Host "Device binary: $binaryWindows"
