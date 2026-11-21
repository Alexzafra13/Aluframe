# Crea en el escritorio un acceso directo que abre AlumFrame del servidor como si fuera una aplicación.
param([string]$Servidor = (Read-Host "IP o nombre del servidor del taller (por ejemplo 192.168.1.50)"))

$edge = @(
  "${env:ProgramFiles(x86)}\Microsoft\Edge\Application\msedge.exe",
  "$env:ProgramFiles\Microsoft\Edge\Application\msedge.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $edge) {
  Write-Error "No se encuentra Microsoft Edge."
  exit 1
}

$escritorio = [Environment]::GetFolderPath("Desktop")
$shell = New-Object -ComObject WScript.Shell
$acceso = $shell.CreateShortcut((Join-Path $escritorio "AlumFrame.lnk"))
$acceso.TargetPath = $edge
$acceso.Arguments = "--app=http://${Servidor}:9090"
$acceso.Description = "AlumFrame"
$acceso.Save()
Write-Host "Acceso directo creado en el escritorio."
