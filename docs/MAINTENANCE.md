# Guía de mantenimiento

Objetivo: que cualquier persona, o una sesión nueva de Claude sin contexto, pueda mantener AlumFrame. Se completa en H7; lo que ya existe está al día.

## 1. Requisitos y compilación
- Rust fijado en `rust-toolchain.toml` (rustup lo instala solo), `just` y `cargo-deny` (`cargo install just cargo-deny --locked`).
- Node 22 solo para los E2E (`e2e/`); el ejecutable no lo necesita.
- `just check` (formato, clippy, pruebas y tamaño de archivos) · `just build` · `just dev` (modo local en `datos-dev/`).
- Activa el hook previo al push una vez por clon: `just instalar-hooks`.
- Si `just dev` dice que la base es «de otra versión de desarrollo», el esquema ha cambiado: borra `datos-dev/` (ADR-0011).

## 2. Pruebas
- `just test`: unitarias e integración (SQLite en memoria).
- `just e2e`: compila en release y ejecuta Playwright contra el ejecutable, con una carpeta de datos temporal y el puerto 9191.
- `just cobertura`: cobertura del dominio (necesita `cargo install cargo-llvm-cov` y `rustup component add llvm-tools-preview`).

## 3. Publicar una versión
1. Sube `version` en `Cargo.toml` (`[workspace.package]`) y anota el cambio en `docs/CAMBIOS.md`.
2. `just check` y `just e2e` en verde.
3. Lleva el cambio a `main`. El workflow `publicar.yml` ve que esa versión aún no tiene etiqueta, pasa las pruebas, compila y crea la versión `vX.Y.Z` con `alumframe-vX.Y.Z-linux-x86_64.tar.gz` y `alumframe-vX.Y.Z-windows-x86_64.zip`. Un push a `main` sin cambiar la versión no publica nada.
4. También se puede lanzar a mano en Actions > «Publicar versión» > *Run workflow*.

## 4. Instalar y actualizar en el taller
**Servidor Linux:** descarga y descomprime el `.tar.gz`, entra en la carpeta y ejecuta `sudo ./instalar.sh`:
```
curl -LO https://github.com/Alexzafra13/Aluframe/releases/download/vX.Y.Z/alumframe-vX.Y.Z-linux-x86_64.tar.gz
tar xzf alumframe-vX.Y.Z-linux-x86_64.tar.gz && cd alumframe-vX.Y.Z-linux-x86_64 && sudo ./instalar.sh
```
Si el repositorio vuelve a ser privado, `curl` baja un archivo de 9 bytes («Not Found»): usa `gh release download vX.Y.Z -R Alexzafra13/Aluframe -p '*linux*'` tras `gh auth login`.
 Instala en `/opt/alumframe`, configura en `/etc/alumframe/alumframe.toml`, guarda los datos en `/var/lib/alumframe` y deja el servicio `alumframe` activo al arrancar. Comprueba `http://IP:9090/health`.
- Actualizar: lo mismo con la versión nueva; la anterior queda en `/opt/alumframe/alumframe.anterior`.
- Volver atrás: ver `despliegue/linux/LEEME.txt`. (Hasta el primer despliegue real no hay migraciones: ADR-0011.)
- Registros: `journalctl -u alumframe`. El instalador espera a que `/health` responda y, si no arranca, muestra los últimos registros.
- **Actualizar ya no borra datos** (desde la 0.11.0, ADR-0021): la base se actualiza sola al arrancar, con una copia previa. **Borrar los datos de prueba** solo hace falta con una base de un desarrollo anterior a la 0.8.0. **Primero instala la versión nueva** y después: `sudo systemctl stop alumframe`, `sudo find /var/lib/alumframe -mindepth 1 -delete` y `sudo systemctl start alumframe`. No sirve `sudo rm -rf /var/lib/alumframe/*`: el `*` lo expande tu usuario, que no puede leer la carpeta, y no se borra nada. Si se borra antes de instalar, la versión vieja vuelve a crear la base y la nueva no arranca («La base de datos es de otra versión»).
- **Series:** las de `data/series/` vienen dentro del ejecutable y se cargan solas al arrancar (ADR-0014); no hay que importarlas.
- **Acceso desde fuera del taller:** con **Tailscale**, sin abrir puertos en el router. En el servidor, `curl -fsSL https://tailscale.com/install.sh | sh`, `sudo tailscale up` y `sudo systemctl enable --now tailscaled`; en el móvil o portátil, la app Tailscale con la misma cuenta, y se abre `http://IP-TAILSCALE:9090` (`tailscale ip -4`). Con `ufw`: `sudo ufw allow in on tailscale0 to any port 9090 proto tcp` y `sudo ufw allow from 192.168.1.0/24 to any port 9090 proto tcp`. No se publica con un proxy inverso: el bloqueo por IP vería todas las peticiones con la IP del proxy.

