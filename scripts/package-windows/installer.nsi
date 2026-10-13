; Internet Speed Monitor installer for Windows.
;
;   makensis -DVERSION=2.0.0 -DSOURCE_DIR=<folder with internet-speed-monitor.exe> ^
;            -DICON=<icon.ico> -DOUTFILE=<installer.exe> installer.nsi
;
; What it does:
; - Installs for the current user (no admin prompt) to
;   %LOCALAPPDATA%\Programs\Internet Speed Monitor, the same folder 1.x used.
; - Removes a 1.x (Electron) installation first, using its own uninstaller in silent
;   mode. 1.x keeps user data when uninstalled; the history is also backed up before the
;   uninstaller runs.
; - Moves a 1.x "Start with computer" entry over to 2.x.
; - Never deletes %APPDATA%\internet-speed-monitor (history, settings, debug log), on
;   install or uninstall.

Unicode true
SetCompressor /SOLID lzma
RequestExecutionLevel user
ManifestDPIAware true

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "StrFunc.nsh"
${StrStr}

!ifndef VERSION
  !error "Pass -DVERSION=x.y.z"
!endif
!ifndef SOURCE_DIR
  !error "Pass -DSOURCE_DIR=<folder with the built exe>"
!endif
!ifndef ICON
  !error "Pass -DICON=<path to icon.ico>"
!endif
!ifndef OUTFILE
  !define OUTFILE "Internet Speed Monitor Setup ${VERSION}.exe"
!endif

!define APP_NAME "Internet Speed Monitor"
!define APP_EXE "internet-speed-monitor.exe"
!define APP_ID "com.omrilotan.internet-speed-monitor"
!define UNINSTALLER "Uninstall ${APP_NAME}.exe"
!define DATA_DIR_NAME "internet-speed-monitor"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APP_ID}"
!define RUN_KEY "Software\Microsoft\Windows\CurrentVersion\Run"
!define STARTUP_APPROVED_KEY "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run"

; Written by the 1.x installer (electron-builder). The GUID is UUID v5 of the app id in
; electron-builder's namespace.
!define LEGACY_GUID "7e80438c-3e51-5232-9119-5f227d0b6b72"
!define LEGACY_INSTALL_KEY "Software\${LEGACY_GUID}"
!define LEGACY_UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${LEGACY_GUID}"
!define LEGACY_UNINSTALLER "Uninstall Internet Speed Monitor.exe"
!define LEGACY_EXE "Internet Speed Monitor.exe"

Name "${APP_NAME}"
OutFile "${OUTFILE}"
InstallDir "$LOCALAPPDATA\Programs\${APP_NAME}"
InstallDirRegKey HKCU "${UNINSTALL_KEY}" "InstallLocation"
BrandingText "${APP_NAME} ${VERSION}"

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "${APP_NAME}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "FileDescription" "${APP_NAME} Setup"
VIAddVersionKey "CompanyName" "Omri Lotan"
VIAddVersionKey "LegalCopyright" "Unlicense"

!define MUI_ICON "${ICON}"
!define MUI_UNICON "${ICON}"
!define MUI_ABORTWARNING
!define MUI_FINISHPAGE_RUN "$INSTDIR\${APP_EXE}"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Var HadLegacyAutostart
Var LegacyRemoved

; Ask running copies of 1.x and 2.x to close, then force any that did not.
!macro CloseRunningApp
  nsExec::Exec 'taskkill /IM "${APP_EXE}"'
  Pop $0
  nsExec::Exec 'taskkill /IM "${LEGACY_EXE}"'
  Pop $0
  Sleep 1500
  nsExec::Exec 'taskkill /F /IM "${APP_EXE}"'
  Pop $0
  nsExec::Exec 'taskkill /F /IM "${LEGACY_EXE}"'
  Pop $0
!macroend

; Keep one copy of the 1.x history, the same backup 2.x makes on first launch.
Function BackupLegacyHistory
  SetShellVarContext current
  StrCpy $0 "$APPDATA\${DATA_DIR_NAME}"
  ${If} ${FileExists} "$0\speed_tests.json"
  ${AndIfNot} ${FileExists} "$0\speed_tests.backup-1.x.json"
    CopyFiles /SILENT "$0\speed_tests.json" "$0\speed_tests.backup-1.x.json"
    DetailPrint "Backed up speed test history to $0\speed_tests.backup-1.x.json"
  ${EndIf}
FunctionEnd

