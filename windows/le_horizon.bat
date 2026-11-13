@echo off
rem Horizon without the launcher window, set up for a slow laptop.
rem Put this next to horizon-launcher.exe (or install the launcher from playhorizon.dev) and
rem double-click it, every time.
rem First run: signs in, installs Horizon and the Waypoints mod, applies fast settings (once).
rem Every run: updates the launcher and Horizon, then shows a menu: play, switch versions,
rem mods, servers, worlds, packs, accounts, screenshots, logs and settings.
setlocal
cd /d "%~dp0"
set "WAYPOINTS=https://github.com/reparking-parker/horizon-waypoints"
set "GAME=%APPDATA%\horizon"
rem Prefer the launcher the Horizon installer puts in AppData (it is the up-to-date one),
rem else an old copy next to this file.
set "L=%LOCALAPPDATA%\Programs\Horizon Launcher\horizon-launcher.exe"
if not exist "%L%" set "L=%~dp0horizon-launcher.exe"
if not exist "%L%" (
  echo Horizon Launcher is not installed. Install it from playhorizon.dev,
  echo or put this file in the same folder as horizon-launcher.exe, then run it again.
  pause
  exit /b 1
)

if not exist horizon-setup-done.txt (
  echo First run: setting up. The "prepare" step is slow on this laptop, let it finish.
  "%L%" settings opening-animation off
  "%L%" settings while-playing hide
  "%L%" settings install-new-versions off
  echo Sign in: a browser opens, type in the code shown here.
  "%L%" account sign-in || goto failed
  "%L%" install latest || goto failed
  "%L%" prepare || "%L%" prepare --download || goto failed
  echo done > horizon-setup-done.txt
)

rem Every run: update the launcher and Horizon. Older versions are kept, so they can still
rem be played.
"%L%" settings prune-old-versions off >nul 2>nul
echo Checking for Horizon and launcher updates...
"%L%" update || echo Could not check for updates, carrying on.

rem Old launchers cannot pick a version or build native mods (they say "invalid value").
call :canpick
if errorlevel 1 (set "OLDLAUNCHER=1") else (set "OLDLAUNCHER=")
call :canbuild
if errorlevel 1 (set "NOBUILD=1") else (set "NOBUILD=")
set "PINS="
"%L%" native-build --help 2>nul | findstr /c:"--ignore-pins" >nul && set "PINS=--ignore-pins"

if defined OLDLAUNCHER (
  echo.
  echo  Your Horizon Launcher is too old for this menu.
  echo  Install the new one from playhorizon.dev ^(the setup shows no window, wait about a
  echo  minute^), then run this file again. For now it plays the version you have.
  echo.
  pause
  goto play
)

rem The Waypoints mod, once: the first run after it is in the mods list.
if not exist horizon-waypoints-installed.txt if not defined NOBUILD (
  echo.
  echo  The Waypoints mod is not installed yet. It needs a build of the game on this computer:
  echo  the first one downloads Rust ^(about 450 MB^) and can take 30 minutes or more here.
  choice /c YN /n /m "Install it now? Press Y or N: "
  if not errorlevel 2 call :waypoints_install
)

:menu
set "HVER="
cls
echo.
echo  ==================== Horizon ====================
echo    1. Play
echo    2. Switch Horizon version
echo    3. Mods ^(Waypoints^)
echo    4. Join a server / server list
echo    5. Worlds
echo    6. Resource and shader packs
echo    7. Accounts
echo    8. Screenshots
echo    9. Logs and crash reports
echo    S. Settings
echo    Q. Quit
echo  =================================================
choice /c 123456789SQ /n /m "Press a key: "
if errorlevel 11 exit /b 0
if errorlevel 10 goto settings
if errorlevel 9 goto logs
if errorlevel 8 goto screenshots
if errorlevel 7 goto accounts
if errorlevel 6 goto packs
if errorlevel 5 goto worlds
if errorlevel 4 goto servers
if errorlevel 3 goto mods
if errorlevel 2 goto versions
goto play

