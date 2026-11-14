param([Parameter(Mandatory)][ValidateSet('x86_64-pc-windows-msvc','aarch64-pc-windows-msvc')][string]$Target,[Parameter(Mandatory)][ValidateSet('x64','arm64')][string]$Architecture,[Parameter(Mandatory)][string]$Version)
$ErrorActionPreference='Stop'
if ($Version -notmatch '^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$') {throw 'Invalid version'}
if(($Architecture -eq 'arm64') -ne ($Target -eq 'aarch64-pc-windows-msvc')){throw 'Architecture/target mismatch'}
$taskWorkspace=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskBinaryRoot=Join-Path $taskWorkspace "target/$Target/release"
$taskOutput=Join-Path $taskWorkspace "dist/release-$Architecture"
if(Test-Path -LiteralPath $taskOutput){throw 'Release output exists; retain it elsewhere before packaging again.'}
New-Item -ItemType Directory -Path $taskOutput | Out-Null
$taskStage=Join-Path $taskWorkspace ("target/portable-stage-"+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $taskStage | Out-Null
function Assert-OwnMachine([string]$Path,[int]$Expected) {
    $taskStream=[IO.File]::OpenRead($Path)
    try {
        $taskReader=[IO.BinaryReader]::new($taskStream)
        if($taskReader.ReadUInt16() -ne 0x5A4D){throw 'Missing PE DOS header'}
        $taskStream.Position=0x3C;$taskOffset=$taskReader.ReadUInt32();$taskStream.Position=$taskOffset
        if($taskReader.ReadUInt32() -ne 0x4550 -or $taskReader.ReadUInt16() -ne $Expected){throw "Unexpected architecture: $Path"}
    } finally {$taskStream.Dispose()}
    if((Get-AuthenticodeSignature -LiteralPath $Path).Status -ne 'NotSigned'){throw 'This release policy requires unsigned Windows binaries.'}
}
$taskMachine=if($Architecture -eq 'arm64'){0xAA64}else{0x8664}
foreach($taskName in @('instancedeck.exe','ideck.exe')) {
    $taskSource=Join-Path $taskBinaryRoot $taskName
    Assert-OwnMachine $taskSource $taskMachine
    Copy-Item -LiteralPath $taskSource -Destination (Join-Path $taskStage $taskName)
}
foreach($taskName in @('LICENSE','PRIVACY.md','SECURITY.md','CODE_SIGNING_POLICY.md')) {Copy-Item -LiteralPath (Join-Path $taskWorkspace $taskName) -Destination $taskStage}
Copy-Item -LiteralPath (Join-Path $taskWorkspace 'docs/portable.md') -Destination (Join-Path $taskStage 'README.md')
Set-Content -LiteralPath (Join-Path $taskStage 'portable-mode.txt') -Value 'InstanceDeck portable distribution. Profile storage uses the configured root.' -Encoding utf8
$taskInstallers=@(Get-ChildItem -LiteralPath (Join-Path $taskBinaryRoot 'bundle/nsis') -Filter "*_${Version}_*-setup.exe" -File)
if($taskInstallers.Count -ne 1){throw 'Expected exactly one NSIS installer'}
if((Get-AuthenticodeSignature -LiteralPath $taskInstallers[0].FullName).Status -ne 'NotSigned'){throw 'Expected unsigned NSIS installer'}
Copy-Item -LiteralPath $taskInstallers[0].FullName -Destination (Join-Path $taskOutput "InstanceDeck_${Version}_${Architecture}-setup.exe")
Compress-Archive -LiteralPath (Get-ChildItem -LiteralPath $taskStage -File).FullName -DestinationPath (Join-Path $taskOutput "InstanceDeck_${Version}_${Architecture}-portable.zip") -CompressionLevel Optimal
$taskFiles=@(Get-ChildItem -LiteralPath $taskOutput -File | Sort-Object Name)
$taskHashes=@($taskFiles | ForEach-Object {"$((Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant())  $($_.Name)"})
$taskHashes | Set-Content -LiteralPath (Join-Path $taskOutput "SHA256SUMS-$Architecture.txt") -Encoding utf8
$taskCommit=git -C $taskWorkspace rev-parse HEAD
@{version=$Version;architecture=$Architecture;target=$Target;commit=$taskCommit.Trim();authenticode='unsigned';machine=$taskMachine;files=@($taskFiles|ForEach-Object{@{name=$_.Name;size=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()}})}|ConvertTo-Json -Depth 8|Set-Content -LiteralPath (Join-Path $taskOutput "build-$Architecture.json") -Encoding utf8
Write-Output "Packaged $Architecture installer and portable archive. No installer was executed."
