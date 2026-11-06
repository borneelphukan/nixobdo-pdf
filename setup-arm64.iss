; Inno Setup script for the ARM64 Windows installer (Windows 11 on ARM).
; Built from a prebuilt binary staged at target\release\nixobdo-pdf.exe.
; Requires Inno Setup 6.3 or newer for the arm64 architecture identifier.

[Setup]
AppName=nixobdo-pdf
AppVersion=0.1.42
AppPublisher=Borneel Bikash Phukan
AppPublisherURL=https://borneelphukan.github.io/nixobdo-pdf/
DefaultDirName={autopf}\nixobdo-pdf
DefaultGroupName=nixobdo-pdf
OutputBaseFilename=nixobdo-pdfSetup-arm64
Compression=lzma2
SolidCompression=yes
ArchitecturesAllowed=arm64
ArchitecturesInstallIn64BitMode=arm64
OutputDir=Output
SetupIconFile=assets\logo.ico

[Files]
Source: "target\release\nixobdo-pdf.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "pdfium.dll"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\nixobdo-pdf"; Filename: "{app}\nixobdo-pdf.exe"
Name: "{autodesktop}\nixobdo-pdf"; Filename: "{app}\nixobdo-pdf.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"

[Run]
Filename: "{app}\nixobdo-pdf.exe"; Description: "Launch nixobdo-pdf"; Flags: nowait postinstall skipifsilent
