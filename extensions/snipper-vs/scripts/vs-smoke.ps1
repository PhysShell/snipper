[CmdletBinding()]
param(
    [ValidateSet("Debug", "Release")]
    [string]$Configuration = "Debug",

    [string]$RootSuffix = "SnipperSmoke",

    [string]$VisualStudioInstanceId = "",

    [int]$TimeoutSeconds = 120,

    [int]$StartupDelaySeconds = 20,

    [switch]$ResetHive,

    [switch]$NoCargoBuild,

    [switch]$DeployOnly,

    [switch]$SkipLspProcessCheck,

    [switch]$KeepDevenvOpen
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-VsWherePath {
    $candidate = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path -LiteralPath $candidate) {
        return $candidate
    }

    throw "vswhere.exe was not found. Install Visual Studio Installer or pass a valid Visual Studio instance ID after vswhere is available."
}

function Get-VisualStudioInstance {
    param(
        [string]$InstanceId
    )

    $vswhere = Get-VsWherePath
    $json = & $vswhere -all -prerelease -products * -requires Microsoft.Component.MSBuild -format json
    if ($LASTEXITCODE -ne 0) {
        throw "vswhere.exe failed with exit code $LASTEXITCODE."
    }

    $instances = @($json | ConvertFrom-Json) |
        Where-Object { $_.isComplete -and $_.isLaunchable }

    if ($instances.Count -eq 0) {
        throw "No launchable Visual Studio instance with MSBuild was found."
    }

    if (-not [string]::IsNullOrWhiteSpace($InstanceId)) {
        $selected = $instances | Where-Object { $_.instanceId -eq $InstanceId } | Select-Object -First 1
        if ($null -eq $selected) {
            throw "Visual Studio instance '$InstanceId' was not found by vswhere."
        }
    }
    else {
        $selected = $instances |
            Sort-Object @{ Expression = { [version]$_.installationVersion }; Descending = $true } |
            Select-Object -First 1
    }

    $msbuildPath = Join-Path $selected.installationPath "MSBuild\Current\Bin\MSBuild.exe"
    if (-not (Test-Path -LiteralPath $msbuildPath)) {
        throw "MSBuild.exe was not found under '$($selected.installationPath)'."
    }

    $devenvPath = $selected.productPath
    if (-not (Test-Path -LiteralPath $devenvPath)) {
        throw "devenv.exe was not found at '$devenvPath'."
    }

    [pscustomobject]@{
        DisplayName      = $selected.displayName
        InstanceId       = $selected.instanceId
        InstallationPath = $selected.installationPath
        InstallationVersion = $selected.installationVersion
        MSBuildPath      = $msbuildPath
        DevenvPath       = $devenvPath
    }
}

function Invoke-CheckedProcess {
    param(
        [string]$FilePath,
        [string[]]$Arguments
    )

    Write-Host ">> $FilePath $($Arguments -join ' ')"
    & $FilePath @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "'$FilePath' failed with exit code $LASTEXITCODE."
    }
}

