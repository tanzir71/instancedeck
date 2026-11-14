param([Parameter(Mandatory)][string]$Tag,[string]$KeyPath=(Join-Path $env:LOCALAPPDATA 'instancedeck-updater.key'))
$ErrorActionPreference='Stop'
if($Tag -notmatch '^v\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$'){throw 'Invalid release tag'}
if([IO.Path]::GetFileName($KeyPath) -ne 'instancedeck-updater.key' -or -not [IO.Path]::IsPathFullyQualified($KeyPath)){throw 'Use the locally generated InstanceDeck updater key path'}
if((Get-Item -LiteralPath $KeyPath).Length -gt 16384){throw 'Unexpected project key size'}
$taskVersion=$Tag.Substring(1)
$taskRepo='tanzir71/instancedeck'
$taskRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
Push-Location $taskRoot
try {
    $taskRelease=gh release view $Tag --repo $taskRepo --json isDraft,assets,createdAt,body | ConvertFrom-Json
    if($LASTEXITCODE -ne 0 -or -not $taskRelease.isDraft){throw 'Only an existing draft can be finalized'}
    $taskCommit=(git rev-parse "$Tag^{commit}").Trim()
    if($LASTEXITCODE -ne 0){throw 'Fetch the release tag before finalizing'}
    $taskBase=@('build-x64.json','build-arm64.json')
    foreach($taskArch in @('x64','arm64')){$taskBase+="InstanceDeck_${taskVersion}_${taskArch}-setup.exe";$taskBase+="InstanceDeck_${taskVersion}_${taskArch}-portable.zip"}
    $taskAllowed=$taskBase+@('SHA256SUMS.txt','latest.json','latest.json.sig',"InstanceDeck_${taskVersion}_x64-setup.exe.sig","InstanceDeck_${taskVersion}_arm64-setup.exe.sig")
    $taskNames=@($taskRelease.assets|ForEach-Object{$_.name})
    if(@($taskBase|Where-Object{$_ -notin $taskNames}).Count -gt 0 -or @($taskNames|Where-Object{$_ -notin $taskAllowed}).Count -gt 0){throw 'Draft has missing or unexpected assets'}
    $taskWork=Join-Path $taskRoot ('target/release-finalization-'+[guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $taskWork | Out-Null
    foreach($taskName in $taskNames){gh release download $Tag --repo $taskRepo --pattern $taskName --dir $taskWork;if($LASTEXITCODE -ne 0){throw 'Could not download draft artifacts'}}
    foreach($taskLine in Get-Content -LiteralPath (Join-Path $taskWork 'SHA256SUMS.txt')) {
        if($taskLine -notmatch '^([a-fA-F0-9]{64})  ([A-Za-z0-9_.-]+)$'){throw 'Invalid checksum entry'}
        $taskHash=$Matches[1];$taskName=$Matches[2]
        if($taskName -notin $taskAllowed -or $taskName -eq 'SHA256SUMS.txt'){throw 'Checksum escaped release namespace'}
        if((Get-FileHash -LiteralPath (Join-Path $taskWork $taskName) -Algorithm SHA256).Hash -ne $taskHash){throw "CI checksum mismatch: $taskName"}
    }
    foreach($taskArch in @('x64','arm64')) {
        $taskReport=Get-Content -Raw -LiteralPath (Join-Path $taskWork "build-$taskArch.json")|ConvertFrom-Json
        if($taskReport.commit -ne $taskCommit -or $taskReport.version -ne $taskVersion -or $taskReport.architecture -ne $taskArch -or $taskReport.authenticode -ne 'unsigned'){throw 'CI report does not match release tag/version/architecture'}
        foreach($taskFile in $taskReport.files){if($taskFile.name -notin $taskBase -or (Get-FileHash -LiteralPath (Join-Path $taskWork $taskFile.name) -Algorithm SHA256).Hash -ne $taskFile.sha256){throw 'Build report checksum mismatch'}}
    }
    function Sign-OwnArtifact([string]$Path) {
        # Only the official signer opens the locally generated key. Its contents
        # are never read by this script or sent to GitHub.
        & pnpm --filter @instancedeck/desktop tauri signer sign --private-key-path $KeyPath '--password=' --app-version $taskVersion $Path *> ($Path+'.signing.log')
        if($LASTEXITCODE -ne 0){throw 'Local project-integrity signing failed; output is retained locally'}
        & cargo run --locked -p ideck-cli -- verify-release-artifact $Path --app-version $taskVersion
        if($LASTEXITCODE -ne 0){throw 'Offline integrity/version verification failed'}
    }
    $taskPlatforms=@{}
    foreach($taskArch in @('x64','arm64')) {
        $taskName="InstanceDeck_${taskVersion}_${taskArch}-setup.exe"
        $taskPath=Join-Path $taskWork $taskName
        if((Get-AuthenticodeSignature -LiteralPath $taskPath).Status -ne 'NotSigned'){throw 'Windows binaries must remain unsigned'}
        Sign-OwnArtifact $taskPath
        $taskPlatform=if($taskArch -eq 'arm64'){'windows-aarch64'}else{'windows-x86_64'}
        $taskPlatforms[$taskPlatform]=@{url="https://github.com/$taskRepo/releases/download/$Tag/$taskName";signature=(Get-Content -Raw -LiteralPath ($taskPath+'.sig')).Trim()}
    }
    $taskFeed=Join-Path $taskWork 'latest.json'
    @{version=$taskVersion;pub_date=$taskRelease.createdAt;notes=$taskRelease.body;platforms=$taskPlatforms}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath $taskFeed -Encoding utf8
    Sign-OwnArtifact $taskFeed
    # Negative tests operate only on a disposable copy of our own installer.
    $taskOriginal=Join-Path $taskWork "InstanceDeck_${taskVersion}_x64-setup.exe"
    & cargo run --locked -p ideck-cli -- verify-release-artifact $taskOriginal --app-version '999.0.0' 2>$null
    if($LASTEXITCODE -eq 0){throw 'Signed-version mismatch was accepted'}
    $taskTamper=Join-Path $taskWork 'tampered-own-fixture'
    New-Item -ItemType Directory -Path $taskTamper | Out-Null
    $taskCopy=Join-Path $taskTamper ([IO.Path]::GetFileName($taskOriginal))
    Copy-Item -LiteralPath $taskOriginal -Destination $taskCopy
    Copy-Item -LiteralPath ($taskOriginal+'.sig') -Destination ($taskCopy+'.sig')
    $taskStream=[IO.File]::Open($taskCopy,[IO.FileMode]::Open,[IO.FileAccess]::ReadWrite)
    try {$taskStream.Position=$taskStream.Length-1;$taskByte=$taskStream.ReadByte();$taskStream.Position=$taskStream.Length-1;$taskStream.WriteByte($taskByte -bxor 1)}finally{$taskStream.Dispose()}
    & cargo run --locked -p ideck-cli -- verify-release-artifact $taskCopy --app-version $taskVersion 2>$null
    if($LASTEXITCODE -eq 0){throw 'Tampered own-installer fixture was accepted'}
    $taskUpload=@($taskAllowed|Where-Object{$_ -ne 'SHA256SUMS.txt' -and (Test-Path -LiteralPath (Join-Path $taskWork $_))})
    $taskUpload|Sort-Object|ForEach-Object{"$((Get-FileHash -LiteralPath (Join-Path $taskWork $_) -Algorithm SHA256).Hash.ToLowerInvariant())  $_"}|Set-Content -LiteralPath (Join-Path $taskWork 'SHA256SUMS.txt') -Encoding utf8
    $taskUpload+='SHA256SUMS.txt'
    gh release upload $Tag --repo $taskRepo @($taskUpload|ForEach-Object{Join-Path $taskWork $_}) --clobber
    if($LASTEXITCODE -ne 0){throw 'Could not upload public signatures/feed/checksums'}
    @{tag=$Tag;commit=$taskCommit;architectures=@('x64','arm64');authenticode='unsigned';installerSignaturesVerified=$true;feedSignatureVerified=$true;signedVersionMismatchRejected=$true;tamperedOwnInstallerRejected=$true;privateKeyTransmitted=$false;installerExecuted=$false;workingDirectory=[IO.Path]::GetRelativePath($taskRoot,$taskWork)}|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $taskRoot 'artifacts/p7/release-verification.json') -Encoding utf8
    Write-Output "Finalized $Tag draft. Uploaded public integrity signatures and feed; no installer was executed."
} finally {Pop-Location}
