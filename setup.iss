[Setup]
AppName=PDFViewer
AppVersion=0.1.0
DefaultDirName={autopf}\PDFViewer
DefaultGroupName=PDFViewer
OutputBaseFilename=PDFViewerSetup
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64
OutputDir=Output
SetupIconFile=assets\logo.ico

[Files]
Source: "target\release\PDFViewer.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "pdfium.dll"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\PDFViewer"; Filename: "{app}\PDFViewer.exe"
Name: "{autodesktop}\PDFViewer"; Filename: "{app}\PDFViewer.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional icons:"

[Run]
Filename: "{app}\PDFViewer.exe"; Description: "Launch PDFViewer"; Flags: nowait postinstall skipifsilent
