param(
  [switch]$SkipExport,
  [switch]$SkipUi
)

# Windows PowerShell 5 wraps ordinary Cargo stderr progress as a
# NativeCommandError when Stop is active. Native failures are checked through
# LASTEXITCODE below; real script failures still use throw/exit explicitly.
$ErrorActionPreference = "Continue"
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

function Test-ContractEvidence {
  param(
    [string[]]$Output,
    [string]$ExpectedTest
  )
  $text = $Output -join "`n"
  $positiveRun = [regex]::IsMatch($text, '(?m)^running\s+[1-9][0-9]*\s+tests?\s*$')
  $escaped = [regex]::Escape($ExpectedTest)
  $expectedPassed = [regex]::IsMatch($text, "(?m)^test .*${escaped}.* \.\.\. ok\s*$")
  return $positiveRun -and $expectedPassed
}

# Cargo returns success when a filter matches zero tests. Keep a direct guard
# on the parser so a future output/refactor cannot silently turn every
# contract green without executing its named test.
if (Test-ContractEvidence @('running 0 tests', 'test result: ok. 0 passed') 'missing_test') {
  throw 'QA evidence parser accepted a zero-test run'
}

$contracts = @()
if (-not $SkipUi) {
  $contracts += [pscustomobject]@{
    Id = "fill-reset-atomic"
    Dimensions = "processing,undo-redo,persistence"
    ExpectedTest = "shape_fill_reset_is_one_atomic_undo_step"
    Args = @("test", "-p", "concat", "--lib", "shape_fill_reset_is_one_atomic_undo_step", "--", "--nocapture")
  }
}
$contracts += @(
  [pscustomobject]@{
    Id = "project-history"
    Dimensions = "undo-redo"
    ExpectedTest = "undo_and_redo_walk_the_history"
    Args = @("test", "-p", "concat-project", "undo_and_redo_walk_the_history")
  },
  [pscustomobject]@{
    Id = "project-round-trip"
    Dimensions = "persistence"
    ExpectedTest = "the_document_round_trips"
    Args = @("test", "-p", "concat-project", "the_document_round_trips")
  },
  [pscustomobject]@{
    Id = "project-0.2-compatibility"
    Dimensions = "backward-compatibility"
    ExpectedTest = "a_legacy_flat_document_loads_as_one_timeline"
    Args = @("test", "-p", "concat-project", "a_legacy_flat_document_loads_as_one_timeline")
  },
  [pscustomobject]@{
    Id = "session-save-reopen"
    Dimensions = "persistence,backward-compatibility"
    ExpectedTest = "a_fresh_project_opens_empty_and_round_trips_a_save"
    Args = @("test", "-p", "concat-host", "a_fresh_project_opens_empty_and_round_trips_a_save")
  },
  [pscustomobject]@{
    Id = "audio-normalization-atomic"
    Dimensions = "processing,undo-redo,persistence"
    ExpectedTest = "audio_normalization_is_one_atomic_undo_step_and_persists"
    Args = @("test", "-p", "concat-project", "audio_normalization_is_one_atomic_undo_step_and_persists", "--", "--nocapture")
  },
  [pscustomobject]@{
    Id = "audio-meter-export-reference"
    Dimensions = "processing,preview-export-match,export"
    ExpectedTest = "rendered_mix_normalization_matches_the_export_signal"
    Args = @("test", "-p", "concat-media", "--lib", "rendered_mix_normalization_matches_the_export_signal", "--", "--nocapture")
  }
)

if (-not $SkipExport) {
  $contracts += @(
    [pscustomobject]@{
      Id = "preview-export-parity"
      Dimensions = "processing,preview-export-match"
      ExpectedTest = "a_background_fills_the_bars_in_the_export_and_the_monitor"
      Args = @("test", "-p", "concat-host", "--test", "export", "a_background_fills_the_bars_in_the_export_and_the_monitor", "--", "--nocapture")
    },
    [pscustomobject]@{
      Id = "audiovisual-export-regression"
      Dimensions = "processing,export"
      ExpectedTest = "every_edit_still_exports"
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
    $output = @(& cargo @($contract.Args) 2>&1 | Tee-Object -Variable contractOutput)
    $code = $LASTEXITCODE
    $evidence = Test-ContractEvidence $output $contract.ExpectedTest
    if ($code -eq 0 -and -not $evidence) {
      Write-Error "[QA] $($contract.Id) returned success but did not execute expected test '$($contract.ExpectedTest)'" -ErrorAction Continue
      $code = 97
    }
    $results += [pscustomobject]@{
      Contract = $contract.Id
      Dimensions = $contract.Dimensions
      Result = if ($code -eq 0) { "PASS" } else { "FAIL ($code)" }
      Evidence = if ($evidence) { $contract.ExpectedTest } else { "missing" }
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
