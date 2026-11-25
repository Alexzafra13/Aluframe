# ADR-0034: el programa se llama Aluframe, como su logotipo

**Estado:** aceptado (2026-11-25, decisión del propietario).

## Contexto
El propietario aportó el logotipo del programa, que dice «ALUFRAME». El programa se llamaba «AlumFrame» en los títulos, el instalador y los accesos directos. Pidió que se llame como el logotipo.

## Decisión
1. **Lo que se ve pasa a «Aluframe»:** títulos de las páginas, texto alternativo del logo, mensajes y errores, el instalador de Windows (nombre, accesos directos y archivo `Aluframe-vX.Y.Z-windows-instalar.exe`), el nombre de la versión publicada y los textos del servicio de Linux.
2. **No cambia lo que guarda o reconoce datos**, para que actualizar no los pierda: la carpeta de datos de Windows `%LOCALAPPDATA%\AlumFrame`, la carpeta de copias de los discos `AlumFrame-copias`, la extensión `.alumframe` de las copias exportadas, el ejecutable y el servicio `alumframe`, las variables `ALUMFRAME_*`, el identificador del instalador y su carpeta de instalación (`AlumFrame.exe`), y las migraciones ya publicadas.
3. Al actualizar, el instalador borra los accesos directos viejos («AlumFrame…») para que no queden duplicados.
4. La documentación anterior conserva «AlumFrame» donde cuenta la historia; lo nuevo usa «Aluframe».

## Consecuencias
- Los pendrives de copias siguen sirviendo sin tocarlos.
- Si algún día se quisiera cambiar también la carpeta de datos, haría falta mover los datos al actualizar: queda fuera de esta decisión.
