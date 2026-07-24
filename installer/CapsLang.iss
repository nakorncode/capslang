#define MyAppName "CapsLang"
#ifndef MyAppVersion
#define MyAppVersion "0.2.0"
#endif
#define MyAppPublisher "NakornCode"
#define MyAppExeName "CapsLang.exe"
#define MyTaskPath "NakornCode\CapsLang"

[Setup]
AppId={{A8C2E4F1-9B3D-4E6A-8F01-2C5D7E9A1B30}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL=https://github.com/nakorncode/capslang
AppSupportURL=https://github.com/nakorncode/capslang
AppUpdatesURL=https://github.com/nakorncode/capslang/releases
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=..\artifacts\installer
OutputBaseFilename=CapsLang-Setup-{#MyAppVersion}-win-x64
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=..\assets\capslang.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
PrivilegesRequired=admin

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "..\artifacts\publish\win-x64\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
; Installer is already elevated; first launch registers the silent elevated logon task.
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppName}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\schtasks.exe"; Parameters: "/Delete /TN ""{#MyTaskPath}"" /F"; Flags: runhidden; RunOnceId: "RemoveCapsLangTask"
