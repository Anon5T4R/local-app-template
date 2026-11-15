# nascer-app.ps1 - renomeia o molde local-app-template para um app novo.
#
# Uso (no repo recem-criado, ANTES do primeiro codigo):
#   powershell -File scripts/nascer-app.ps1 -Id localkeys -Name "Local Keys" -Repo Local-Keys
#
# O que faz:
#   1. Valida o id (nome maquina: minusculo, digitos e '-') e recusa colisao com pecas da suíte.
#   2. Renomeia arquivos e pastas que carregam o nome do molde (apps/localapp, <id>.wxs,
#      com.<id>.app.desktop, icones, catalogo i18n).
#   3. Troca o texto de identificacao em todo arquivo de texto do repo (nunca em libs/ nem em
#      assets/craft-fonts/, que sao arvores vendored byte a byte):
#        Anon5T4R/local-app-template -> Anon5T4R/<Repo>
#        com.localapp.app            -> com.<id>.app
#        LOCALAPP                    -> <ID em maiusculas>   (env vars, <PREFIX>_TITLEBAR)
#        Local App                   -> <Name>                 (display)
#        localapp                    -> <id>                   (crate, binario, prefixo das crates)
#   4. Gera GUIDs NOVOS para o MSI (UpgradeCode e Component GUIDs) - nunca reusa os do molde.
#   5. Lista o que mudou e o que ainda e manual (icone, tagline, entrada no catalogo, minisign).
#
# ASCII-only de proposito: PowerShell 5.1 le .ps1 sem BOM como ANSI; acento aqui quebraria o script.
param(
  [Parameter(Mandatory = $true)][string] $Id,
  [Parameter(Mandatory = $true)][string] $Name,
  [string] $Repo = '',
  [switch] $DryRun
)
$ErrorActionPreference = 'Stop'

$Root = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if (-not (Test-Path (Join-Path $Root 'apps\localapp'))) {
  throw "Este nao parece ser o molde: $Root\apps\localapp nao existe (ja renomeado?)."
}
if ($Id -notmatch '^[a-z][a-z0-9]*(-[a-z0-9]+)*$') {
  throw "Id invalido: '$Id'. Use minusculas, digitos e '-' (ex.: localkeys, local-player)."
}
if ($Id -eq 'localapp') { throw 'O id ainda e o do molde.' }
if (-not $Name.Trim()) { throw 'Name vazio.' }
if (-not $Repo) { $Repo = $Id }
if ($Repo -notmatch '^[A-Za-z0-9._-]+$') { throw "Repo invalido: '$Repo' (so o nome, sem owner)." }

$IdUpper = ($Id.ToUpperInvariant() -replace '-', '')
$Name_ = $Name.Trim()
$replacements = [ordered]@{
  'Anon5T4R/local-app-template' = "Anon5T4R/$Repo"
  'com.localapp.app'            = "com.$Id.app"
  'LOCALAPP'                    = $IdUpper
  'Local App'                   = $Name_
  'localapp'                    = $Id
}

# ---- 1. arquivos e pastas com o nome do molde ----------------------------------------------------
function Rename-Path([string] $Relative, [string] $NewRelative) {
  $from = Join-Path $Root $Relative
  $to = Join-Path $Root $NewRelative
  if (-not (Test-Path -LiteralPath $from)) { return }
  if ($DryRun) { Write-Output "  dir/arquivo: $Relative -> $NewRelative"; return }
  New-Item -ItemType Directory -Force -Path (Split-Path -Parent $to) | Out-Null
  Move-Item -LiteralPath $from -Destination $to
  Write-Output "  dir/arquivo: $Relative -> $NewRelative"
}

New-Item -ItemType Directory -Force -Path (Join-Path $Root 'apps') | Out-Null
Rename-Path 'apps\localapp' "apps\$Id"

$moves = @(
  @('packaging\windows\localapp.wxs', "packaging\windows\$Id.wxs"),
  @('packaging\linux\com.localapp.app.desktop', "packaging\linux\com.$Id.app.desktop"),
  @('packaging\linux\com.localapp.app.metainfo.xml.in', "packaging\linux\com.$Id.app.metainfo.xml.in"),
  @('assets\app-icon\localapp.svg', "assets\app-icon\$Id.svg"),
  @('assets\app-icon\localapp-1024.png', "assets\app-icon\$Id-1024.png"),
  @('assets\app-icon\localapp.ico', "assets\app-icon\$Id.ico")
)
foreach ($move in $moves) { Rename-Path $move[0] $move[1] }

