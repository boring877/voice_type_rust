param(
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$TauriRoot = Join-Path $ProjectRoot "src-tauri"
$ReleaseDir = Join-Path $TauriRoot "target\release"
$ExePath = Join-Path $ReleaseDir "voice_type_tauri.exe"
$IconsDir = Join-Path $TauriRoot "icons"
$OutputDir = Join-Path $ProjectRoot "msix-output"
$StagingDir = Join-Path $OutputDir "staging"

if (-not (Test-Path $ExePath)) {
    Write-Error "voice_type_tauri.exe not found at $ExePath. Run 'bun run tauri build' first."
    exit 1
}

if ($Version -eq "") {
    $Version = (Get-Content (Join-Path $TauriRoot "tauri.conf.json") | ConvertFrom-Json).version
}

$VersionParts = $Version -split "\."
$Major = [int]$VersionParts[0]
$Minor = [int]$VersionParts[1]
$Patch = [int]$VersionParts[2]
$PackageVersion = "{0}.{1}.{2}.0" -f $Major, $Minor, $Patch

$IdentityName = "Boring877.VoiceType"
$Publisher = "CN=AA95BB46-4C4F-4A69-A44B-1C3DA80D5C2B"
$PublisherId = "kg07y93afj4jj"
$ProcessorArchitecture = "x64"
$MakeAppx = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\makeappx.exe"

$PackageName = "{0}_{1}_{2}__{3}.msix" -f $IdentityName, $PackageVersion, $ProcessorArchitecture, $PublisherId
$OutputPath = Join-Path $OutputDir $PackageName

Write-Host "Building MSIX: $PackageName" -ForegroundColor Cyan

if (Test-Path $StagingDir) { Remove-Item $StagingDir -Recurse -Force }
New-Item -ItemType Directory -Path $StagingDir -Force | Out-Null

$AppxManifest = @'
<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:uap3="http://schemas.microsoft.com/appx/manifest/uap/windows10/3"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
  IgnorableNamespaces="uap uap3 rescap">
  <Identity Name="{0}" Publisher="{1}" Version="{2}" ProcessorArchitecture="{3}" />
  <Properties>
    <DisplayName>Voice Type</DisplayName>
    <PublisherDisplayName>Boring877</PublisherDisplayName>
    <Description>Push-to-talk speech to text powered by Groq Whisper</Description>
    <Logo>{4}</Logo>
  </Properties>
  <Resources>
    <Resource Language="en-US" />
    <Resource uap:Scale="100" />
    <Resource uap:Scale="125" />
    <Resource uap:Scale="150" />
    <Resource uap:Scale="200" />
    <Resource uap:Scale="400" />
  </Resources>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.19041.0" />
  </Dependencies>
  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
  <Applications>
    <Application Id="VoiceType" Executable="voice_type_tauri.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements
        DisplayName="Voice Type"
        Description="Push-to-talk speech to text"
        BackgroundColor="transparent"
        Square150x150Logo="{5}"
        Square44x44Logo="{6}"
      />
    </Application>
  </Applications>
</Package>
'@ -f $IdentityName, $Publisher, $PackageVersion, $ProcessorArchitecture, "images\StoreLogo.png", "images\Square150x150Logo.png", "images\Square44x44Logo.png"

$AppxManifest | Out-File -FilePath (Join-Path $StagingDir "AppxManifest.xml") -Encoding UTF8

$ImagesDir = Join-Path $StagingDir "images"
New-Item -ItemType Directory -Path $ImagesDir -Force | Out-Null

Copy-Item (Join-Path $IconsDir "StoreLogo.png") $ImagesDir
Copy-Item (Join-Path $IconsDir "Square150x150Logo.png") $ImagesDir
Copy-Item (Join-Path $IconsDir "Square44x44Logo.png") $ImagesDir

Copy-Item $ExePath $StagingDir

if (Test-Path $OutputPath) { Remove-Item $OutputPath -Force }

& $MakeAppx pack /d $StagingDir /p $OutputPath /o

if ($LASTEXITCODE -eq 0) {
    Write-Host "`nMSIX created: $OutputPath" -ForegroundColor Green
    Write-Host "Upload this file to Microsoft Partner Center to update your Store listing." -ForegroundColor Yellow
} else {
    Write-Error "makeappx.exe failed with exit code $LASTEXITCODE"
    exit 1
}
