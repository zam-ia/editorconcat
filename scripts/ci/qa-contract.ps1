param(
  [switch]$SkipExport
)

$ErrorActionPreference = "Stop"
$env:CARGO_BUILD_JOBS = "1"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$cargoRoot = Join-Path $root "src"

# The local Windows bundle keeps native libraries outside Cargo's target
# directory.  CI may already provide these variables; a checkout made with the
# repository bootstrap uses the paths below.
$ffmpeg = Get-ChildItem (Join-Path $root ".build-deps\ffmpeg") -Directory -ErrorAction SilentlyContinue |
  Select-Object -First 1
$ort = Get-ChildItem (Join-Path $root ".build-deps\onnxruntime") -Directory -ErrorAction SilentlyContinue |
  Select-Object -First 1
$sherpa = Join-Path $root ".build-deps\sherpa-dlls"
$sherpaLibs = Join-Path $root ".build-deps\sherpa-libs"
$cmake = Get-Command cmake -ErrorAction SilentlyContinue
if (-not $cmake) {
  $cmake = Get-ChildItem (Join-Path $env:LOCALAPPDATA "Microsoft\WinGet\Packages") `
    -Filter "cmake.exe" -Recurse -ErrorAction SilentlyContinue |
    Select-Object -First 1
}
if ($ffmpeg) {
  $env:FFMPEG_DIR = $ffmpeg.FullName
  $env:PATH = "$(Join-Path $ffmpeg.FullName 'bin');$env:PATH"
}
if ($ort) {
  $env:ORT_LIB_LOCATION = Join-Path $ort.FullName "lib"
  $env:PATH = "$env:ORT_LIB_LOCATION;$env:PATH"
}
if (Test-Path $sherpa) {
  $env:SHERPA_LIB_PATH = $sherpa
  $env:PATH = "$sherpa;$env:PATH"
}
if (Test-Path $sherpaLibs) {
  $env:SHERPA_ONNX_LIB_DIR = $sherpaLibs
}
if ($cmake) {
  $cmakePath = if ($cmake.Source) { $cmake.Source } else { $cmake.FullName }
  $env:PATH = "$(Split-Path -Parent $cmakePath);$env:PATH"
}

$contracts = @(
  [pscustomobject]@{
    Id = "fill-reset-atomic"
    Dimensions = "processing,undo-redo,persistence"
    Args = @("test", "-p", "concat", "--lib", "shape_fill_reset_is_one_atomic_undo_step", "--", "--nocapture")
  },
  [pscustomobject]@{
    Id = "project-history"
    Dimensions = "undo-redo"
    Args = @("test", "-p", "concat-project", "undo_and_redo_walk_the_history")
  },
  [pscustomobject]@{
    Id = "project-round-trip"
    Dimensions = "persistence"
    Args = @("test", "-p", "concat-project", "the_document_round_trips")
  },
  [pscustomobject]@{
    Id = "project-0.2-compatibility"
    Dimensions = "backward-compatibility"
    Args = @("test", "-p", "concat-project", "a_legacy_flat_document_loads_as_one_timeline")
  },
  [pscustomobject]@{
    Id = "session-save-reopen"
    Dimensions = "persistence,backward-compatibility"
    Args = @("test", "-p", "concat-host", "a_fresh_project_opens_empty_and_round_trips_a_save")
  }
)

if (-not $SkipExport) {
  $contracts += @(
    [pscustomobject]@{
      Id = "preview-export-parity"
      Dimensions = "processing,preview-export-match"
      Args = @("test", "-p", "concat-host", "--test", "export", "a_background_fills_the_bars_in_the_export_and_the_monitor", "--", "--nocapture")
    },
    [pscustomobject]@{
      Id = "audiovisual-export-regression"
      Dimensions = "processing,export"
      Args = @("test", "-p", "concat-host", "--test", "export", "every_edit_still_exports", "--", "--nocapture")
    }
  )
}

$results = @()
Push-Location $cargoRoot
try {
  foreach ($contract in $contracts) {
    $started = Get-Date
    Write-Host "[QA] $($contract.Id): $($contract.Dimensions)"
    & cargo @($contract.Args)
    $code = $LASTEXITCODE
    $results += [pscustomobject]@{
      Contract = $contract.Id
      Dimensions = $contract.Dimensions
      Result = if ($code -eq 0) { "PASS" } else { "FAIL ($code)" }
      Seconds = [math]::Round(((Get-Date) - $started).TotalSeconds, 1)
    }
    if ($code -ne 0) {
      break
    }
  }
}
finally {
  Pop-Location
}

$results | Format-Table -AutoSize
if ($results.Result -match '^FAIL') {
  exit 1
}
