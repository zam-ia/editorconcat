param(
    [Parameter(Mandatory = $true)]
    [string]$DownloadsRoot
)

$ErrorActionPreference = 'Stop'
$effectsRoot = Join-Path $env:APPDATA 'app.concat.editor\effects'
New-Item -ItemType Directory -Force -Path $effectsRoot | Out-Null

$striped = Join-Path $DownloadsRoot 'color-grading-luts'
$a6000 = Join-Path $DownloadsRoot 'a6000-LUTs'
$chosen = @()
$chosen += Get-ChildItem (Join-Path $striped 'Cinema Through The Ages') -Filter '*.cube' -File
foreach ($name in @(
    'Striped Purple - December Greens.cube',
    'Striped Purple - Philly Fog.cube',
    'Striped Purple - Teal Embers.cube',
    'Striped Purple - Teal Supreme.cube'
)) {
    $chosen += Get-Item -LiteralPath (Join-Path $striped "Scenic Moods\$name")
}
foreach ($name in @(
    'Striped Purple - Amber Haze.cube',
    'Striped Purple - Caramel Plum Cake.cube',
    'Striped Purple - Cherry Champagne.cube',
    'Striped Purple - Polaroid Mint Cream.cube'
)) {
    $chosen += Get-Item -LiteralPath (Join-Path $striped "Avant Garde\$name")
}
$chosen += Get-ChildItem $a6000 -Filter '*.cube' -File

$shader = @'
// The table, carried past its ends, and nothing else; the host mixes it
// by intensity.
fn effect(uv: vec2<f32>) -> vec4<f32> {
    let c = sample(uv);
    return vec4<f32>(look(c.rgb), c.a);
}
'@

foreach ($file in $chosen) {
    $stem = $file.BaseName.Trim()
    $slug = (($stem.ToLowerInvariant() -replace '[^a-z0-9]+', '-').Trim('-'))
    $id = "user.$slug"
    $folder = Join-Path $effectsRoot $id
    New-Item -ItemType Directory -Force -Path $folder | Out-Null
    Copy-Item -LiteralPath $file.FullName -Destination (Join-Path $folder 'look.cube') -Force
    $manifest = @"
format = 2

[effect]
id = "$id"
name = "$stem"
kind = "filter"
category = "LUTs cinematograficos"
description = "Look cinematografico importado con intensidad regulable."

[lut]
file = "look.cube"

[wgsl]
entry = "effect.wgsl"
space = "display"
"@
    [IO.File]::WriteAllText(
        (Join-Path $folder 'effect.toml'),
        $manifest,
        [Text.UTF8Encoding]::new($false)
    )
    [IO.File]::WriteAllText(
        (Join-Path $folder 'effect.wgsl'),
        $shader,
        [Text.UTF8Encoding]::new($false)
    )
}

Copy-Item (Join-Path $striped 'LICENSE') (Join-Path $effectsRoot 'LICENSE-StripedPurple-MIT.txt') -Force
Copy-Item (Join-Path $a6000 'LICENSE') (Join-Path $effectsRoot 'LICENSE-a6000-CC0.txt') -Force

$installed = Get-ChildItem $effectsRoot -Directory | Where-Object Name -Like 'user.*'
$bytes = ($installed | Get-ChildItem -Recurse -File | Measure-Object Length -Sum).Sum
[pscustomobject]@{
    InstalledLuts = $installed.Count
    SizeMB = [math]::Round($bytes / 1MB, 1)
    Folder = $effectsRoot
}
