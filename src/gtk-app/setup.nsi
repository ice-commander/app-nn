!include "MUI2.nsh"
!include "x64.nsh"

Name "NodeInNet Ice Commander"
OutFile "..\\..\\distr\\nodeinnet-ice-commander-0.7.121-1-win64.exe"
InstallDir "$PROGRAMFILES64\NodeInNet Ice Commander"
Target amd64-unicode

SetCompressor /SOLID lzma

RequestExecutionLevel admin

!define MUI_ICON "assets\win32-icon.ico"
!define MUI_UNICON "assets\win32-icon.ico"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

!define MUI_FINISHPAGE_RUN "$INSTDIR\nodeinnet-ice-commander.exe"
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
        StrCpy $INSTDIR "$PROGRAMFILES64\NodeInNet Ice Commander"
    ${Else}
        MessageBox MB_OK|MB_ICONSTOP "This program requires 64-bit Windows / Эта программа требует 64-битную версию Windows."
        Abort
    ${EndIf}
FunctionEnd

Section "Ice Commander (Required)" SecMain
    SectionIn RO ; Read Only - cannot be deselected
    SetOutPath "$INSTDIR"
    
    File "..\..\bin\distr\exe\target\x86_64-pc-windows-gnu\release\nodeinnet-ice-commander.exe"
    
    File /r "..\..\artifacts\gtk4-win32-x64\*"

    ; The bundle ships LGPL (GTK stack) and GPL (libmpv/FFmpeg) libraries; their license
    ; texts and the source references must accompany it.
    SetOutPath "$INSTDIR\licenses"
    File "..\..\assets\licenses\*.txt"
    SetOutPath "$INSTDIR"
    
    CreateDirectory "$SMPROGRAMS\NodeInNet Ice Commander"
    CreateShortcut "$SMPROGRAMS\NodeInNet Ice Commander\NodeInNet Ice Commander.lnk" "$INSTDIR\nodeinnet-ice-commander.exe" "" "$INSTDIR\nodeinnet-ice-commander.exe" 0
    CreateShortcut "$SMPROGRAMS\NodeInNet Ice Commander\Uninstall NodeInNet Ice Commander.lnk" "$INSTDIR\Uninstall.exe"
    
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommander" "DisplayName" "NodeInNet Ice Commander"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommander" "UninstallString" '"$INSTDIR\Uninstall.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommander" "DisplayIcon" '"$INSTDIR\nodeinnet-ice-commander.exe"'
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommander" "DisplayVersion" "0.7.121"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommander" "Publisher" "NodeInNet"
SectionEnd

Section /o "Create Desktop Shortcut" SecDesktop
    CreateShortcut "$DESKTOP\NodeInNet Ice Commander.lnk" "$INSTDIR\nodeinnet-ice-commander.exe"
SectionEnd

Section "Uninstall"
	SetRegView 64

    ExecWait 'taskkill /F /IM nodeinnet-ice-commander.exe'
    
    RMDir /r "$INSTDIR"
    
    Delete "$SMPROGRAMS\NodeInNet Ice Commander\NodeInNet Ice Commander.lnk"
    Delete "$SMPROGRAMS\NodeInNet Ice Commander\Uninstall NodeInNet Ice Commander.lnk"
    RMDir "$SMPROGRAMS\NodeInNet Ice Commander"
    Delete "$DESKTOP\NodeInNet Ice Commander.lnk"
    
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommander"
SectionEnd
