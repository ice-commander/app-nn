!include "MUI2.nsh"
!include "x64.nsh"

Name "NodeInNet Ice Commander"
OutFile "..\\..\\distr\\nodeinnet-ice-commander-gtk-0.7.126-1-win64.exe"
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
!insertmacro MUI_LANGUAGE "Polish"
!insertmacro MUI_LANGUAGE "Czech"
!insertmacro MUI_LANGUAGE "Slovak"
!insertmacro MUI_LANGUAGE "German"
!insertmacro MUI_LANGUAGE "Spanish"
!insertmacro MUI_LANGUAGE "Ukrainian"
!insertmacro MUI_LANGUAGE "Italian"
!insertmacro MUI_LANGUAGE "French"
!insertmacro MUI_LANGUAGE "Romanian"
!insertmacro MUI_LANGUAGE "Hungarian"
!insertmacro MUI_LANGUAGE "Belarusian"
!insertmacro MUI_LANGUAGE "Bulgarian"
!insertmacro MUI_LANGUAGE "Serbian"

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
    SectionIn RO
    SetOutPath "$INSTDIR"
    
    File "..\..\bin\distr\exe\target\x86_64-pc-windows-gnu\release\nodeinnet-ice-commander.exe"
    
    File /r /x liblzo2-2.dll "..\..\artifacts\gtk4-win32-x64\*"
    File "..\..\bin\distr\fakelzo\liblzo2-2.dll"

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
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\NodeInNetIceCommander" "DisplayVersion" "0.7.122"
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
