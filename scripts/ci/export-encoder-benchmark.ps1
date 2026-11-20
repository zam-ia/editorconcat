param(
    [int]$Frames = 300,
    [int]$Width = 1440,
    [int]$Height = 2560,
    [int]$Fps = 30,
    [string]$OutputDirectory = (Join-Path $env:TEMP 'concat-encoder-benchmark')
)

$ErrorActionPreference = 'Stop'
$repo = Resolve-Path (Join-Path $PSScriptRoot '..\..')
$ffmpegRoot = Get-ChildItem (Join-Path $repo '.build-deps\ffmpeg') -Directory |
    Select-Object -First 1
if (-not $ffmpegRoot) {
    throw 'Bundled FFmpeg was not found under .build-deps\ffmpeg.'
}
$ffmpeg = Join-Path $ffmpegRoot.FullName 'bin\ffmpeg.exe'
New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null

$cases = @(
    @{ Name = 'libx264'; Arguments = @('-c:v', 'libx264', '-preset', 'veryfast') },
    @{ Name = 'h264_qsv'; Arguments = @('-c:v', 'h264_qsv') },
    @{ Name = 'h264_mf'; Arguments = @('-c:v', 'h264_mf') }
)
$rows = foreach ($case in $cases) {
    $output = Join-Path $OutputDirectory ($case.Name + '.mp4')
    if (Test-Path -LiteralPath $output) {
        Remove-Item -LiteralPath $output -Force
    }
    $arguments = @(
        '-hide_banner', '-loglevel', 'error',
        '-f', 'lavfi', '-i', "testsrc2=size=${Width}x${Height}:rate=$Fps",
        '-frames:v', $Frames,
        '-pix_fmt', 'yuv420p'
    ) + $case.Arguments + @('-b:v', '12M', '-y', $output)
    $timer = [Diagnostics.Stopwatch]::StartNew()
    & $ffmpeg @arguments
    $exitCode = $LASTEXITCODE
    $timer.Stop()
    [pscustomobject]@{
        Encoder = $case.Name
        ExitCode = $exitCode
        Seconds = [math]::Round($timer.Elapsed.TotalSeconds, 3)
        FPS = if ($exitCode -eq 0) {
            [math]::Round($Frames / $timer.Elapsed.TotalSeconds, 2)
        } else { 0 }
        Bytes = if (Test-Path -LiteralPath $output) {
            (Get-Item -LiteralPath $output).Length
        } else { 0 }
    }
}

$json = Join-Path $OutputDirectory 'results.json'
$rows | ConvertTo-Json | Set-Content -LiteralPath $json -Encoding utf8
$rows | Format-Table -AutoSize
Write-Output "Results: $json"
if (($rows | Where-Object ExitCode -ne 0).Count -eq $rows.Count) {
    exit 1
}
