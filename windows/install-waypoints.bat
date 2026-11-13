@echo off
rem Installs the Waypoints mod without opening the launcher window.
rem Put this next to horizon-launcher.exe and double-click it once. After that, start the game
rem as you always do (horizon.bat). Double-click it again later to get a newer version of the mod.
cd /d "%~dp0"
if not exist horizon-launcher.exe (
  echo Put this file in the same folder as horizon-launcher.exe, then run it again.
  pause
  exit /b 1
)
echo Getting the Waypoints mod from GitHub...
horizon-launcher.exe mods add-github https://github.com/reparking-parker/horizon-waypoints --trust-native-code || horizon-launcher.exe mods update-github waypoints --trust-native-code || goto failed
echo.
echo Building the game with the mod. The first time it downloads Rust (about 450 MB)
echo and takes 10 to 30 minutes. Leave this window open until it says Done.
horizon-launcher.exe native-build --trust-native-code || goto failed
echo.
echo Done. Start the game with horizon.bat as usual, then type /wp list in chat.
pause
exit /b 0

:failed
echo.
echo Something failed above. Send a screenshot of this window.
pause
exit /b 1
