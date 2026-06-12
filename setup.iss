[Setup]
AppName=nixobdo-pdf
AppVersion=0.1.26
AppPublisher=Borneel Bikash Phukan
AppPublisherURL=https://borneelphukan.github.io/nixobdo-pdf/
DefaultDirName={autopf}\nixobdo-pdf
DefaultGroupName=nixobdo-pdf
OutputBaseFilename=nixobdo-pdfSetup
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64
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
