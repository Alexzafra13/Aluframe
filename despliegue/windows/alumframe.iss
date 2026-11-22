; Instalador de AlumFrame para Windows (ADR-0015). Lo compila GitHub Actions con Inno Setup:
;   ISCC.exe /DVersion=0.5.0 /DBinarios=target\release despliegue\windows\alumframe.iss
; Se instala por usuario (sin permisos de administrador); los datos viven en
; %LOCALAPPDATA%\AlumFrame y no se borran al desinstalar.

#ifndef Version
  #define Version "0.0.0"
#endif
#ifndef Binarios
  #define Binarios "..\..\target\release"
#endif

[Setup]
AppId={{6F1C2A4E-9B3D-4E5F-8A71-2C9D0E4B7A15}
AppName=AlumFrame
AppVersion={#Version}
AppPublisher=AlumFrame
DefaultDirName={autopf}\AlumFrame
DefaultGroupName=AlumFrame
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputDir=.
OutputBaseFilename=AlumFrame-{#Version}-instalar
SetupIconFile=alumframe.ico
UninstallDisplayIcon={app}\alumframe.ico
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "escritorio"; Description: "Crear un icono en el escritorio"

[Files]
; Windows no distingue mayúsculas: la herramienta de consola lleva otro nombre.
Source: "{#Binarios}\alumframe-escritorio.exe"; DestDir: "{app}"; DestName: "AlumFrame.exe"; Flags: ignoreversion
Source: "{#Binarios}\alumframe.exe"; DestDir: "{app}"; DestName: "alumframe-consola.exe"; Flags: ignoreversion
Source: "alumframe.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\AlumFrame"; Filename: "{app}\AlumFrame.exe"; IconFilename: "{app}\alumframe.ico"
Name: "{autoprograms}\AlumFrame · cambiar conexión"; Filename: "{app}\AlumFrame.exe"; Parameters: "--elegir"; IconFilename: "{app}\alumframe.ico"
Name: "{autodesktop}\AlumFrame"; Filename: "{app}\AlumFrame.exe"; IconFilename: "{app}\alumframe.ico"; Tasks: escritorio

[Run]
Filename: "{app}\AlumFrame.exe"; Description: "Abrir AlumFrame"; Flags: nowait postinstall skipifsilent
