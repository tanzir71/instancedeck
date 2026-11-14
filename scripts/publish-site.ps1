$ErrorActionPreference='Stop'
$taskRoot=[IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskDist=Join-Path $taskRoot 'apps/site/dist'
foreach($taskName in @('index.html','.nojekyll','storybook/index.html')) {
    if(-not(Test-Path -LiteralPath (Join-Path $taskDist $taskName) -PathType Leaf)){throw "Build and package the site first: $taskName is missing"}
}
$taskEntries=@(Get-ChildItem -LiteralPath $taskDist -Recurse -Force)
if($taskEntries|Where-Object{($_.Attributes -band [IO.FileAttributes]::ReparsePoint) -or $_.Name -eq '.git'}){throw 'Prebuilt site cannot contain links or Git metadata'}
$taskRepo='tanzir71/instancedeck'
$taskRemote="https://github.com/$taskRepo.git"
$taskSource=(git -C $taskRoot rev-parse HEAD).Trim()
if($LASTEXITCODE -ne 0){throw 'Cannot identify source commit'}
$taskStage=Join-Path $taskRoot ('target/site-publish-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $taskStage | Out-Null
git -C $taskStage init --quiet --initial-branch gh-pages
if($LASTEXITCODE -ne 0){throw 'Could not initialize isolated publication directory'}
git -C $taskStage remote add origin $taskRemote
git -C $taskStage config core.autocrlf false
foreach($taskSetting in @('user.name','user.email')) {
    $taskIdentity=git -C $taskRoot config $taskSetting
    if($LASTEXITCODE -ne 0){throw 'Configure the source repository Git author first'}
    git -C $taskStage config $taskSetting $taskIdentity
}
$taskExisting=git -C $taskRoot ls-remote --heads $taskRemote gh-pages
if($LASTEXITCODE -ne 0){throw 'Could not read publication branch'}
if($taskExisting) {
    git -C $taskStage fetch --depth 1 origin gh-pages
    if($LASTEXITCODE -ne 0){throw 'Could not fetch retained publication history'}
    # A soft reset gives the new commit its existing parent, with an empty
    # index/worktree. Old files remain in history; nothing is hard-deleted.
    git -C $taskStage reset --soft FETCH_HEAD
    if($LASTEXITCODE -ne 0){throw 'Could not preserve publication history'}
}
Get-ChildItem -LiteralPath $taskDist -Force | ForEach-Object {Copy-Item -LiteralPath $_.FullName -Destination $taskStage -Recurse -Force}
Set-Content -LiteralPath (Join-Path $taskStage '.gitattributes') -Value '* -text' -Encoding ascii
@{sourceCommit=$taskSource;builtLocally=$true;publishedAt=[DateTime]::UtcNow.ToString('o')}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $taskStage 'build-info.json') -Encoding utf8
git -C $taskStage add --all
git -C $taskStage commit --quiet -m "site: publish local build from $($taskSource.Substring(0,7))"
if($LASTEXITCODE -ne 0){throw 'Could not commit prebuilt files'}
git -C $taskStage push origin gh-pages
if($LASTEXITCODE -ne 0){throw 'Publication branch push failed; all staging files are retained'}
$taskSettings=Join-Path $taskRoot ('target/pages-settings-'+[guid]::NewGuid().ToString('N')+'.json')
@{build_type='legacy';source=@{branch='gh-pages';path='/'}}|ConvertTo-Json|Set-Content -LiteralPath $taskSettings -Encoding utf8
gh api --method PUT "repos/$taskRepo/pages" --input $taskSettings | Out-Null
if($LASTEXITCODE -ne 0){throw 'Files are pushed; configure Pages to use gh-pages at /'}
gh api --method POST "repos/$taskRepo/pages/builds" | Out-Null
if($LASTEXITCODE -ne 0){throw 'Files are pushed; request the Pages hosting deployment'}
@{sourceCommit=$taskSource;branch='gh-pages';builtLocally=$true;stagingDirectory=[IO.Path]::GetRelativePath($taskRoot,$taskStage)}|ConvertTo-Json|Set-Content -LiteralPath (Join-Path $taskRoot 'artifacts/p7/local-site-publication.json') -Encoding utf8
Write-Output 'Pushed the prebuilt site to gh-pages. GitHub Pages only needs to deploy these files.'