rem ---------------------------------------------------------------- versions
:versions
echo.
echo  Horizon version
echo    1. Play a version I already have ^(this time only, without rebuilding the mods^)
echo    2. Download the newest version and switch to it
echo    3. Download a specific version and switch to it
echo    B. Back
choice /c 123B /n /m "Press a key: "
if errorlevel 4 goto menu
if errorlevel 3 goto pick
if errorlevel 2 goto newest
goto have

:have
echo.
echo Installed versions (* = the one that plays by default):
"%L%" versions --installed
echo.
set /p "HVER=Type one to play it, or just press Enter for the * one: "
goto play

:newest
rem Ask the launcher which version is newest (the first one that is not a pre-release).
for /f "usebackq delims=" %%t in (`powershell -NoProfile -Command "((& '%L%' versions --json | Out-String) | ConvertFrom-Json) | Where-Object { -not $_.prerelease } | Select-Object -First 1 -ExpandProperty tag"`) do set "HVER=%%t"
if not defined HVER (
  echo Could not get the version list. Check the internet connection.
  pause
  goto menu
)
goto install

:pick
echo.
echo Versions you can download (newest at the top, press Space to see more):
echo.
"%L%" versions | more
echo.
set /p "HVER=Type the version exactly as listed (example v0.2.97), or press Enter to go back: "
if not defined HVER goto menu
rem "0.2.97" means "v0.2.97"
set "FIRST=%HVER:~0,1%"
for %%d in (0 1 2 3 4 5 6 7 8 9) do if "%FIRST%"=="%%d" set "HVER=v%HVER%"
goto install

:install
echo.
echo Downloading Horizon %HVER%. The "prepare" step is slow on this laptop, let it finish.
"%L%" settings prune-old-versions off >nul 2>nul
"%L%" install %HVER% || goto badversion
"%L%" settings selected-version %HVER% >nul 2>nul
"%L%" prepare --version %HVER% || "%L%" prepare --version %HVER% --download || goto failed
rem The mods are built for one version: build them again on the new one.
if exist horizon-waypoints-installed.txt (
  echo.
  echo Building the game with your mods on %HVER%. This can take a while, keep this window open.
  "%L%" native-build --version %HVER% %PINS% --trust-native-code && goto built
  echo.
  echo The mods did not build on %HVER%. Waypoints needs Horizon Nightly 2026-11-08 or newer.
  echo Playing %HVER% without mods this time.
  pause
)
goto play

:built
rem The build is what the instance plays now: no --version, or it would start the plain one.
set "HVER="
goto play

:badversion
echo.
echo Could not download version "%HVER%". Check that it is typed exactly as in the list.
pause
goto menu

rem ---------------------------------------------------------------- mods
:mods
echo.
echo  Mods
if exist horizon-waypoints-installed.txt (echo    Waypoints: installed) else (echo    Waypoints: not installed)
echo    1. Install Waypoints, or get its newest version
echo    2. Build the game with the installed mods again
echo    3. Remove Waypoints
echo    4. List the installed mods
echo    5. Open the mods folder
echo    B. Back
choice /c 12345B /n /m "Press a key: "
if errorlevel 6 goto menu
if errorlevel 5 (explorer "%GAME%\horizon-mods" & goto mods)
if errorlevel 4 ("%L%" mods list & pause & goto mods)
if errorlevel 3 goto waypoints_remove
if errorlevel 2 (call :build & goto mods)
call :waypoints_install
goto mods

:waypoints_install
if defined NOBUILD (
  echo This launcher cannot build native mods yet. Update it from playhorizon.dev first.
  pause
  exit /b 1
)
echo.
echo Getting Waypoints from GitHub...
if exist horizon-waypoints-installed.txt (
  "%L%" mods update-github waypoints --trust-native-code || goto mod_failed
) else (
  "%L%" mods add-github %WAYPOINTS% --trust-native-code || goto mod_failed
)
echo done > horizon-waypoints-installed.txt
call :build
exit /b 0

