!include "MUI2.nsh"
!include "x64.nsh"

Name "NodeInNet Ice Commander Web Server"
OutFile "..\\..\\distr\\ice-commander-webserver-0.7.92-1-win64.exe"
InstallDir "$PROGRAMFILES64\NodeInNet Ice Commander Web Server"
Target amd64-unicode

SetCompressor /SOLID lzma

RequestExecutionLevel admin

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

!define MUI_FINISHPAGE_RUN "$INSTDIR\nodeinnet-ice-webserver.exe"
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"
!insertmacro MUI_LANGUAGE "Russian"

Function .onInit
    ${If} ${RunningX64}
        SetRegView 64
        StrCpy $INSTDIR "$PROGRAMFILES64\NodeInNet Ice Commander Web Server"
    ${Else}
        MessageBox MB_OK|MB_ICONSTOP "This program requires 64-bit Windows / Эта программа требует 64-битную версию Windows."
        Abort
    ${EndIf}
FunctionEnd

Section "Ice Commander Web Server (Required)" SecMain
    SectionIn RO ; Read Only - cannot be deselected
    SetOutPath "$INSTDIR"

    ; A single GTK-free binary — no gtk4-win32-x64 DLLs to bundle.
    File "..\..\bin\distr\exe\target\x86_64-pc-windows-gnu\release\nodeinnet-ice-webserver.exe"

    CreateDirectory "$SMPROGRAMS\NodeInNet Ice Commander Web Server"
    CreateShortcut "$SMPROGRAMS\NodeInNet Ice Commander Web Server\NodeInNet Ice Commander Web Server.lnk" "$INSTDIR\nodeinnet-ice-webserver.exe" "" "$INSTDIR\nodeinnet-ice-webserver.exe" 0
    CreateShortcut "$SMPROGRAMS\NodeInNet Ice Commander Web Server\Uninstall NodeInNet Ice Commander Web Server.lnk" "$INSTDIR\Uninstall.exe"

    WriteUninstaller "$INSTDIR\Uninstall.exe"

    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommanderWebServer" "DisplayName" "NodeInNet Ice Commander Web Server"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommanderWebServer" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommanderWebServer" "DisplayIcon" '"$INSTDIR\nodeinnet-ice-webserver.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommanderWebServer" "DisplayVersion" "0.7.92"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommanderWebServer" "Publisher" "NodeInNet"
SectionEnd

Section /o "Create Desktop Shortcut" SecDesktop
    CreateShortcut "$DESKTOP\NodeInNet Ice Commander Web Server.lnk" "$INSTDIR\nodeinnet-ice-webserver.exe"
SectionEnd

Section "Uninstall"
	SetRegView 64

    ExecWait 'taskkill /F /IM nodeinnet-ice-webserver.exe'

    RMDir /r "$INSTDIR"

    Delete "$SMPROGRAMS\NodeInNet Ice Commander Web Server\NodeInNet Ice Commander Web Server.lnk"
    Delete "$SMPROGRAMS\NodeInNet Ice Commander Web Server\Uninstall NodeInNet Ice Commander Web Server.lnk"
    RMDir "$SMPROGRAMS\NodeInNet Ice Commander Web Server"
    Delete "$DESKTOP\NodeInNet Ice Commander Web Server.lnk"

    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommanderWebServer"
SectionEnd
