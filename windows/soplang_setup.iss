#define MyAppName "Soplang"
#ifndef MyAppVersion
#define MyAppVersion "0.0.0"
#endif
#ifndef MyNumericVersion
#define MyNumericVersion MyAppVersion
#endif
#define MyAppPublisher "Soplang Software Foundation"
#define MyAppURL "https://www.soplang.org/"
#define MyAppExeName "soplang.exe"
#if FileExists("soplang_icon.ico")
#define MyIconName "soplang_icon.ico"
#else
#define MyIconName MyAppExeName
#endif

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
; Do not use the same AppId value in installers for other applications.
AppId={{F1C77F9E-F26A-4D23-9A8B-CF3D26AE5A18}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} Compiler
UninstallDisplayName={#MyAppName} Compiler
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
ChangesAssociations=yes
#if FileExists("soplang_icon.ico")
SetupIconFile=soplang_icon.ico
#endif
UninstallDisplayIcon={app}\soplang.exe
WizardStyle=modern
VersionInfoVersion={#MyNumericVersion}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription=Installer for {#MyAppName}
VersionInfoCopyright=© 2025 {#MyAppPublisher}
VersionInfoProductName={#MyAppName} Compiler
VersionInfoProductVersion={#MyAppVersion}


[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "addtopath"; Description: "Add Soplang to PATH"; GroupDescription: "System settings:"
Name: "fileassociation"; Description: "Associate Soplang with .sop and .so files"; GroupDescription: "File associations:"

[Files]
Source: "..\dist\soplang\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
#if FileExists("soplang_icon.ico")
Source: "soplang_icon.ico"; DestDir: "{app}"; Flags: ignoreversion
#endif
Source: "soplang_cmd.bat"; DestDir: "{app}"; Flags: ignoreversion
Source: "soplang_launcher.bat"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Soplang Interpreter"; Filename: "{app}\soplang_launcher.bat"; IconFilename: "{app}\{#MyIconName}"; Comment: "Run Soplang Interactive Shell"; WorkingDir: "{app}"
Name: "{group}\Soplang Command Prompt"; Filename: "{sys}\cmd.exe"; Parameters: "/k set ""PATH={app};%PATH%"" && cd /d ""{app}"""; IconFilename: "{app}\{#MyIconName}"; Comment: "Open a command prompt with Soplang in the path"; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Soplang Interpreter"; Filename: "{app}\soplang_launcher.bat"; IconFilename: "{app}\{#MyIconName}"; Tasks: desktopicon; WorkingDir: "{app}"

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
Root: HKCR; Subkey: "SoplangFile\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#MyIconName}"; Tasks: fileassociation

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

function PathEntryMatches(Entry, Path: string): Boolean;
begin
  { Resolve 8.3 aliases while the installation directory still exists. }
  Result := CompareText(
    GetShortName(RemoveBackslashUnlessRoot(RemoveQuotes(Trim(Entry)))),
    GetShortName(RemoveBackslashUnlessRoot(Path))) = 0;
end;

function FindPathEntry(Paths, Path: string; var First, Last: Integer): Boolean;
begin
  First := 1;
  while First <= Length(Paths) do begin
    Last := First;
    while (Last <= Length(Paths)) and (Paths[Last] <> ';') do
      Last := Last + 1;
    if PathEntryMatches(Copy(Paths, First, Last - First), Path) then begin
      Result := True;
      exit;
    end;
    First := Last + 1;
  end;
  Result := False;
end;

procedure EnvAddPath(Path: string);
var
  Paths: string;
  First, Last: Integer;
begin
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths) then
    Paths := '';
  if FindPathEntry(Paths, Path, First, Last) then exit;

  if Paths <> '' then Paths := Paths + ';';
  Paths := Paths + Path;
  if not RegWriteExpandStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths) then
    RaiseException('Could not add Soplang to PATH');
  Log('Added Soplang PATH entry: ' + Path);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
    if WizardIsTaskSelected('addtopath') then
      EnvAddPath(ExpandConstant('{app}'));
end;

procedure RemovePath(Path: string);
var
  Paths: string;
  First, Last: Integer;
begin
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths) then exit;
  if not FindPathEntry(Paths, Path, First, Last) then begin
    Log('Soplang PATH entry not found: ' + Path);
    exit;
  end;

  { Remove this entry and one separator, preserving every other entry. }
  if First > 1 then
    Delete(Paths, First - 1, Last - First + 1)
  else if Last <= Length(Paths) then
    Delete(Paths, First, Last - First + 1)
  else
    Paths := '';

  if not RegWriteExpandStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', Paths) then
    RaiseException('Could not remove Soplang from PATH');
  Log('Removed Soplang PATH entry: ' + Path);
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usUninstall then
    RemovePath(ExpandConstant('{app}'));
end;
