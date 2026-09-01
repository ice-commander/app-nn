!include "MUI2.nsh"
!include "x64.nsh"

Name "NodeInNet Ice Commander Console"
OutFile "..\\..\\distr\\ice-commander-console-0.7.92-1-win64.exe"
InstallDir "$PROGRAMFILES64\NodeInNet Ice Commander Console"
Target amd64-unicode

SetCompressor /SOLID lzma

RequestExecutionLevel admin

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

!define MUI_FINISHPAGE_RUN "$INSTDIR\nodeinnet-ice-console.exe"
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
        StrCpy $INSTDIR "$PROGRAMFILES64\NodeInNet Ice Commander Console"
    ${Else}
        MessageBox MB_OK|MB_ICONSTOP "This program requires 64-bit Windows / Эта программа требует 64-битную версию Windows."
        Abort
    ${EndIf}
FunctionEnd

Section "Ice Commander Console (Required)" SecMain
    SectionIn RO ; Read Only - cannot be deselected
    SetOutPath "$INSTDIR"

    ; A single GTK-free binary — no gtk4-win32-x64 DLLs to bundle.
    File "..\..\bin\distr\exe\target\x86_64-pc-windows-gnu\release\nodeinnet-ice-console.exe"

    CreateDirectory "$SMPROGRAMS\NodeInNet Ice Commander Console"
    CreateShortcut "$SMPROGRAMS\NodeInNet Ice Commander Console\NodeInNet Ice Commander Console.lnk" "$INSTDIR\nodeinnet-ice-console.exe" "" "$INSTDIR\nodeinnet-ice-console.exe" 0
    CreateShortcut "$SMPROGRAMS\NodeInNet Ice Commander Console\Uninstall NodeInNet Ice Commander Console.lnk" "$INSTDIR\Uninstall.exe"

    WriteUninstaller "$INSTDIR\Uninstall.exe"

    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommanderConsole" "DisplayName" "NodeInNet Ice Commander Console"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommanderConsole" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommanderConsole" "DisplayIcon" '"$INSTDIR\nodeinnet-ice-console.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommanderConsole" "DisplayVersion" "0.7.92"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommanderConsole" "Publisher" "NodeInNet"
SectionEnd

Section /o "Create Desktop Shortcut" SecDesktop
    CreateShortcut "$DESKTOP\NodeInNet Ice Commander Console.lnk" "$INSTDIR\nodeinnet-ice-console.exe"
SectionEnd

Section "Uninstall"
	SetRegView 64

    ExecWait 'taskkill /F /IM nodeinnet-ice-console.exe'

    RMDir /r "$INSTDIR"

    Delete "$SMPROGRAMS\NodeInNet Ice Commander Console\NodeInNet Ice Commander Console.lnk"
    Delete "$SMPROGRAMS\NodeInNet Ice Commander Console\Uninstall NodeInNet Ice Commander Console.lnk"
    RMDir "$SMPROGRAMS\NodeInNet Ice Commander Console"
    Delete "$DESKTOP\NodeInNet Ice Commander Console.lnk"

    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommanderConsole"
SectionEnd