function Test-IsUnderPath {
    param(
        [string]$Path,
        [string]$BasePath
    )

    $fullPath = [System.IO.Path]::GetFullPath($Path).TrimEnd("\")
    $fullBasePath = [System.IO.Path]::GetFullPath($BasePath).TrimEnd("\")

    return $fullPath.Equals($fullBasePath, [System.StringComparison]::OrdinalIgnoreCase) -or
        $fullPath.StartsWith($fullBasePath + "\", [System.StringComparison]::OrdinalIgnoreCase)
}

function Get-VsHiveRoots {
    param(
        [string]$InstanceId,
        [string]$RootSuffix
    )

    $suffix = "_$InstanceId$RootSuffix"
    $basePaths = @(
        (Join-Path $env:LOCALAPPDATA "Microsoft\VisualStudio"),
        (Join-Path $env:APPDATA "Microsoft\VisualStudio")
    )

    foreach ($basePath in $basePaths) {
        if (-not (Test-Path -LiteralPath $basePath)) {
            continue
        }

        Get-ChildItem -LiteralPath $basePath -Directory -ErrorAction SilentlyContinue |
            Where-Object { $_.Name.EndsWith($suffix, [System.StringComparison]::OrdinalIgnoreCase) }
    }
}

function Remove-VsHive {
    param(
        [string]$InstanceId,
        [string]$RootSuffix
    )

    if ([string]::IsNullOrWhiteSpace($RootSuffix) -or $RootSuffix.Equals("Exp", [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to reset the shared Visual Studio Exp hive. Use a dedicated root suffix."
    }

    $roots = @(Get-VsHiveRoots -InstanceId $InstanceId -RootSuffix $RootSuffix)
    foreach ($root in $roots) {
        $basePath = Split-Path -Path $root.FullName -Parent
        if (-not (Test-IsUnderPath -Path $root.FullName -BasePath $basePath)) {
            throw "Refusing to remove unexpected path '$($root.FullName)'."
        }

        Write-Host "Removing Visual Studio smoke hive '$($root.FullName)'."
        Remove-Item -LiteralPath $root.FullName -Recurse -Force
    }
}

function Test-VsixContents {
    param(
        [string]$VsixPath,
        [string]$Configuration
    )

    if (-not (Test-Path -LiteralPath $VsixPath)) {
        throw "VSIX was not produced at '$VsixPath'."
    }

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [System.IO.Compression.ZipFile]::OpenRead($VsixPath)
    try {
        $entryNames = @($zip.Entries | ForEach-Object { $_.FullName })
        $requiredEntries = @(
            "extension.vsixmanifest",
            "Snipper.VisualStudio.dll",
            "Snipper.VisualStudio.pkgdef",
            "bin/snipper-lsp.exe"
        )

        if ($Configuration -eq "Debug") {
            $requiredEntries += "Snipper.VisualStudio.pdb"
        }

        foreach ($entry in $requiredEntries) {
            if ($entryNames -notcontains $entry) {
                throw "VSIX '$VsixPath' does not contain required entry '$entry'."
            }
        }

        $manifestEntry = $zip.GetEntry("extension.vsixmanifest")
        if ($null -eq $manifestEntry) {
            throw "VSIX manifest was not found."
        }

        $reader = [System.IO.StreamReader]::new($manifestEntry.Open())
        try {
            $manifestText = $reader.ReadToEnd()
        }
        finally {
            $reader.Dispose()
        }

        foreach ($asset in @("Microsoft.VisualStudio.VsPackage", "Microsoft.VisualStudio.MefComponent")) {
            if ($manifestText.IndexOf($asset, [System.StringComparison]::OrdinalIgnoreCase) -lt 0) {
                throw "VSIX manifest does not contain asset '$asset'."
            }
        }
    }
    finally {
        $zip.Dispose()
    }
}

function Get-DeployedSnipperExtension {
    param(
        [string]$InstanceId,
        [string]$RootSuffix,
        [string]$Configuration
    )

    $roots = @(Get-VsHiveRoots -InstanceId $InstanceId -RootSuffix $RootSuffix)
    if ($roots.Count -eq 0) {
        throw "Visual Studio hive for root suffix '$RootSuffix' was not created."
    }

    $dlls = foreach ($root in $roots) {
        Get-ChildItem -LiteralPath $root.FullName -Recurse -Filter "Snipper.VisualStudio.dll" -ErrorAction SilentlyContinue
    }

    $dll = @($dlls | Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1)
    if ($dll.Count -eq 0) {
        throw "Snipper.VisualStudio.dll was not deployed into root suffix '$RootSuffix'."
    }

    $extensionDir = Split-Path -Path $dll[0].FullName -Parent
    $requiredFiles = @(
        "Snipper.VisualStudio.pkgdef",
        "bin\snipper-lsp.exe"
    )

    if ($Configuration -eq "Debug") {
        $requiredFiles += "Snipper.VisualStudio.pdb"
    }

    foreach ($file in $requiredFiles) {
        $path = Join-Path $extensionDir $file
        if (-not (Test-Path -LiteralPath $path)) {
            throw "Deployed extension is missing '$file' at '$extensionDir'."
        }
    }

    [pscustomobject]@{
        Directory = $extensionDir
        Assembly  = $dll[0].FullName
        LspPath   = Join-Path $extensionDir "bin\snipper-lsp.exe"
    }
}

function Get-DevenvProcessesForRootSuffix {
    param(
        [string]$RootSuffix
    )

    Get-CimInstance Win32_Process -Filter "Name = 'devenv.exe'" -ErrorAction SilentlyContinue |
        Where-Object {
            $_.CommandLine -and
            $_.CommandLine.IndexOf("/rootsuffix", [System.StringComparison]::OrdinalIgnoreCase) -ge 0 -and
            $_.CommandLine.IndexOf($RootSuffix, [System.StringComparison]::OrdinalIgnoreCase) -ge 0
        }
}

function Wait-Until {
    param(
        [scriptblock]$Probe,
        [int]$TimeoutSeconds,
        [string]$Description
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
        $result = & $Probe
        if ($result) {
            return $result
        }

        Start-Sleep -Milliseconds 1000
    } while ((Get-Date) -lt $deadline)

    throw "Timed out after $TimeoutSeconds seconds waiting for $Description."
}

function Get-LoadedSnipperModule {
    param(
        [int]$DevenvProcessId
    )

    try {
        $process = Get-Process -Id $DevenvProcessId -ErrorAction Stop
        @($process.Modules) |
            Where-Object { $_.FileName -like "*Snipper.VisualStudio.dll" } |
            Select-Object -First 1
    }
    catch {
        $null
    }
}

function Get-SnipperLspProcess {
    param(
        [int]$DevenvProcessId,
        [string]$ExtensionDirectory
    )

    $expectedLspPath = Join-Path $ExtensionDirectory "bin\snipper-lsp.exe"
    Get-CimInstance Win32_Process -Filter "Name = 'snipper-lsp.exe'" -ErrorAction SilentlyContinue |
        Where-Object {
            $_.ParentProcessId -eq $DevenvProcessId -or
            ($_.CommandLine -and $_.CommandLine.IndexOf($expectedLspPath, [System.StringComparison]::OrdinalIgnoreCase) -ge 0)
        } |
        Select-Object -First 1
}

function Write-ActivityLogHints {
    param(
        [string]$ActivityLogPath
    )

    if ([string]::IsNullOrWhiteSpace($ActivityLogPath) -or -not (Test-Path -LiteralPath $ActivityLogPath)) {
        return
    }

    $matches = Select-String -Path $ActivityLogPath -Pattern "Snipper", "snipper-lsp", "LanguageServer", "MefComponent" -ErrorAction SilentlyContinue |
        Select-Object -Last 25

    if ($matches) {
        Write-Host "ActivityLog hints:"
        foreach ($match in $matches) {
            Write-Host ("  {0}: {1}" -f $match.LineNumber, $match.Line.Trim())
        }
    }
}

function Test-ActivityLogRequestsRestart {
    param(
        [string]$ActivityLogPath
    )

    if ([string]::IsNullOrWhiteSpace($ActivityLogPath) -or -not (Test-Path -LiteralPath $ActivityLogPath)) {
        return $false
    }

    try {
        [xml]$activityLog = Get-Content -LiteralPath $ActivityLogPath
        $restartEntry = $activityLog.activity.entry |
            Where-Object { $_.description -like "*Forcing VS restart*" } |
            Select-Object -First 1

        return $null -ne $restartEntry
    }
    catch {
        return $false
    }
}

function Get-SnipperPackageLoadEvidence {
    param(
        [int]$DevenvProcessId,
        [string]$ActivityLogPath
    )

    $module = Get-LoadedSnipperModule -DevenvProcessId $DevenvProcessId
    if ($module) {
        return [pscustomobject]@{
            Kind = "Module"
            Path = $module.FileName
        }
    }

    if (-not [string]::IsNullOrWhiteSpace($ActivityLogPath) -and (Test-Path -LiteralPath $ActivityLogPath)) {
        try {
            [xml]$activityLog = Get-Content -LiteralPath $ActivityLogPath
            $entry = $activityLog.activity.entry |
                Where-Object { $_.description -eq "End package load [SnipperPackage]" } |
                Select-Object -Last 1

            if ($entry) {
                return [pscustomobject]@{
                    Kind = "ActivityLog"
                    Path = "$ActivityLogPath#$($entry.record)"
                }
            }
        }
        catch {
            return $null
        }
    }

    return $null
}

if ([string]::IsNullOrWhiteSpace($RootSuffix) -or $RootSuffix.Equals("Exp", [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Use a dedicated root suffix. This smoke test intentionally refuses the shared 'Exp' hive."
}

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..")).Path
$projectPath = Join-Path $repoRoot "extensions\snipper-vs\Snipper.VisualStudio\Snipper.VisualStudio.csproj"
$vsixPath = Join-Path $repoRoot "extensions\snipper-vs\Snipper.VisualStudio\bin\$Configuration\net472\Snipper.VisualStudio.vsix"
$runRoot = Join-Path $repoRoot "scratch\vs-smoke"
$runDirectory = Join-Path $runRoot (Get-Date -Format "yyyyMMdd-HHmmss")
New-Item -ItemType Directory -Path $runDirectory -Force | Out-Null

$activityLogPath = Join-Path $runDirectory "ActivityLog.xml"
$devenvProcess = $null
$deployedExtension = $null

try {
    $vs = Get-VisualStudioInstance -InstanceId $VisualStudioInstanceId
    Write-Host "Using $($vs.DisplayName) $($vs.InstallationVersion) [$($vs.InstanceId)]."

    if ($ResetHive) {
        Remove-VsHive -InstanceId $vs.InstanceId -RootSuffix $RootSuffix
    }

    $existingDevenv = @(Get-DevenvProcessesForRootSuffix -RootSuffix $RootSuffix)
    if ($existingDevenv.Count -gt 0) {
        throw "A devenv.exe process is already running with root suffix '$RootSuffix'. Close it before running the smoke test."
    }

    $cargoBuild = if ($NoCargoBuild) { "false" } else { "true" }
    Invoke-CheckedProcess -FilePath $vs.MSBuildPath -Arguments @(
        $projectPath,
        "/t:Build;DeployVsixExtensionFiles",
        "/p:Configuration=$Configuration",
        "/p:DeployExtension=true",
        "/p:CreateVsixContainer=true",
        "/p:VSSDKTargetPlatformRegRootSuffix=$RootSuffix",
        "/p:DeployTargetInstanceId=$($vs.InstanceId)",
        "/p:SnipperLspCargoBuild=$cargoBuild",
        "/m:1",
        "/v:minimal"
    )

    Test-VsixContents -VsixPath $vsixPath -Configuration $Configuration
    $deployedExtension = Get-DeployedSnipperExtension -InstanceId $vs.InstanceId -RootSuffix $RootSuffix -Configuration $Configuration

    Write-Host "VSIX: $vsixPath"
    Write-Host "Deployed extension: $($deployedExtension.Directory)"

    if ($DeployOnly) {
        Write-Host "Visual Studio deploy smoke test passed."
        return
    }

    $smokeProjectDirectory = Join-Path $runDirectory "SmokeApp"
    New-Item -ItemType Directory -Path $smokeProjectDirectory -Force | Out-Null

    $solutionPath = Join-Path $runDirectory "SmokeSolution.slnx"
    @(
        "<Solution>",
        "  <Project Path=`"SmokeApp/SmokeApp.csproj`" />",
        "</Solution>"
    ) | Set-Content -LiteralPath $solutionPath -Encoding UTF8

    $smokeProjectPath = Join-Path $smokeProjectDirectory "SmokeApp.csproj"
    @(
        "<Project Sdk=`"Microsoft.NET.Sdk`">",
        "  <PropertyGroup>",
        "    <OutputType>Exe</OutputType>",
        "    <TargetFramework>net8.0</TargetFramework>",
        "    <Nullable>enable</Nullable>",
        "  </PropertyGroup>",
        "</Project>"
    ) | Set-Content -LiteralPath $smokeProjectPath -Encoding UTF8

    $smokeFilePath = Join-Path $smokeProjectDirectory "Program.cs"
    @(
        "namespace Snipper.VisualStudio.Smoke;",
        "",
        "internal static class Smoke",
        "{",
        "    private static void Main()",
        "    {",
        "        var values = new[] { 1, 2, 3 };",
        "        values.",
        "    }",
        "}"
    ) | Set-Content -LiteralPath $smokeFilePath -Encoding UTF8

    $maxLaunchAttempts = 2
    for ($launchAttempt = 1; $launchAttempt -le $maxLaunchAttempts; $launchAttempt++) {
        $activityLogPath = if ($launchAttempt -eq 1) {
            Join-Path $runDirectory "ActivityLog.xml"
        }
        else {
            Join-Path $runDirectory "ActivityLog.$launchAttempt.xml"
        }

        $launchArguments = @(
            "/rootsuffix",
            $RootSuffix,
            "/log",
            ('"' + $activityLogPath + '"'),
            ('"' + $solutionPath + '"')
        )

        Write-Host "Launching devenv.exe with root suffix '$RootSuffix' (attempt $launchAttempt)."
        $devenvProcess = Start-Process -FilePath $vs.DevenvPath -ArgumentList $launchArguments -PassThru

        try {
            [void]$devenvProcess.WaitForInputIdle([Math]::Min($TimeoutSeconds, 60) * 1000)
        }
        catch {
            # Some VS startup paths do not expose input-idle reliably; process/module polling below is authoritative.
        }

        Start-Sleep -Seconds 5
        if (Get-Process -Id $devenvProcess.Id -ErrorAction SilentlyContinue) {
            break
        }

        if ($launchAttempt -lt $maxLaunchAttempts -and (Test-ActivityLogRequestsRestart -ActivityLogPath $activityLogPath)) {
            Write-Host "Visual Studio requested a restart after extension registration; relaunching."
            $devenvProcess = $null
            continue
        }

        throw "devenv.exe exited before the smoke test could open the C# probe file."
    }

    if ($null -eq $devenvProcess -or -not (Get-Process -Id $devenvProcess.Id -ErrorAction SilentlyContinue)) {
        throw "devenv.exe is not running after launch."
    }

    if ($StartupDelaySeconds -gt 0) {
        Write-Host "Waiting $StartupDelaySeconds seconds for Visual Studio startup services."
        Start-Sleep -Seconds $StartupDelaySeconds
    }

    $loadEvidence = Wait-Until -TimeoutSeconds $TimeoutSeconds -Description "SnipperPackage to load in devenv.exe" -Probe {
        if (-not (Get-Process -Id $devenvProcess.Id -ErrorAction SilentlyContinue)) {
            throw "devenv.exe exited before Snipper.VisualStudio.dll was loaded."
        }

        Get-SnipperPackageLoadEvidence -DevenvProcessId $devenvProcess.Id -ActivityLogPath $activityLogPath
    }
    Write-Host "Loaded package evidence: $($loadEvidence.Kind) $($loadEvidence.Path)"

    if (-not $SkipLspProcessCheck) {
        $openFileCommand = "File.OpenFile $smokeFilePath"
        $openFileArguments = "/rootsuffix $RootSuffix /command `"$openFileCommand`""
        $commandProcess = Start-Process -FilePath $vs.DevenvPath -ArgumentList $openFileArguments -PassThru

        if (-not $commandProcess.WaitForExit(60000)) {
            Stop-Process -Id $commandProcess.Id -Force -ErrorAction SilentlyContinue
            throw "Timed out while sending File.OpenFile to the Visual Studio smoke instance."
        }

        Start-Sleep -Seconds 3

        $lspProcess = Wait-Until -TimeoutSeconds $TimeoutSeconds -Description "snipper-lsp.exe to start from the deployed extension" -Probe {
            if (-not (Get-Process -Id $devenvProcess.Id -ErrorAction SilentlyContinue)) {
                throw "devenv.exe exited before snipper-lsp.exe was started."
            }

            Get-SnipperLspProcess -DevenvProcessId $devenvProcess.Id -ExtensionDirectory $deployedExtension.Directory
        }

        Write-Host "Started LSP: pid=$($lspProcess.ProcessId) command=$($lspProcess.CommandLine)"
    }

    Write-Host "ActivityLog: $activityLogPath"
    Write-Host "Visual Studio smoke test passed."
}
catch {
    Write-Host "Visual Studio smoke test failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-ActivityLogHints -ActivityLogPath $activityLogPath
    throw
}
finally {
    if ($null -ne $devenvProcess -and -not $KeepDevenvOpen) {
        $runningDevenv = Get-Process -Id $devenvProcess.Id -ErrorAction SilentlyContinue
        if ($runningDevenv) {
            Write-Host "Closing devenv.exe pid=$($runningDevenv.Id)."
            [void]$runningDevenv.CloseMainWindow()
            if (-not $runningDevenv.WaitForExit(15000)) {
                Stop-Process -Id $runningDevenv.Id -Force -ErrorAction SilentlyContinue
            }
        }
    }
}
