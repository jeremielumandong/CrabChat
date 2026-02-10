param(
    [Parameter(Position = 0)]
    [string]$Command = "dev"
)

$ErrorActionPreference = "Stop"

$APP = "crabchat"
$VERSION = (Select-String -Path "Cargo.toml" -Pattern '^version\s*=\s*"(.*)"' | Select-Object -First 1).Matches.Groups[1].Value
$TARGET_DIR = "target"
$INSTALL_DIR = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { "$env:USERPROFILE\.cargo\bin" }

function Show-Usage {
    Write-Host @"
Usage: .\build.ps1 [command]

Commands:
  dev         Build debug binary (default)
  release     Build optimized release binary
  test        Run all tests
  check       Run cargo check + clippy
  clean       Remove build artifacts
  install     Build release and install to $INSTALL_DIR
  uninstall   Remove installed binary
  run         Build debug and run
  fmt         Format code
  size        Show release binary size

Environment:
  INSTALL_DIR   Installation directory (default: $env:USERPROFILE\.cargo\bin)
"@
}

function Invoke-Dev {
    Write-Host "==> Building debug binary..."
    cargo build
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    Write-Host "==> Built: $TARGET_DIR\debug\$APP.exe"
}

function Invoke-Release {
    Write-Host "==> Building release binary..."
    cargo build --release
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $bin = "$TARGET_DIR\release\$APP.exe"
    $size = (Get-Item $bin).Length
    $sizeStr = if ($size -ge 1MB) { "{0:N1} MB" -f ($size / 1MB) } else { "{0:N0} KB" -f ($size / 1KB) }
    Write-Host "==> Built: $bin ($sizeStr)"
}

function Invoke-Test {
    Write-Host "==> Running tests..."
    cargo test
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

function Invoke-Check {
    Write-Host "==> Running cargo check..."
    cargo check
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    Write-Host "==> Running clippy..."
    cargo clippy -- -D warnings 2>$null
    if ($LASTEXITCODE -ne 0) { cargo clippy }
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

function Invoke-Clean {
    Write-Host "==> Cleaning build artifacts..."
    cargo clean
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    Write-Host "==> Done."
}

function Invoke-Install {
    Invoke-Release
    Write-Host "==> Installing to $INSTALL_DIR\$APP.exe..."
    if (-not (Test-Path $INSTALL_DIR)) {
        New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
    }
    Copy-Item "$TARGET_DIR\release\$APP.exe" "$INSTALL_DIR\$APP.exe" -Force
    Write-Host "==> Installed $APP v$VERSION to $INSTALL_DIR\$APP.exe"
}

function Invoke-Uninstall {
    $bin = "$INSTALL_DIR\$APP.exe"
    if (Test-Path $bin) {
        Write-Host "==> Removing $bin..."
        Remove-Item $bin -Force
        Write-Host "==> Uninstalled."
    } else {
        Write-Host "==> $APP is not installed at $bin"
    }
}

function Invoke-Run {
    Write-Host "==> Building and running..."
    cargo run
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

function Invoke-Fmt {
    Write-Host "==> Formatting code..."
    cargo fmt
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    Write-Host "==> Done."
}

function Invoke-Size {
    $bin = "$TARGET_DIR\release\$APP.exe"
    if (-not (Test-Path $bin)) {
        Invoke-Release
    }
    Write-Host ""
    Write-Host "Binary: $bin"
    $size = (Get-Item $bin).Length
    $sizeStr = if ($size -ge 1MB) { "{0:N1} MB" -f ($size / 1MB) } else { "{0:N0} KB" -f ($size / 1KB) }
    Write-Host "Size:   $sizeStr"
}

switch ($Command) {
    "dev"       { Invoke-Dev }
    "release"   { Invoke-Release }
    "test"      { Invoke-Test }
    "check"     { Invoke-Check }
    "clean"     { Invoke-Clean }
    "install"   { Invoke-Install }
    "uninstall" { Invoke-Uninstall }
    "run"       { Invoke-Run }
    "fmt"       { Invoke-Fmt }
    "size"      { Invoke-Size }
    { $_ -in "-h", "--help", "help" } { Show-Usage }
    default {
        Write-Host "Unknown command: $Command"
        Show-Usage
        exit 1
    }
}