foreach ($size in @('16x16', '24x24', '32x32', '48x48', '64x64', '128x128', '256x256', '512x512')) {
  Rename-Path "assets\app-icon\hicolor\$size\apps\com.localapp.app.png" "assets\app-icon\hicolor\$size\apps\com.$Id.app.png"
}
Rename-Path 'assets\app-icon\hicolor\scalable\apps\com.localapp.app.svg' "assets\app-icon\hicolor\scalable\apps\com.$Id.app.svg"

foreach ($locale in Get-ChildItem (Join-Path $Root 'i18n\locales') -Directory) {
  Rename-Path "i18n\locales\$($locale.Name)\localapp.tsv" "i18n\locales\$($locale.Name)\$Id.tsv"
}

# ---- 2. texto de identificacao em todo arquivo de texto ------------------------------------------
$textExtensions = @('.rs', '.toml', '.tsv', '.md', '.txt', '.yml', '.yaml', '.json', '.xml', '.wxs', '.ps1', '.sh', '.desktop', '.in', '.properties', '.lock')
$skip = @('libs', 'assets\craft-fonts', 'target', 'dist', 'tmp', '.git')
$changed = New-Object System.Collections.Generic.List[string]

$files = Get-ChildItem -LiteralPath $Root -Recurse -File -Force | Where-Object {
  $relative = $_.FullName.Substring($Root.Length + 1)
  $ext = $_.Extension.ToLowerInvariant()
  ($textExtensions -contains $ext) -and
    -not ($skip | Where-Object { $relative -eq $_ -or $relative.StartsWith("$_\") })
}

foreach ($file in $files) {
  $relative = $file.FullName.Substring($Root.Length + 1)
  $text = [System.IO.File]::ReadAllText($file.FullName, [System.Text.Encoding]::UTF8)
  $new = $text
  foreach ($key in $replacements.Keys) { $new = $new.Replace($key, $replacements[$key]) }

  if ($file.Name -like '*.wxs') {
    # GUIDs novos, um por ocorrencia: UpgradeCode e cada Component identificam o produto instalado.
    $new = [regex]::Replace($new, '[0-9A-Fa-f]{8}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{4}-[0-9A-Fa-f]{12}', {
      param($m) [guid]::NewGuid().ToString().ToUpperInvariant()
    })
  }

  if ($new -ne $text) {
    $changed.Add($relative)
    if (-not $DryRun) { [System.IO.File]::WriteAllText($file.FullName, $new, (New-Object System.Text.UTF8Encoding($false))) }
  }
}

Write-Output ''
Write-Output "$($changed.Count) arquivo(s) reescrito(s):"
$changed | ForEach-Object { Write-Output "  $_" }

Write-Output ''
Write-Output 'Ainda e manual (README.md, "Como nascer um app"):'
Write-Output '  1. icone: substituir o placeholder (assets/app-icon/README.md) e rodar packaging/icons.sh'
Write-Output '  2. tagline/descricao: apps/<id>/src/suite_entry.rs, metainfo, nfpm.yaml, PKGBUILD'
Write-Output '  3. entrada no catalogo: local-libs crates/update/src/suite.rs + SUITE, sync.ps1 -Mode push,'
Write-Output '     depois trocar o App declarado em apps/<id>/src/suite_entry.rs por craft_update::suite::<APP>'
Write-Output '  4. minisign: local-libs/scripts/minisign-nascimento.ps1 -Repo Anon5T4R/<repo> (e de novo apos o push)'
Write-Output '  5. dependencias novas: inventario no local-deps/DEPENDENCIES.md'
Write-Output '  6. registrar o app no docs/planos/padrao-craft.md e no local-deps (dev-notes)'
Write-Output ''
Write-Output "Conferir: cargo xtask fmt; cargo build --locked; cargo xtask assets; cargo xtask layers"
