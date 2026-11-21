# Guía de mantenimiento

Objetivo: que cualquier persona, o una sesión nueva de Claude sin contexto, pueda mantener AlumFrame. Se completa en H7; lo que ya existe está al día.

## 1. Requisitos y compilación
- Rust fijado en `rust-toolchain.toml` (rustup lo instala solo), `just` y `cargo-deny` (`cargo install just cargo-deny --locked`).
- Node 22 solo para los E2E (`e2e/`); el ejecutable no lo necesita.
- `just check` (formato, clippy, pruebas y tamaño de archivos) · `just build` · `just dev` (modo local en `datos-dev/`).
- Activa el hook previo al push una vez por clon: `just instalar-hooks`.

## 2. Pruebas
- `just test`: unitarias e integración (SQLite en memoria).
- `just e2e`: compila en release y ejecuta Playwright contra el ejecutable, con una carpeta de datos temporal y el puerto 9191.
- `just cobertura`: cobertura del dominio (necesita `cargo install cargo-llvm-cov` y `rustup component add llvm-tools-preview`).

## 3. Publicar una versión
1. Sube `version` en `Cargo.toml` (`[workspace.package]`) y anota el cambio en `docs/CAMBIOS.md`.
2. `just check` y `just e2e` en verde.
3. `git tag v0.2.0 && git push origin v0.2.0`. GitHub Actions (`publicar.yml`) deja en la página de versiones `alumframe-v0.2.0-linux-x86_64.tar.gz` y `alumframe-v0.2.0-windows-x86_64.zip`.

## 4. Instalar y actualizar en el taller
**Servidor Linux:** descomprime el `.tar.gz`, entra en la carpeta y ejecuta `sudo ./instalar.sh`. Instala en `/opt/alumframe`, configura en `/etc/alumframe/alumframe.toml`, guarda los datos en `/var/lib/alumframe` y deja el servicio `alumframe` activo al arrancar. Comprueba `http://IP:9090/health`.
- Actualizar: lo mismo con la versión nueva; la anterior queda en `/opt/alumframe/alumframe.anterior`.
- Volver atrás: ver `despliegue/linux/LEEME.txt`. Si la base ya se migró a un esquema nuevo, el programa antiguo no arranca; restaura la copia `antes-de-migrar-*.db` de `/var/lib/alumframe/copias/`.
- Registros: `journalctl -u alumframe`.

**PC de la oficina (Windows):** ejecuta `crear-acceso-directo.ps1` del `.zip` e indica la IP del servidor. `alumframe.exe` es el modo local (probar o emergencia).

## 5. Copias de seguridad (H7)
Hoy: antes de cada migración se guarda una copia con `VACUUM INTO` en `copias/`. La copia nocturna, la retención y la restauración guiada llegan en H7. **Nunca copies `alumframe.db` con el programa en marcha.**

## 6. Operaciones frecuentes
- Dar de alta operarios, cambiar su PIN o desactivarlos: Ajustes > Usuarios.
- Contraseña del propietario olvidada (en el servidor): `sudo -u alumframe ALUMFRAME_CONFIG=/etc/alumframe/alumframe.toml /opt/alumframe/alumframe restablecer-contrasena "Nombre"`.
- Tarifas, series e IVA: llegan en H1 y H2.

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
