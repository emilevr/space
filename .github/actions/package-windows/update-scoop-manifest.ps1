[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$Target,
    [Parameter(Mandatory=$true)][string]$ArchiveFileHash,
    [Parameter(Mandatory=$true)][string]$Version
)

$artifactsDir = 'artifacts/pages/scoop'
$sourceTemplatePath = 'install/scoop/space.json'

New-Item -Type Directory -Path $artifactsDir -Force | Out-Null

$content = Get-Content $sourceTemplatePath

$result = $content | Select-String -Pattern '0\.0\.0'
foreach ($match in $result.Matches) {
    $content = $content -replace [regex]::Escape($match.Value), $Version
}

$result = $content | Select-String -Pattern '000'
foreach ($match in $result.Matches) {
    $content = $content -replace [regex]::Escape($match.Value), $ArchiveFileHash
}

$content | Out-File -FilePath "$artifactsDir/space.json" -Encoding utf8 -Force
