[CmdletBinding()]
param()

function Install-LinuxOrMacOS {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory=$true)][string]$ArchiveFilename
    )

    $installDirPath = "/usr/local/bin"
    Write-Host "👷 Installing space CLI to $installDirPath." -NoNewline
    Write-Host " This will typically require root access." -ForegroundColor Cyan

    $latestReleaseArchive = "https://github.com/emilevr/space/releases/latest/download/$ArchiveFilename"
    $archiveFilePath = "$HOME/$ArchiveFilename"

    $extractDirPath = '$HOME'
    $extractBinaryPath = "$extractDirPath/space"
    $installedBinaryPath = "$installDirPath/space"

    try {
        Write-Host "⭳⭳ Downloading the latest version from $latestReleaseArchive"
        Invoke-WebRequest -Uri $latestReleaseArchive -OutFile $archiveFilePath | Out-Null

        Write-Host "👷 Extracting archive $archiveFilePath to $extractDirPath"
        tar -xzf $archiveFilePath -C $extractDirPath/

        Write-Host "👷 Making space executable"
        chmod +x $extractBinaryPath

        Write-Host "👷 Moving $extractBinaryPath to $installedBinaryPath." -NoNewline
        Write-Host " This will typically fail without root access." -ForegroundColor Cyan
        Move-Item -Path $extractBinaryPath -Destination $installedBinaryPath -Force

        return $true
    } finally {
        Write-Host "👷 Cleaning up:"
        Write-Host "   Removing downloaded archive file $archiveFilePath"
        Remove-Item -Path $archiveFilePath -Force -ErrorAction SilentlyContinue
    }

    return $false
}

function Install-Windows {
    [CmdletBinding()]
    param()

    $installDirPath = Join-Path -Path $HOME -ChildPath 'space'

    Write-Host "Installing space CLI to $installDirPath"

    New-Item -Type Directory -Path $installDirPath -Force | Out-Null

    $archiveFilename = 'space-x86_64-pc-windows-msvc.zip'
    $archiveFilePath = "$HOME/$archiveFilename"
    $latestReleaseArchive = "https://github.com/emilevr/space/releases/latest/download/$archiveFilename"
    $pathSeparator = ';'

    try {
        Write-Host "⭳⭳ Downloading the latest version from $latestReleaseArchive"
        Invoke-WebRequest -Uri $latestReleaseArchive -OutFile $archiveFilePath | Out-Null

        Write-Host "👷 Extracting archive $archiveFilePath to $installDirPath"
        Expand-Archive -Path $archiveFilePath -DestinationPath $installDirPath -Force

        Write-Host "👷 Adding the space CLI to the path"
        $escapedPath = [regex]::Escape($installDirPath)
        $result = $env:PATH | Select-String -Pattern "$escapedPath\\?"
        if ($result.Matches.Success) {
            Write-Host "   The PATH variable already contains the required entry. Not modifying PATH."
        } else {
            Write-Host "   Updating PowerShell session path environment variable"
            $pathEntries = $env:PATH -split $pathSeparator
            $env:PATH = ($pathEntries + $installDirPath) -join $pathSeparator

            Write-Host "   Updating user path environment variable"
            [System.Environment]::SetEnvironmentVariable('PATH', $env:PATH, [System.EnvironmentVariableTarget]::User)
        }

        return $true
    } finally {
        Write-Host "👷 Cleaning up:"
        Write-Host "   Removing downloaded archive file $archiveFilePath"
        Remove-Item -Path $archiveFilePath -Force -ErrorAction SilentlyContinue
    }

    return $false
}

$ErrorActionPreference = "Stop"

if ($IsLinux) {
    $installed = Install-LinuxOrMacOS -ArchiveFilename 'space-x86_64-unknown-linux-musl.tar.gz'
} elseif ($IsMacOS) {
    $installed = Install-LinuxOrMacOS -ArchiveFilename 'space-x86_64-apple-darwin.tar.gz'
} elseif ($IsWindows) {
    $installed = Install-Windows
} else {
    Write-Error "❌ Only Linux, MacOS and Windows installs are supported."
}

if ($installed) {
    Write-Host "✔️ Installation completed. Run 'space --help' to see a list of available options." -ForegroundColor Green
}
