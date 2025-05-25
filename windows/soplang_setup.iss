#define MyAppName "Soplang"
#define MyAppVersion "2.0.0"
#define MyAppPublisher "Soplang Software Foundation"
#define MyAppURL "https://www.soplang.org/"
#define MyAppExeName "soplang.exe"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
; Do not use the same AppId value in installers for other applications.
AppId={{F1C77F9E-F26A-4D23-9A8B-CF3D26AE5A18}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} Interpreter
UninstallDisplayName={#MyAppName} Interpreter
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
OutputBaseFilename=soplang-setup
Compression=lzma
SolidCompression=yes
ChangesEnvironment=yes
SetupIconFile=soplang_icon.ico
UninstallDisplayIcon={app}\soplang.exe
WizardStyle=modern
VersionInfoVersion={#MyAppVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription=Installer for {#MyAppName}
VersionInfoCopyright=© 2025 {#MyAppPublisher}
VersionInfoProductName={#MyAppName} Interpreter
VersionInfoProductVersion={#MyAppVersion}


[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "addtopath"; Description: "Add Soplang to PATH"; GroupDescription: "System settings:"
Name: "fileassociation"; Description: "Associate Soplang with .sop and .so files"; GroupDescription: "File associations:"

[Files]
Source: "..\dist\soplang\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "soplang_icon.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "file_association.reg"; DestDir: "{app}"; Flags: ignoreversion
Source: "soplang_cmd.bat"; DestDir: "{app}"; Flags: ignoreversion
Source: "soplang_launcher.bat"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Soplang Interpreter"; Filename: "{app}\soplang_launcher.bat"; IconFilename: "{app}\soplang_icon.ico"; Comment: "Run Soplang Interactive Shell"; WorkingDir: "{app}"
Name: "{group}\Soplang Command Prompt"; Filename: "{sys}\cmd.exe"; Parameters: "/k set ""PATH={app};%PATH%"" && cd /d ""{app}"""; IconFilename: "{app}\soplang_icon.ico"; Comment: "Open a command prompt with Soplang in the path"; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Soplang Interpreter"; Filename: "{app}\soplang_launcher.bat"; IconFilename: "{app}\soplang_icon.ico"; Tasks: desktopicon; WorkingDir: "{app}"

[Registry]
; Primary file extension (.sop)
Root: HKCR; Subkey: ".sop"; ValueType: string; ValueName: ""; ValueData: "SoplangFile"; Flags: uninsdeletevalue; Tasks: fileassociation
Root: HKCR; Subkey: ".sop"; ValueType: string; ValueName: "Content Type"; ValueData: "application/x-soplang"; Tasks: fileassociation
Root: HKCR; Subkey: ".sop"; ValueType: string; ValueName: "PerceivedType"; ValueData: "text"; Tasks: fileassociation

; Secondary file extension (.so)
Root: HKCR; Subkey: ".so"; ValueType: string; ValueName: ""; ValueData: "SoplangFile"; Flags: uninsdeletevalue; Tasks: fileassociation
Root: HKCR; Subkey: ".so"; ValueType: string; ValueName: "Content Type"; ValueData: "application/x-soplang"; Tasks: fileassociation
Root: HKCR; Subkey: ".so"; ValueType: string; ValueName: "PerceivedType"; ValueData: "text"; Tasks: fileassociation

; File type
Root: HKCR; Subkey: "SoplangFile"; ValueType: string; ValueName: ""; ValueData: "Soplang Source Code"; Flags: uninsdeletekey; Tasks: fileassociation
Root: HKCR; Subkey: "SoplangFile"; ValueType: string; ValueName: "FriendlyTypeName"; ValueData: "Soplang Source Code"; Tasks: fileassociation
Root: HKCR; Subkey: "SoplangFile\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\soplang_icon.ico"; Tasks: fileassociation

; Shell commands
Root: HKCR; Subkey: "SoplangFile\shell"; ValueType: string; ValueName: ""; ValueData: "open"; Tasks: fileassociation
Root: HKCR; Subkey: "SoplangFile\shell\open"; ValueType: string; ValueName: ""; ValueData: "Run with Soplang"; Tasks: fileassociation
Root: HKCR; Subkey: "SoplangFile\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: fileassociation
Root: HKCR; Subkey: "SoplangFile\shell\edit"; ValueType: string; ValueName: ""; ValueData: "Edit Soplang Source"; Tasks: fileassociation
Root: HKCR; Subkey: "SoplangFile\shell\edit\command"; ValueType: string; ValueName: ""; ValueData: "notepad.exe ""%1"""; Tasks: fileassociation

; Application registration
Root: HKCU; Subkey: "Software\Classes\Applications\{#MyAppExeName}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Applications\{#MyAppExeName}\SupportedTypes"; ValueType: string; ValueName: ".sop"; ValueData: ""
Root: HKCU; Subkey: "Software\Classes\Applications\{#MyAppExeName}\SupportedTypes"; ValueType: string; ValueName: ".so"; ValueData: ""

[Code]
const
  EnvironmentKey = 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';

procedure EnvAddPath(Path: string);
var
  Paths: string;
begin
  { Retrieve current path }
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths) then
    Paths := '';

  { Skip if already in path }
  if Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';') > 0 then exit;

  { Add path to end }
  Paths := Paths + ';' + Path;

  { Overwrite path environment variable }
  RegWriteStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    if IsTaskSelected('addtopath') then
      EnvAddPath(ExpandConstant('{app}'));
end;

procedure RemovePath(Path: string);
var
  Paths: string;
  P: Integer;
begin
  { Retrieve current path }
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths) then
    Paths := '';

  { Skip if not found in path }
  P := Pos(';' + Uppercase(Path) + ';', ';' + Uppercase(Paths) + ';');
  if P = 0 then exit;

  { Update path }
  Delete(Paths, P - 1, Length(Path) + 1);

  { Overwrite path environment variable }
  RegWriteStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    RemovePath(ExpandConstant('{app}'));
end;
