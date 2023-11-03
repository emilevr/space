[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$Target
)

New-Item -Type Directory -Path './artifacts/dist' -Force | Out-Null

$binFileNames = @('space', 'space-bench')
foreach ($fileName in $binFileNames) {
    $binFilePath = "./target/release/${fileName}.exe"
    $archiveFilePath = "./artifacts/dist/${fileName}-${Target}.zip"
    $archiveHashFilePath = "${archiveFilePath}.sha256"

    Write-Host "Compressing ${binFilePath} to ${archiveFilePath}"
    $compress = @{
        Path = $binFilePath
        DestinationPath = $archiveFilePath
        CompressionLevel = "Optimal"
    }
    Compress-Archive @compress

    Write-Host "Calculating SHA256 of ${archiveFilePath} to ${archiveHashFilePath}"
    $hash = Get-FileHash -Path $archiveFilePath -Algorithm SHA256 `
        | Select-Object @{Label='Hash';Expression={$_.Hash.ToLower()}} `
        | Select-Object -ExpandProperty Hash
    $hash > $archiveHashFilePath

    Write-Host "ARCHIVE_FILE_HASH=$hash" >> "${env:GITHUB_ENV}"
}
