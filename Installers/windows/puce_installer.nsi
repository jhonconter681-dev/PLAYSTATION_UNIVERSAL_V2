!define APP_NAME "PlayStation Universal Controller Emulator"
!define APP_SHORT_NAME "PUCE"
!define APP_VERSION "1.0.0"
!define APP_PUBLISHER "PUCE Team"
!define APP_EXE "PUCE.exe"

SetCompressor lzma
Name "${APP_NAME} ${APP_VERSION}"
OutFile "PUCE_Setup_v${APP_VERSION}.exe"
InstallDir "$PROGRAMFILES64\${APP_SHORT_NAME}"
RequestExecutionLevel admin

Page directory
Page instfiles

Section "Main Application" SecMain
  SetOutPath "$INSTDIR"
  
  ; Copy binaries and libraries
  File "..\..\target\release\PUCE.exe"
  File "..\..\target\release\puce_core.dll"
  File "..\..\UI\build\windows\x64\runner\Release\data\app.so"
  ; (In a real build, we would copy the entire flutter build/windows/x64/runner/Release directory)
  
  ; Create uninstaller
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  
  ; Start Menu shortcut
  CreateDirectory "$SMPROGRAMS\${APP_SHORT_NAME}"
  CreateShortcut "$SMPROGRAMS\${APP_SHORT_NAME}\${APP_SHORT_NAME}.lnk" "$INSTDIR\${APP_EXE}"
  
  ; Desktop shortcut
  CreateShortcut "$DESKTOP\${APP_SHORT_NAME}.lnk" "$INSTDIR\${APP_EXE}"
SectionEnd

Section "ViGEm Bus Driver (Required for Windows Emulation)" SecViGEm
  ; Optional: Download and run ViGEm installer if not detected
  ; For this script we assume the user has it, or we ship the MSI.
  ; ExecWait '"$INSTDIR\ViGEmBusSetup_1.21.442_x64.msi" /quiet'
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\PUCE.exe"
  Delete "$INSTDIR\puce_core.dll"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir /r "$INSTDIR"
  
  Delete "$SMPROGRAMS\${APP_SHORT_NAME}\${APP_SHORT_NAME}.lnk"
  RMDir "$SMPROGRAMS\${APP_SHORT_NAME}"
  Delete "$DESKTOP\${APP_SHORT_NAME}.lnk"
SectionEnd
