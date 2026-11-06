param([switch]$Release, [string]$Output = "dist/preview-0.1.0a1-windows")
$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path "$PSScriptRoot/../../..").Path
$env:RUSTUP_HOME = "$workspace/.tooling/rustup"
$env:CARGO_HOME = "$workspace/.tooling/cargo"
$env:RUSTUP_TOOLCHAIN = '1.95.0'
$env:PATH = "$env:CARGO_HOME/bin;$env:PATH"
$env:CRAFT_I18N_DIR = "$workspace/.tooling/craft-i18n"
$env:CRAFT_I18N_REQUIRED = '1'
$env:PYO3_PYTHON = "$workspace/.tooling/stage3/build-env/Scripts/python.exe"
$vswhere = "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
$vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (!$vs) { throw 'Visual Studio x64 C++ build tools required' }
Import-Module "$vs/Common7/Tools/Microsoft.VisualStudio.DevShell.dll"
Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
Set-Location "$workspace/foliorivet"
if ($Release) {
 & $env:PYO3_PYTHON tools/release/build_candidate.py --i18n $env:CRAFT_I18N_DIR --output $Output --offline
 exit $LASTEXITCODE
}
cargo build --locked --offline -p wordcraft-cli --bin foliorivet-cli
if ($LASTEXITCODE) { exit $LASTEXITCODE }
& $env:PYO3_PYTHON -m maturin build --locked --offline --manifest-path bindings/python/Cargo.toml --interpreter $env:PYO3_PYTHON --out dist/windows
exit $LASTEXITCODE
