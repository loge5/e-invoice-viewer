; Inno Setup script of the Windows installer (spec 0022, ADR 0005).
; Installs the folder built by bundle.sh for the current user, without
; administrator rights, and registers the app under "Open with" for .pdf and
; .xml without ever becoming the default application.
;
; ISCC /DVersion=<version> scripts\windows\installer.iss

#ifndef Version
  #error Pass the version: ISCC /DVersion=<version>
#endif

[Setup]
AppId={{CEB96F2E-D33D-413B-B72A-E77F78EC50CB}
AppName=E-Invoice Viewer
AppVersion={#Version}
AppVerName=E-Invoice Viewer {#Version}
AppPublisher=Rolf Loges
AppPublisherURL=https://github.com/loge5/e-invoice-viewer
AppSupportURL=https://github.com/loge5/e-invoice-viewer/issues
DefaultDirName={localappdata}\Programs\E-Invoice Viewer
DisableDirPage=yes
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; Windows 10 1809
MinVersion=10.0.17763
ChangesAssociations=yes
OutputDir=..\..\target\windows
OutputBaseFilename=E-Invoice_Viewer-{#Version}-windows-x86_64-setup
SetupIconFile=..\..\data\icons\de.loges.EInvoiceViewer.ico
UninstallDisplayIcon={app}\bin\e-invoice-viewer.exe
UninstallDisplayName=E-Invoice Viewer
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
; The EU languages Inno Setup ships a translation for (spec 0030); the
; others (el, et, ga, hr, lt, lv, mt, ro) install in English.
Name: "bg"; MessagesFile: "compiler:Languages\Bulgarian.isl"
Name: "cs"; MessagesFile: "compiler:Languages\Czech.isl"
Name: "da"; MessagesFile: "compiler:Languages\Danish.isl"
Name: "de"; MessagesFile: "compiler:Languages\German.isl"
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "fi"; MessagesFile: "compiler:Languages\Finnish.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"
Name: "hu"; MessagesFile: "compiler:Languages\Hungarian.isl"
Name: "it"; MessagesFile: "compiler:Languages\Italian.isl"
Name: "nl"; MessagesFile: "compiler:Languages\Dutch.isl"
Name: "pl"; MessagesFile: "compiler:Languages\Polish.isl"
Name: "pt"; MessagesFile: "compiler:Languages\Portuguese.isl"
Name: "sk"; MessagesFile: "compiler:Languages\Slovak.isl"
Name: "sl"; MessagesFile: "compiler:Languages\Slovenian.isl"
Name: "sv"; MessagesFile: "compiler:Languages\Swedish.isl"

[InstallDelete]
; An update replaces the program files completely (no stale DLLs).
Type: filesandordirs; Name: "{app}\bin"
Type: filesandordirs; Name: "{app}\lib"
Type: filesandordirs; Name: "{app}\share"

[Files]
Source: "..\..\target\windows\E-Invoice Viewer\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\E-Invoice Viewer"; Filename: "{app}\bin\e-invoice-viewer.exe"

[Registry]
; The application, as offered under "Open with".
Root: HKCU; Subkey: "Software\Classes\Applications\e-invoice-viewer.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "E-Invoice Viewer"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Applications\e-invoice-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".pdf"; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Applications\e-invoice-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".xml"; ValueData: ""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\Applications\e-invoice-viewer.exe\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\bin\e-invoice-viewer.exe"" ""%1"""; Flags: uninsdeletekey
; A file type of its own, listed for .pdf and .xml; the extensions' default
; (the default application) is never touched.
Root: HKCU; Subkey: "Software\Classes\EInvoiceViewer.Document"; ValueType: string; ValueName: ""; ValueData: "E-Invoice"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\EInvoiceViewer.Document\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\bin\e-invoice-viewer.exe,0"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\EInvoiceViewer.Document\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\bin\e-invoice-viewer.exe"" ""%1"""; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\.pdf\OpenWithProgids"; ValueType: string; ValueName: "EInvoiceViewer.Document"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.xml\OpenWithProgids"; ValueType: string; ValueName: "EInvoiceViewer.Document"; ValueData: ""; Flags: uninsdeletevalue

[Run]
Filename: "{app}\bin\e-invoice-viewer.exe"; Description: "{cm:LaunchProgram,E-Invoice Viewer}"; Flags: nowait postinstall skipifsilent
