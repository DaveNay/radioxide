Name "Radioxide"
OutFile "radioxide-installer.exe"
InstallDir "$PROGRAMFILES\Radioxide"
RequestExecutionLevel admin

Section "Install"
    SetOutPath "$INSTDIR"
    File "target\release\radioxide-gui.exe"
    File "target\release\radioxide-daemon.exe"
    File "target\release\radioxide-cli.exe"
    CreateShortcut "$DESKTOP\Radioxide GUI.lnk" "$INSTDIR\radioxide-gui.exe"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\radioxide-gui.exe"
    Delete "$INSTDIR\radioxide-daemon.exe"
    Delete "$INSTDIR\radioxide-cli.exe"
    Delete "$DESKTOP\Radioxide GUI.lnk"
    RMDir "$INSTDIR"
SectionEnd