:build
echo.
echo Building the game with your mods. The first time it downloads Rust ^(about 450 MB^);
echo it can take 30 minutes or more here. Keep this window open until it says Done.
"%L%" native-build %PINS% --trust-native-code || goto build_failed
echo.
echo Done. Your old waypoints come back by themselves: type /wp list in chat to see them.
pause
exit /b 0

:build_failed
echo.
echo The build failed. Waypoints needs Horizon Nightly 2026-11-08 or newer: switch to the
echo newest version in menu 2, then build again. If it still fails, send a screenshot.
pause
exit /b 1

:mod_failed
echo.
echo Could not get the mod from GitHub. Check the internet connection, then try again.
pause
exit /b 1

:waypoints_remove
"%L%" mods remove waypoints || (pause & goto mods)
del horizon-waypoints-installed.txt 2>nul
echo Removed. Your saved waypoints stay in the game folder if you install it again.
echo To play without it, pick a version in menu 2, option 1.
pause
goto mods

rem ---------------------------------------------------------------- servers
:servers
echo.
echo  Servers
"%L%" multiplayer list
echo.
echo    1. Join a server from the list ^(or any address^)
echo    2. Add a server to the list
echo    3. Remove a server from the list
echo    B. Back
choice /c 123B /n /m "Press a key: "
if errorlevel 4 goto menu
if errorlevel 3 goto server_remove
if errorlevel 2 goto server_add
set "SRV="
set /p "SRV=Type its number, name or address: "
if not defined SRV goto servers
call :optimize
"%L%" play --server "%SRV%" || goto failed
call :optimize
exit /b 0

:server_add
set "SRV=" & set "SRVNAME="
set /p "SRV=The server's address (example play.example.com): "
if not defined SRV goto servers
set /p "SRVNAME=A name for it (or Enter for the address): "
if defined SRVNAME ("%L%" multiplayer add "%SRV%" --name "%SRVNAME%") else ("%L%" multiplayer add "%SRV%")
pause
goto servers

:server_remove
set "SRV="
set /p "SRV=The number, name or address to remove: "
if defined SRV "%L%" multiplayer remove "%SRV%"
pause
goto servers

rem ---------------------------------------------------------------- worlds
:worlds
echo.
echo  Worlds
"%L%" worlds list
echo.
echo    1. Play a world straight away
echo    2. Back up a world ^(a zip in the backups folder^)
echo    3. Copy a world
echo    4. Delete a world ^(it goes to the Recycle Bin^)
echo    5. Open the saves folder
echo    B. Back
choice /c 12345B /n /m "Press a key: "
set "K=%errorlevel%"
if "%K%"=="6" goto menu
if "%K%"=="5" (explorer "%GAME%\saves" & goto worlds)
set "WORLD="
set /p "WORLD=The world's name, as listed: "
if not defined WORLD goto worlds
if "%K%"=="4" goto world_delete
if "%K%"=="3" ("%L%" worlds copy "%WORLD%" & pause & goto worlds)
if "%K%"=="2" ("%L%" worlds backup "%WORLD%" & pause & goto worlds)
call :optimize
"%L%" play --world "%WORLD%" || goto failed
call :optimize
exit /b 0

:world_delete
choice /c YN /n /m "Delete this world? It goes to the Recycle Bin. Press Y or N: "
if errorlevel 2 goto worlds
"%L%" worlds delete "%WORLD%"
pause
goto worlds

