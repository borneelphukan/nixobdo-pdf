; nixobdo-pdf - Inno Setup installer
;
; Compiled by .github/workflows/release.yml from the repository root, for
; example:
;   ISCC.exe /DMyAppVersion=0.1.42 /DMyVersionInfo=0.1.42.0 /DMyArch=x64 ^
;            /DMySourceExe=...\target\x86_64-pc-windows-msvc\release\nixobdo-pdf.exe ^
;            /DMyPdfiumDll=...\pdfium.dll ^
;            /DMyOutputBaseFilename=nixobdo-pdfSetup ^
;            installer\nixobdo-pdf.iss
;
; The x64 output keeps the historical `nixobdo-pdfSetup.exe` name because the
; in-app updater downloads that exact asset from the latest GitHub release.

#ifndef MyAppVersion
  #define MyAppVersion "0.0.0"
#endif
#ifndef MyVersionInfo
  #define MyVersionInfo "0.0.0.0"
#endif
#ifndef MyArch
  #define MyArch "x64"
#endif
#ifndef MySourceExe
  #define MySourceExe "..\target\release\nixobdo-pdf.exe"
#endif
#ifndef MyPdfiumDll
  #define MyPdfiumDll "pdfium.dll"
#endif
#ifndef MyOutputBaseFilename
  #define MyOutputBaseFilename "nixobdo-pdfSetup"
#endif

[Setup]
AppId={{6D2F5A81-9C3B-4E72-B1D4-7A8E5F0C3B29}
AppName=nixobdo-pdf
AppVersion={#MyAppVersion}
AppVerName=nixobdo-pdf {#MyAppVersion}
AppPublisher=Borneel Bikash Phukan
AppPublisherURL=https://borneelphukan.github.io/nixobdo-pdf/
AppSupportURL=https://github.com/borneelphukan/nixobdo-pdf/issues
AppUpdatesURL=https://github.com/borneelphukan/nixobdo-pdf/releases
DefaultDirName={autopf}\nixobdo-pdf
DefaultGroupName=nixobdo-pdf
DisableProgramGroupPage=yes
OutputDir=..\Output
OutputBaseFilename={#MyOutputBaseFilename}
SetupIconFile=..\assets\logo.ico
UninstallDisplayIcon={app}\nixobdo-pdf.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
VersionInfoVersion={#MyVersionInfo}
VersionInfoProductVersion={#MyVersionInfo}
VersionInfoProductName=nixobdo-pdf
VersionInfoCompany=Borneel Bikash Phukan
VersionInfoDescription=nixobdo-pdf Setup
#if MyArch == "x64"
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
#elif MyArch == "arm64"
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
#else
ArchitecturesAllowed=x86compatible
#endif

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"; Flags: unchecked

[Files]
Source: "{#MySourceExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#MyPdfiumDll}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}\docs"; Flags: ignoreversion skipifsourcedoesntexist
Source: "..\README.md"; DestDir: "{app}\docs"; Flags: ignoreversion skipifsourcedoesntexist

[Icons]
Name: "{group}\nixobdo-pdf"; Filename: "{app}\nixobdo-pdf.exe"
Name: "{group}\Documentation"; Filename: "{app}\docs\README.md"
Name: "{group}\Uninstall nixobdo-pdf"; Filename: "{uninstallexe}"
Name: "{autodesktop}\nixobdo-pdf"; Filename: "{app}\nixobdo-pdf.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\nixobdo-pdf.exe"; Description: "Launch nixobdo-pdf"; Flags: nowait postinstall skipifsilent
