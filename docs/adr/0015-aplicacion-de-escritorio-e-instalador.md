# ADR-0015: aplicación de escritorio para Windows e instalador

**Estado:** aceptado (2026-11-22, decisión del propietario).

## Contexto
El paquete de Windows era un `.zip` con `alumframe.exe` en modo local, que dejaba una consola abierta y usaba el navegador normal, más un script de PowerShell para crear un acceso directo al servidor. El propietario quiere un programa «de toda la vida»: instalador, icono, una ventana propia sin consola y que se cierre al cerrar la ventana. Y lo quiere como en otros programas del sector: **la misma aplicación** sirve para trabajar solo en ese PC o conectada al servidor Linux del taller, y se elige dentro de ella.

## Decisión
1. **Ejecutable nuevo `alumframe-escritorio`** (en el instalador, `AlumFrame.exe`), del mismo crate `servidor`. Lleva `windows_subsystem = "windows"`: sin consola. `alumframe` sigue igual: es el servidor de Linux y la herramienta de consola (en Windows se instala como `alumframe-consola.exe`, porque Windows no distingue mayúsculas).
2. **Al abrirse pregunta dónde trabajar** (`/escritorio/conexion`) y lo recuerda en `%LOCALAPPDATA%\AlumFrame\escritorio.toml`:
   - **En este equipo:** arranca el programa en modo local (solo `127.0.0.1`, puerto 9090), con los datos en `%LOCALAPPDATA%\AlumFrame\datos`.
   - **Servidor del taller:** abre la dirección del servidor (se completa a `http://IP:9090`). En este equipo no queda nada en marcha.
   - El acceso del menú Inicio «AlumFrame · cambiar conexión» (`--elegir`) vuelve a preguntar.
3. **La ventana es Edge (o Chrome) en modo aplicación** (`--app`, perfil propio en `%LOCALAPPDATA%\AlumFrame\ventana`): sin pestañas ni barra de direcciones. Con un perfil propio, el proceso lanzado es el de la ventana; cuando termina, el programa se detiene de forma ordenada. Un segundo clic en el icono, con el programa ya abierto, solo abre otra ventana (se comprueba que en el puerto contesta AlumFrame y no otro programa).
4. **Sin consola, los errores van a un registro** (`%LOCALAPPDATA%\AlumFrame\alumframe.log`, se reinicia al pasar de 5 MB) y, si no puede arrancar, se abre una ventana con el motivo.
5. **Instalador con Inno Setup**, compilado en GitHub Actions (`despliegue/windows/alumframe.iss`). Instalación por usuario, sin administrador; iconos en el escritorio y el menú Inicio; aparece en «Aplicaciones instaladas». Desinstalar **no borra los datos**.

## Requisitos del ADR-0002 para lo añadido
- **Dependencias nuevas en el programa:** ninguna. La ventana usa el Edge que trae Windows 10 y 11.
- **Inno Setup:** solo se usa al publicar, no forma parte del programa. Es gratuito (licencia propia de tipo BSD), estable desde hace más de 20 años y muy usado. Si desapareciera, el `.exe` se puede repartir suelto o con otro instalador (WiX, MSIX) sin tocar el código.
- **Pruebas:**
  - `tests/escritorio.rs` ejecuta la aplicación con un navegador falso: pregunta la primera vez, abre directamente lo elegido, no arranca nada contra el servidor y se cierra al cerrar la ventana;
  - `tests/escritorio_pantallas.rs` prueba la pantalla de elección;
  - la CI de Windows compila el instalador en cada push.
  - La ventana real de Edge solo se puede probar en un Windows.

## Alternativas descartadas
- **Ventana nativa con WebView2 (`wry`/`tao`) o Tauri:** es un programa «de verdad», sin rastro de Edge, pero añade dependencias grandes, código específico de cada sistema y mantenimiento. Se reconsidera si la ventana de Edge no convence al usarla.
- **Icono junto al reloj para cerrar el programa:** también necesita una dependencia. Cerrar la ventana ya cierra el programa.
- **Elegir el modo al instalar:** el propietario prefiere poder cambiarlo desde la aplicación.

## Consecuencias
- El `.zip` de Windows y `crear-acceso-directo.ps1` desaparecen; se publica `AlumFrame-vX.Y.Z-windows-instalar.exe`.
- El instalador no está firmado: la primera vez Windows SmartScreen avisa («Windows protegió su PC» → «Más información» → «Ejecutar de todas formas»). Firmarlo exige un certificado de pago; se decidirá si hace falta.
- Pasar datos entre «este equipo» y el servidor se hace exportando e importando (ADR-0016), no sincronizando.