rem ---------------------------------------------------------------- packs
:packs
echo.
echo  Resource and shader packs
"%L%" packs list
echo.
echo    1. Add a pack file ^(.zip or .hshader^)
echo    2. Turn a resource pack on
echo    3. Turn a resource pack off
echo    4. Turn a shader pack on
echo    5. Turn a shader pack off
echo    6. Open the resource packs folder
echo    B. Back
choice /c 123456B /n /m "Press a key: "
set "K=%errorlevel%"
if "%K%"=="7" goto menu
if "%K%"=="6" (explorer "%GAME%\resourcepacks" & goto packs)
set "PACK="
if "%K%"=="1" goto pack_add
set /p "PACK=The pack's name, as listed: "
if not defined PACK goto packs
if "%K%"=="5" ("%L%" packs off "%PACK%" --kind shader & pause & goto packs)
if "%K%"=="4" ("%L%" packs on "%PACK%" --kind shader & pause & goto packs)
if "%K%"=="3" ("%L%" packs off "%PACK%" & pause & goto packs)
("%L%" packs on "%PACK%" & pause & goto packs)

:pack_add
set /p "PACK=Drag the pack file into this window, then press Enter: "
if defined PACK "%L%" packs add %PACK%
pause
goto packs

rem ---------------------------------------------------------------- accounts
:accounts
echo.
echo  Accounts
"%L%" account list
echo.
echo    1. Sign in with another Microsoft account
echo    2. Switch the account that plays
echo    3. Play offline under a name ^(LAN and offline servers^)
echo    B. Back
choice /c 123B /n /m "Press a key: "
if errorlevel 4 goto menu
if errorlevel 3 goto offline
if errorlevel 2 goto account_select
"%L%" account sign-in
pause
goto accounts

:account_select
set "WHO="
set /p "WHO=The account's name, as listed: "
if defined WHO "%L%" account select "%WHO%"
pause
goto accounts

:offline
set "WHO="
set /p "WHO=The name to play as (3 to 16 letters, digits or _): "
if defined WHO "%L%" account offline "%WHO%"
pause
goto accounts

rem ---------------------------------------------------------------- screenshots
:screenshots
echo.
"%L%" screenshots list
echo.
echo    1. Open the screenshots folder
echo    B. Back
choice /c 1B /n /m "Press a key: "
if errorlevel 2 goto menu
explorer "%GAME%\screenshots"
goto screenshots

rem ---------------------------------------------------------------- logs
:logs
echo.
echo  Logs
echo    1. Show the end of the game's log
echo    2. Show the newest crash report
echo    3. Upload the log for help ^(gives a link to send; names are taken out^)
echo    B. Back
choice /c 123B /n /m "Press a key: "
if errorlevel 4 goto menu
if errorlevel 3 ("%L%" share-log & pause & goto logs)
if errorlevel 2 ("%L%" logs --crash | more & pause & goto logs)
"%L%" logs --lines 60
pause
goto logs

rem ---------------------------------------------------------------- settings
:settings
echo.
echo  Settings
echo    1. Apply the fast settings again ^(low graphics for this laptop^)
echo    2. Mods build faster to rebuild ^(slower game: dev^)
echo    3. Mods build for the best game speed ^(default: performance^)
echo    4. Show the launcher's settings
echo    5. Open the game folder
echo    B. Back
choice /c 12345B /n /m "Press a key: "
if errorlevel 6 goto menu
if errorlevel 5 (explorer "%GAME%" & goto settings)
if errorlevel 4 ("%L%" settings & pause & goto settings)
if errorlevel 3 ("%L%" settings native-profile performance & pause & goto settings)
if errorlevel 2 ("%L%" settings native-profile dev & pause & goto settings)
del horizon-optimized.txt 2>nul
call :optimize
pause
goto settings

rem ---------------------------------------------------------------- play
:play
call :optimize
set "VARG="
if defined HVER set "VARG=--version %HVER%"
"%L%" play %VARG% && goto played
echo.
echo The game did not start. Preparing this version and trying once more...
"%L%" prepare %VARG% || "%L%" prepare %VARG% --download || goto failed
"%L%" play %VARG% || goto failed
:played
call :optimize
exit /b 0

