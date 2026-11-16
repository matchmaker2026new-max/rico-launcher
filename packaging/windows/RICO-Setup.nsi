Unicode true

!ifndef APP_VERSION
  !define APP_VERSION "0.6.0"
!endif
!ifndef SOURCE_EXE
  !define SOURCE_EXE "..\..\target\release\arctic-launcher.exe"
!endif
!ifndef LOGO_ICO
  !define LOGO_ICO "..\..\assets\rico-logo.ico"
!endif
!ifndef OUTPUT_FILE
  !define OUTPUT_FILE "..\..\dist\windows\RICO-Setup-${APP_VERSION}.exe"
!endif

!include "MUI2.nsh"

Name "RICO"
Caption "RICO Setup"
OutFile "${OUTPUT_FILE}"
InstallDir "$LOCALAPPDATA\Programs\RICO"
RequestExecutionLevel user
; Use the conventional zlib compressor rather than solid LZMA packing. This
; keeps the installer easier for security scanners and users to inspect.
SetCompressor /SOLID zlib
Icon "${LOGO_ICO}"
UninstallIcon "${LOGO_ICO}"

!define MUI_ABORTWARNING
!define MUI_ICON "${LOGO_ICO}"
!define MUI_UNICON "${LOGO_ICO}"
!define MUI_WELCOMEPAGE_TITLE "Welcome to RICO Setup"
!define MUI_WELCOMEPAGE_TEXT "This installer installs RICO. Close any running RICO window (including its tray icon) before continuing."
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!define MUI_UNCONFIRMPAGE_TEXT_TOP "Close RICO before uninstalling. You can choose whether to remove saved profiles, accounts, settings, and Minecraft worlds as well."
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

VIProductVersion "${APP_VERSION}.0"
VIAddVersionKey "ProductName" "RICO"
VIAddVersionKey "FileDescription" "RICO Minecraft Launcher Setup"
VIAddVersionKey "ProductVersion" "${APP_VERSION}"
VIAddVersionKey "FileVersion" "${APP_VERSION}.0"
VIAddVersionKey "LegalCopyright" "RICO"

Function .onInit
  SetShellVarContext current
FunctionEnd

Function un.onInit
  SetShellVarContext current
FunctionEnd

Section "RICO Launcher" SecLauncher
  SectionIn RO
  SetOutPath "$INSTDIR"
  File /oname=RICO.exe "${SOURCE_EXE}"
  Delete "$INSTDIR\arctic-launcher.exe"
  WriteUninstaller "$INSTDIR\Uninstall.exe"

  CreateDirectory "$SMPROGRAMS\RICO"
  CreateShortcut "$SMPROGRAMS\RICO\RICO.lnk" "$INSTDIR\RICO.exe" "" "$INSTDIR\RICO.exe"
  CreateShortcut "$SMPROGRAMS\RICO\Uninstall RICO.lnk" "$INSTDIR\Uninstall.exe"

  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RICO" "DisplayName" "RICO"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RICO" "DisplayVersion" "${APP_VERSION}"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RICO" "Publisher" "RICO"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RICO" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RICO" "DisplayIcon" "$INSTDIR\RICO.exe"
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RICO" "UninstallString" "$\"$INSTDIR\Uninstall.exe$\""
  WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RICO" "QuietUninstallString" "$\"$INSTDIR\Uninstall.exe$\" /S"
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RICO" "NoModify" 1
  WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RICO" "NoRepair" 1
SectionEnd

Section /o "Desktop shortcut" SecDesktop
  CreateShortcut "$DESKTOP\RICO.lnk" "$INSTDIR\RICO.exe" "" "$INSTDIR\RICO.exe"
SectionEnd

Section "Uninstall"
  MessageBox MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON1 "Also delete RICO's local data? This removes saved profiles, accounts, settings, and Minecraft worlds from $LOCALAPPDATA\RICO." IDNO keep_data
  RMDir /r "$LOCALAPPDATA\RICO"
  keep_data:
  Delete "$DESKTOP\RICO.lnk"
  Delete "$SMPROGRAMS\RICO\RICO.lnk"
  Delete "$SMPROGRAMS\RICO\Uninstall RICO.lnk"
  Delete "$APPDATA\Microsoft\Windows\Start Menu\Programs\RICO.lnk"
  Delete "$APPDATA\Microsoft\Windows\Start Menu\Programs\RICO Launcher.lnk"
  RMDir /r "$SMPROGRAMS\RICO"
  RMDir /r "$INSTDIR"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\RICO"
SectionEnd
