; Inno Setup script for the 32-bit (x86) Windows installer.
; Built from a prebuilt binary staged at target\release\nixobdo-pdf.exe.
; Installs in 32-bit mode so it works on older 32-bit Windows.

[Setup]
AppName=nixobdo-pdf
AppVersion=0.1.42
AppPublisher=Borneel Bikash Phukan
AppPublisherURL=https://borneelphukan.github.io/nixobdo-pdf/
DefaultDirName={autopf}\nixobdo-pdf
DefaultGroupName=nixobdo-pdf
OutputBaseFilename=nixobdo-pdfSetup-x86
Compression=lzma2
SolidCompression=yes
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