:canpick
rem Succeeds when this launcher knows every command the version menu uses.
"%L%" play --help 2>nul | findstr /c:"--version" >nul || exit /b 1
"%L%" settings --help 2>nul | findstr /c:"prune-old-versions" >nul || exit /b 1
"%L%" versions --help 2>nul | findstr /c:"--json" >nul || exit /b 1
exit /b 0

:canbuild
rem Succeeds when this launcher can install mods from GitHub and build native mods.
"%L%" native-build --help 2>nul | findstr /c:"--trust-native-code" >nul || exit /b 1
"%L%" mods --help 2>nul | findstr /c:"add-github" >nul || exit /b 1
exit /b 0

:optimize
if exist horizon-optimized.txt exit /b 0
powershell -NoProfile -ExecutionPolicy Bypass -Command "$s = [IO.File]::ReadAllText('%~f0'); iex ($s -split ('#' + 'POWERSHELL' + '#'))[-1]"
exit /b 0

:failed
echo.
echo Something failed above. Send a screenshot of this window.
pause
exit /b 1

#POWERSHELL#
# Fast settings, written into the game folder once. Keys the game has not written
# yet are added; options.txt is finished after the first game start.
$game = Join-Path $env:APPDATA 'horizon'
New-Item -ItemType Directory -Force (Join-Path $game 'horizon-mods') | Out-Null

$settings = @{
  'options.txt' = @(
    'renderDistance:5', 'simulationDistance:5', 'graphicsPreset:"custom"',
    'ao:false', 'biomeBlendRadius:0', 'chunkSectionFadeInTime:0.0', 'cutoutLeaves:false',
    'enableVsync:false', 'maxFps:260', 'entityDistanceScaling:0.5', 'entityShadows:false',
    'improvedTransparency:false', 'mipmapLevels:0', 'particles:2', 'renderClouds:"false"',
    'vignette:false', 'weatherRadius:3', 'menuBackgroundBlurriness:0',
    'inactivityFpsLimit:"minimized"', 'fullscreen:true', 'bobView:false')
  'horizon-graphics.txt' = @(
    'preset:custom', 'lodDistance:0', 'farDetail:reduced', 'caveCulling:true',
    'entityCulling:true', 'sky:false', 'fog:false', 'weather:false',
    'animatedTextures:false', 'presentMode:immediate', 'framePacing:efficient',
    'resolutionScale:70', 'fxaa:false', 'hdr:false', 'overlay:fps')
  'horizon-extras.txt' = @('dynamicLights:false', 'discordPresence:false')
}

foreach ($name in $settings.Keys) {
  $file = Join-Path $game $name
  if ($name -eq 'options.txt' -and -not (Test-Path -LiteralPath $file)) { continue }  # the game writes it first
  $lines = [System.Collections.Generic.List[string]]::new()
  if (Test-Path -LiteralPath $file) { $lines.AddRange([IO.File]::ReadAllLines($file)) }
  foreach ($kv in $settings[$name]) {
    $key = $kv.Split(':')[0] + ':'
    $i = $lines.FindIndex([Predicate[string]] { param($l) $l.StartsWith($key) })
    if ($i -ge 0) { $lines[$i] = $kv } else { $lines.Add($kv) }
  }
  [IO.File]::WriteAllLines($file, $lines)  # UTF-8 without BOM
}

# The old WebAssembly Waypoints mod no longer runs: the native one replaces it.
$old = Join-Path $game 'horizon-mods\waypoints.hmod'
if (Test-Path -LiteralPath $old) { Remove-Item -LiteralPath $old -Force }

# options.txt only counts as done once the game itself has written it
$opts = Join-Path $game 'options.txt'
if ((Test-Path -LiteralPath $opts) -and (Get-Content -LiteralPath $opts | Measure-Object).Count -gt 40) {
  Set-Content 'horizon-optimized.txt' 'done'
  Write-Host 'Fast settings applied.'
} else {
  Write-Host 'Fast settings applied (they finish after the first game start).'
}
