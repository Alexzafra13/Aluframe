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
**Servidor Linux:** descomprime el `.tar.gz`, entra en la carpeta y ejecuta `sudo ./instalar.sh`. Instala en `/opt/alumframe`, configura en `/etc/alumframe/alumframe.toml`, guarda los datos en `/var/lib/alumframe` y deja el servicio `alumframe` activo al arrancar. Comprueba `http://IP:9090/health`.
- Actualizar: lo mismo con la versión nueva; la anterior queda en `/opt/alumframe/alumframe.anterior`.
- Volver atrás: ver `despliegue/linux/LEEME.txt`. (Hasta el primer despliegue real no hay migraciones: ADR-0011.)
- Registros: `journalctl -u alumframe`.

**PC de la oficina (Windows):** ejecuta `crear-acceso-directo.ps1` del `.zip` e indica la IP del servidor. `alumframe.exe` es el modo local (probar o emergencia).

## 5. Copias de seguridad (H7)
Hoy: antes de cada migración se guarda una copia con `VACUUM INTO` en `copias/`. La copia nocturna, la retención y la restauración guiada llegan en H7. **Nunca copies `alumframe.db` con el programa en marcha.**

## 6. Operaciones frecuentes
- Dar de alta operarios, cambiar su PIN o desactivarlos: Ajustes > Usuarios.
- Grosor de sierra, recorte de extremos y retal mínimo: Ajustes > Taller.
- Contraseña del propietario olvidada (en el servidor): `sudo -u alumframe ALUMFRAME_CONFIG=/etc/alumframe/alumframe.toml /opt/alumframe/alumframe restablecer-contrasena "Nombre"`.
- **Añadir o corregir una serie:** edita su YAML (formato en `docs/FORMATO_SERIES.md`) e impórtalo en Catálogo > Series > Importar serie. La vista previa muestra errores y avisos; cada importación crea una versión nueva y las anteriores se conservan.
- Antes de tocar una serie real, comprueba sus ventanas con «Probar una ventana» y que `just test` sigue en verde: los tests dorados comparan `data/series/` con `fixtures/real/`.
- **Actualizar una tarifa:** Catálogo > Tarifas > Importar tarifa. Elige el proveedor y el CSV con las columnas `referencia;descripcion;unidad;precio_tarifa;longitud_barra` (y `tipo` opcional). Desde Excel, «Guardar como» → CSV. Revisa la vista previa (los saltos de más del 30 % salen marcados) y confirma. Si algo no cuadra, «Deshacer» en la versión vigente.
- **Cambiar unos pocos precios:** Catálogo > Artículos, filtra por proveedor y escribe el precio en la tabla, o marca artículos y aplica un porcentaje. Cada cambio queda como una versión de la tarifa.
- **Importar clientes** de una hoja de cálculo: Clientes > Importar clientes (columnas en ADR-0012, todas opcionales).
- **Petición RGPD de un cliente:** en su ficha, «Exportar datos» (acceso) o «Borrar definitivamente» (supresión).
- **Margen, cobro de perfiles, mano de obra, IVA y datos de la empresa:** Ajustes > Empresa y precios. Lo que se deje sin elegir hace que las ventanas salgan «incompletas» y lo dicen.
- **Presupuestos:** si cambia una tarifa, el presupuesto en borrador avisa y ofrece «Recalcular precios». Un presupuesto aceptado no se toca: se guarda congelado con su huella.

## 7. Actualizar el toolchain y las dependencias
Cada 6 meses o ante un aviso de seguridad:
1. Cambia `rust-toolchain.toml` si procede.
2. `cargo update` y después `just antiguedad`: lista lo publicado hace menos de 14 días. Bájalo con `cargo update -p nombre@version --precise version-anterior`.
3. `cargo deny check`, `just check` y `just e2e`; prueba con una copia de los datos reales.

## 8. Si la base de datos falla
`sqlite3 alumframe.db "PRAGMA integrity_check"` con el servicio parado. Si no responde `ok`, restaura la última copia (H7).

## 9. Dónde están las decisiones
`docs/adr/`, `docs/CONTEXTO.md`, `docs/PROGRESS.md` y `docs/CAMBIOS.md`.

## 10. Cómo retomar el proyecto con Claude
Lee `CLAUDE.md` y `docs/PROGRESS.md`, ejecuta `just check` y sigue el siguiente hito pendiente.