; Remove "Start with computer" entries that point at the 1.x executable and remember
; that the user had it on.
Function MigrateLegacyAutostart
  StrCpy $R0 0
  loop:
    EnumRegValue $R1 HKCU "${RUN_KEY}" $R0
    StrCmp $R1 "" done
    ReadRegStr $R2 HKCU "${RUN_KEY}" $R1
    ${StrStr} $R3 $R2 "\${LEGACY_EXE}"
    ${If} $R3 != ""
      DeleteRegValue HKCU "${RUN_KEY}" $R1
      DeleteRegValue HKCU "${STARTUP_APPROVED_KEY}" $R1
      StrCpy $HadLegacyAutostart 1
      DetailPrint "Moved 1.x start-with-computer entry ($R1)"
      ; Values shift down after a delete: read the same index again.
      Goto loop
    ${EndIf}
    IntOp $R0 $R0 + 1
    Goto loop
  done:
FunctionEnd

; Run the 1.x uninstaller registered under ROOT (HKCU or HKLM), silently.
; MODE is the flag electron-builder's uninstaller expects (/currentuser or /allusers).
!macro RemoveLegacyInstall ROOT MODE DEFAULT_DIR
  ClearErrors
  ReadRegStr $R4 ${ROOT} "${LEGACY_UNINSTALL_KEY}" "UninstallString"
  ${IfNot} ${Errors}
  ${AndIf} $R4 != ""
    ReadRegStr $R5 ${ROOT} "${LEGACY_INSTALL_KEY}" "InstallLocation"
    ${If} $R5 == ""
      StrCpy $R5 "${DEFAULT_DIR}"
    ${EndIf}
    ${If} ${FileExists} "$R5\${LEGACY_UNINSTALLER}"
      Call BackupLegacyHistory
      DetailPrint "Removing ${APP_NAME} 1.x from $R5"
      ; Run a copy so the original can be deleted; _?= keeps the copy waiting for it.
      ; No --delete-app-data and no data-deletion define in 1.x: user data is kept.
      CopyFiles /SILENT "$R5\${LEGACY_UNINSTALLER}" "$PLUGINSDIR\legacy-uninstaller.exe"
      ExecWait '"$PLUGINSDIR\legacy-uninstaller.exe" /S ${MODE} _?=$R5' $R6
      ${If} $R6 == 0
        Delete "$R5\${LEGACY_UNINSTALLER}"
        RMDir "$R5"
        StrCpy $LegacyRemoved 1
        DetailPrint "Removed ${APP_NAME} 1.x"
      ${Else}
        DetailPrint "The 1.x uninstaller exited with code $R6"
        MessageBox MB_OK|MB_ICONEXCLAMATION "${APP_NAME} 1.x could not be removed automatically.$\r$\nYou can remove it later from Settings > Apps. Your speed test history is kept either way." /SD IDOK
      ${EndIf}
    ${EndIf}
  ${EndIf}
!macroend

Function .onInit
  StrCpy $HadLegacyAutostart 0
  StrCpy $LegacyRemoved 0
  InitPluginsDir
FunctionEnd

Section "Install"
  SetShellVarContext current

  !insertmacro CloseRunningApp
  Call MigrateLegacyAutostart
  !insertmacro RemoveLegacyInstall HKCU "/currentuser" "$LOCALAPPDATA\Programs\${APP_NAME}"
  !insertmacro RemoveLegacyInstall HKLM "/allusers" "$PROGRAMFILES64\${APP_NAME}"

  SetOutPath "$INSTDIR"
  File "${SOURCE_DIR}\${APP_EXE}"
  WriteUninstaller "$INSTDIR\${UNINSTALLER}"

  CreateShortcut "$SMPROGRAMS\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"
  CreateShortcut "$DESKTOP\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"

  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${APP_NAME}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\${APP_EXE},0"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "Omri Lotan"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "URLInfoAbout" "https://github.com/omrilotan/internet-speed-monitor"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\${UNINSTALLER}"'
  WriteRegStr HKCU "${UNINSTALL_KEY}" "QuietUninstallString" '"$INSTDIR\${UNINSTALLER}" /S'
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1

  ; Same value the app writes for "Start with computer" (see src/launch.rs).
  ${If} $HadLegacyAutostart == 1
    WriteRegStr HKCU "${RUN_KEY}" "${APP_NAME}" '"$INSTDIR\${APP_EXE}" '
  ${EndIf}
SectionEnd

Section "Uninstall"
  SetShellVarContext current

  nsExec::Exec 'taskkill /IM "${APP_EXE}"'
  Pop $0
  Sleep 1000
  nsExec::Exec 'taskkill /F /IM "${APP_EXE}"'
  Pop $0

  Delete "$INSTDIR\${APP_EXE}"
  Delete "$INSTDIR\${UNINSTALLER}"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\${APP_NAME}.lnk"
  Delete "$DESKTOP\${APP_NAME}.lnk"

  DeleteRegValue HKCU "${RUN_KEY}" "${APP_NAME}"
  DeleteRegValue HKCU "${STARTUP_APPROVED_KEY}" "${APP_NAME}"
  DeleteRegKey HKCU "${UNINSTALL_KEY}"

  ; %APPDATA%\internet-speed-monitor (history, settings, log) is intentionally kept.
SectionEnd