**PC con Windows:** descarga `AlumFrame-vX.Y.Z-windows-instalar.exe` de la página de versiones y ejecútalo (no pide administrador; la primera vez SmartScreen avisa porque no está firmado: «Más información» → «Ejecutar de todas formas»). Crea los iconos «AlumFrame» y «AlumFrame · cambiar conexión». Al abrirlo se elige **en este equipo** o **servidor del taller** (ADR-0015).
- Datos, registro y preferencias: `%LOCALAPPDATA%\AlumFrame` (`datos\`, `alumframe.log`, `escritorio.toml`). Desinstalar no los borra.
- Actualizar: ejecutar el instalador de la versión nueva con AlumFrame cerrado.
- **Si no arranca por la base de datos** (de una versión de desarrollo antigua, o de una más nueva que la instalada), la ventana lo explica, dice cuál es la carpeta de datos y ofrece «Abrir la carpeta de datos». Con una base de desarrollo antigua ofrece también «Empezar de cero»: la base no se borra, se mueve a la subcarpeta `apartadas` (con la fecha en el nombre) y al volver a abrir AlumFrame empieza vacío. Con una base más nueva no lo ofrece: hay que instalar la versión más reciente. Las copias y la demostración no se tocan.
- Contraseña olvidada en modo local: en PowerShell, `$env:ALUMFRAME_MODO="local"; $env:ALUMFRAME_CARPETA_DATOS="$env:LOCALAPPDATA\AlumFrame\datos"; & "$env:LOCALAPPDATA\Programs\AlumFrame\alumframe-consola.exe" restablecer-contrasena "Nombre"` (la carpeta de instalación puede variar: es la del icono).

**Pasar datos entre equipos:** Ajustes → Datos → «Exportar datos» en el origen y «Importar» en el destino (ADR-0016). Importar **sustituye** los datos; antes guarda una copia de lo que había en la carpeta `copias` de los datos. Se puede importar una copia de la misma versión o de una anterior (se actualiza al importarla), no de una más nueva.

## 5. Copias de seguridad (ADR-0022)
- Solas: cada día a las 14:00, al cerrar el programa (o parar el servicio) y al arrancar si faltó la de las 14:00. Se guardan en `copias/automaticas` de la carpeta de datos y en cada disco conectado con una carpeta `AlumFrame-copias` en su raíz: 7 diarias y 4 semanales.
- **Preparar un pendrive o disco:** conéctalo y en Ajustes > Copias, «Usar este disco para las copias» (o crea a mano la carpeta `AlumFrame-copias`). Conviene tener dos y alternarlos, con uno fuera del taller.
- **En el servidor Linux** (sin escritorio los discos no se montan solos): pasos en `despliegue/linux/LEEME.txt` (montaje en `/mnt/copias` con `/etc/fstab` y `nofail`). Otra ruta: `discos_de_copia` en `alumframe.toml` y en `ReadWritePaths` del servicio.
- **Avisos:** el Inicio avisa si la última copia falló, si no hay disco o si la última copia en un disco tiene 3 días o más. Detalle en Ajustes > Copias y en el registro (`journalctl -u alumframe`).
- **Restaurar:** Ajustes > Copias > «Restaurar…» en la copia elegida; se ve qué contiene antes de confirmar y lo actual se guarda antes. En un equipo nuevo: instala, crea el propietario, conecta el disco y restaura (o Ajustes > Datos > Importar con el archivo `.alumframe`). Una copia de una versión anterior se actualiza al restaurarla.
- Antes de cada actualización que cambie la base se guarda `copias/antes-de-actualizar-….alumframe`.
- **Nunca copies `alumframe.db` con el programa en marcha.**

## 6. Operaciones frecuentes
- Dar de alta operarios, cambiar su PIN o desactivarlos: Ajustes > Usuarios.
- Grosor de sierra, recorte de extremos y retal mínimo: Ajustes > Taller.
- Contraseña del propietario olvidada (en el servidor): `sudo -u alumframe ALUMFRAME_CONFIG=/etc/alumframe/alumframe.toml /opt/alumframe/alumframe restablecer-contrasena "Nombre"`.
- **Añadir o corregir una serie:** edita o añade su YAML en `data/series/` (formato en `docs/FORMATO_SERIES.md`), pasa las pruebas y publica una versión: al arrancar entra sola como versión nueva (ADR-0014). Para probar un cambio sin publicar, impórtalo en Catálogo > Series > Importar serie. La vista previa muestra errores y avisos; cada importación crea una versión nueva y las anteriores se conservan.
- **Colores (acabados):** se corrigen o añaden en `data/acabados.yaml` (nombre, RAL, color `#rrggbb` del dibujo, `madera`, `cierres: blanco|negro`); al arrancar la versión nueva se aplican sin tocar el recargo que haya puesto el propietario (ADR-0019).
- **Añadir siluetas de perfiles** (ADR-0017): con el catálogo PDF del fabricante, `python3 scripts/extraer_siluetas.py catalogo.pdf 4-6 /tmp/siluetas` (necesita poppler: `pdftocairo` y `pdftotext`). Revisa `indice.txt`, renombra cada archivo a su referencia comprobándola en el catálogo, cópialo a `data/perfiles/<fabricante>/` y añade su línea al `indice.txt` de esa carpeta. En la serie: `MARCO_LATERAL: { referencia: …, silueta: aluval/21210 }`.
- **Añadir el dibujo de un herraje** (ADR-0029): si la ficha PDF es vectorial, `python3 scripts/extraer_herraje.py ficha.pdf <página> x0,y0,x1,y1 data/perfiles/herrajes/<nombre>.svg` (recuadro en puntos PDF; `--tumbado` para un herraje largo). Para medir el recuadro, `pdftocairo -png -r 72 -f N -l N ficha.pdf pagina` (un píxel es un punto). Si el catálogo solo trae la imagen, sácala sin pérdida con `pdfimages -png` y pásala con `python3 scripts/imagen_de_herraje.py imagen.png data/perfiles/herrajes/<nombre>.svg` (necesita Pillow; `--recorte`, `--quitar-azul` y `--borrar` para quitar cotas, textos o tornillos de una foto o un plano). Anota el origen en `data/perfiles/herrajes/indice.txt` y cítalo en la serie: `CIERRE_EMBUTIDO: { referencia: CIERRE-EMBUTIDO, silueta: herrajes/cierre-embutido }`. Para el logotipo de un fabricante, un PNG con fondo transparente en `data/marcas/<fabricante en minúsculas>.png`.
- Antes de tocar una serie real, comprueba sus ventanas con «Probar una ventana» y que `just test` sigue en verde: los tests dorados comparan `data/series/` con `fixtures/real/`.
- **Actualizar una tarifa:** Catálogo > Tarifas > Importar tarifa. Elige el proveedor y el CSV con las columnas `referencia;descripcion;unidad;precio_tarifa;longitud_barra` (y `tipo` opcional). Desde Excel, «Guardar como» → CSV. Revisa la vista previa (los saltos de más del 30 % salen marcados) y confirma. Si algo no cuadra, «Deshacer» en la versión vigente.
- **Cambiar unos pocos precios:** Catálogo > Artículos, filtra por proveedor y escribe el precio en la tabla, o marca artículos y aplica un porcentaje. Cada cambio queda como una versión de la tarifa.
- **Actualizar los códigos postales** (ADR-0020): `curl -o /tmp/ES.zip https://download.geonames.org/export/zip/ES.zip` y `python3 scripts/generar_codigos_postales.py /tmp/ES.zip data/codigos_postales.tsv`; revisa la diferencia con `git diff --stat`, pasa las pruebas y publica una versión.
- **Actualizar el callejero** (ADR-0023; el INE publica uno en enero y otro en julio): mira el nombre del último en https://www.ine.es/prodyser/callejero/ («Ficheros nacionales»), `curl -o /tmp/caj.zip https://www.ine.es/prodyser/callejero/caj_esp/caj_esp_072025.zip` (cambia la fecha) y `python3 scripts/generar_callejero.py /tmp/caj.zip 06 "julio de 2025" data/callejero/06.tsv`. Cambia también la fecha de la cita en `plantillas/ajustes_datos.html`. Si el script avisa de un tipo de vía desconocido, añádelo a `TIPOS`. Revisa `git diff --stat`, pasa las pruebas y publica. **Otra provincia:** genera `data/callejero/<código>.tsv` con su código y añádelo en `crates/servidor/src/aplicacion/calles.rs`.
- **Logotipo y datos de la empresa** para los documentos: Ajustes > Empresa y precios (PNG o JPG de hasta 1 MB; sin logotipo sale el nombre en grande).
- **Importar clientes** de una hoja de cálculo: Clientes > Importar clientes (columnas en ADR-0012, todas opcionales).
- **Petición RGPD de un cliente:** en su ficha, «Exportar datos» (acceso) o «Borrar definitivamente» (supresión).
- **Margen, cobro de perfiles, mano de obra, IVA y datos de la empresa:** Ajustes > Empresa y precios. Lo que se deje sin elegir hace que las ventanas salgan «incompletas» y lo dicen.
- **Stock:** empieza con una entrada «Inventario inicial» de cada artículo (las barras enteras y, aparte, cada retal con su largo en cm). Se descuenta solo al terminar cada orden (ficha del trabajo > «Terminar orden y descontar stock»). Las correcciones, en Stock > Ajuste, con su motivo. El mínimo de cada artículo, en su ficha de stock.
- **Cambiar la base de datos** (para quien desarrolla): migración nueva en `crates/servidor/src/bd/migraciones/`, nunca editar las publicadas (ADR-0021, «Cómo se añade una migración»).
- **Taller:** al aceptar un presupuesto aparece su trabajo en Inicio y en Trabajos. En la ficha del trabajo se asigna el operario (y la fecha prevista), se imprime la orden de taller y se modifican medidas. Los pasos de cada ventana se cambian en Ajustes > Taller (valen para los trabajos nuevos).
- **Presupuestos:** si cambia una tarifa, el presupuesto en borrador avisa y ofrece «Recalcular precios». Un presupuesto aceptado no se toca: se guarda congelado con su huella.

## 6 bis. Demostración (ADR-0028)
- **Para probar o aprender** sin tocar los datos: «Probar la demostración» en la pantalla de entrada (o en Ajustes > Datos). Se entra directamente como «Dueño (demo)», sin contraseña; con «Cambiar de usuario» se elige «Juan (demo)» para ver la pantalla del operario, también sin PIN. Se sale con el botón de la franja naranja.
- Va en `demostracion/alumframe.db` dentro de la carpeta de datos, con su marca: no entra en las copias, no se exporta y no se puede importar en los datos de verdad. «Empezar de nuevo» (dueño de la demostración) la deja como al principio. Para borrarla del todo, con el programa parado, se borra esa subcarpeta; al volver a entrar se rellena otra vez.
- Los datos se generan en `crates/servidor/src/aplicacion/demostracion/` con los mismos casos de uso del programa; la tarifa inventada está en `fixtures/demo/tarifa-demo-aluprom-21.csv`. Si un cambio rompe la demostración, falla `tests/demostracion.rs`.

## 7. Actualizar el toolchain y las dependencias
Cada 6 meses o ante un aviso de seguridad:
1. Cambia `rust-toolchain.toml` si procede.
2. `cargo update` y después `just antiguedad`: lista lo publicado hace menos de 14 días. Bájalo con `cargo update -p nombre@version --precise version-anterior`.
3. `cargo deny check`, `just check` y `just e2e`; prueba con una copia de los datos reales.

## 8. Si la base de datos falla
`sqlite3 alumframe.db "PRAGMA integrity_check"` con el servicio parado. Si no responde `ok`, restaura la última copia (apartado 5).

## 9. Dónde están las decisiones
`docs/adr/`, `docs/CONTEXTO.md`, `docs/PROGRESS.md` y `docs/CAMBIOS.md`.

## 10. Cómo retomar el proyecto con Claude
Lee `CLAUDE.md` y `docs/PROGRESS.md`, ejecuta `just check` y sigue el siguiente hito pendiente.
