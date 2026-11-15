# Collect the embedded fonts' licence texts into one file for the installer and the portable zip.
# Keeps the upstream texts intact, including the copyright notices above each OFL.
function Write-FontLicences([string] $Destination, [string] $CraftFontsDir) {
  if ($env:CRAFT_FONTS_REQUIRED -and -not $CraftFontsDir) { throw 'CRAFT_FONTS_REQUIRED is set but CRAFT_FONTS_DIR is missing' }
  $parts = [Collections.Generic.List[string]]::new()
  $parts.Add("Local App font licences`n")
  # egui's bundled fonts (Ubuntu Light, Hack, Noto Emoji) are covered by NOTICE, which ships beside
  # this file; the craft-fonts faces embedded from the checkout are listed below.
  if ($CraftFontsDir) {
    $manifest = Join-Path $CraftFontsDir 'fonts\manifest.txt'
    $licences = Get-Content -LiteralPath $manifest | ForEach-Object {
      if ($_.Trim() -and -not $_.Trim().StartsWith('#')) {
        $fields = $_ -split '\|'
        if ($fields.Count -lt 6) { throw "Malformed font manifest line: $_" }
        $fields[5].Trim()
      }
    } | Sort-Object -Unique
    foreach ($licence in $licences) {
      $parts.Add("`n--- $licence ---`n")
      $parts.Add([IO.File]::ReadAllText((Join-Path $CraftFontsDir $licence)))
    }
  }
  [IO.File]::WriteAllText($Destination, ($parts -join "`n"), [Text.UTF8Encoding]::new($false))
}
