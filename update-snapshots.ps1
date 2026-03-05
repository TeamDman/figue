param(
    [string]$Root = $PSScriptRoot,

    [switch]$Windows,

    [switch]$Linux
)

if ($Windows -and $Linux) {
    throw 'Use either -Windows or -Linux, not both.'
}

$osSep = if ($Windows) {
    '\'
} elseif ($Linux) {
    '/'
} else {
    [System.IO.Path]::DirectorySeparatorChar
}
$otherSep = if ($osSep -eq '/') { '\' } else { '/' }

$snapshotFiles = Get-ChildItem -Path $Root -Recurse -File -Filter '*.snap'

foreach ($file in $snapshotFiles) {
    $original = Get-Content -Path $file.FullName -Raw

    $updated = [regex]::Replace(
        $original,
        '(?m)(defined at\s+)([^\r\n]+)',
        {
            param($match)
            $prefix = $match.Groups[1].Value
            $pathPart = $match.Groups[2].Value
            $normalizedPath = $pathPart -replace [regex]::Escape($otherSep), [string]$osSep
            "$prefix$normalizedPath"
        }
    )

    if ($updated -ne $original) {
        Set-Content -Path $file.FullName -Value $updated -NoNewline
        Write-Host "Updated $($file.FullName)"
    }
}

Write-Host "Done. Normalized 'defined at ...' path separators to '$osSep'."
