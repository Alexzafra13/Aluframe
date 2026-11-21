# ADR-0014: las series del taller vienen incluidas en el programa

**Estado:** aceptado (2026-11-21, decisión del propietario: «que vengan de serie las propias series»).

## Contexto
Hasta la v0.4.0 una instalación nueva arrancaba sin ninguna serie y había que importar a mano cada YAML de `data/series/` (Catálogo > Series > Importar serie). El propietario quiere que el programa funcione nada más instalarlo («plug and play») y que las series que se vayan añadiendo al repositorio lleguen con cada versión.

## Decisión
- Los YAML de `data/series/` se **incluyen en el ejecutable** al compilar, con `rust-embed`, que ya se usaba para los recursos estáticos. No hay dependencias nuevas.
- **Al arrancar**, cada serie incluida pasa por la misma validación que la importación a mano. Si su archivo exacto (SHA-256) no ha estado nunca en esa serie, se guarda como **versión nueva**, que pasa a ser la vigente, con el apunte de auditoría `cargar_serie_incluida` (sin usuario). La columna `importada_por` se quita de `serie_version`: quién importó a mano consta en la auditoría (`importar_serie`). Si ya estuvo, no se toca.
- Si el propietario importa otra versión a mano, se respeta mientras el archivo incluido no cambie. Cuando una versión nueva del programa trae ese archivo cambiado, entra como versión nueva y la del propietario queda en el historial (opción A, elegida por el propietario).
- Una serie incluida con errores **no impide arrancar**: se salta y se anota en el registro. La prueba `incluye_todas_las_series_de_data_series` y los tests dorados evitan publicarla así.
- Las series de **demostración** (`fixtures/demo/`) nunca se incluyen ni se cargan solas.

## Consecuencias
- Instalar y abrir basta para presupuestar con las series del taller.
- Corregir o añadir una serie es: editar `data/series/`, pasar las pruebas y publicar una versión. Los presupuestos aceptados no cambian (guardan su copia congelada) y los borradores siguen con la versión con la que se calcularon hasta recalcular.
- La importación a mano se conserva: sirve para probar un cambio sin esperar a una versión. Cambiar una serie sigue sin exigir recompilar (`CLAUDE.md`).
- El ejecutable crece lo que ocupan los YAML (unos pocos KB).

## Alternativas descartadas
- **Leer los YAML de una carpeta junto al ejecutable:** obliga a copiar y mantener más archivos en la instalación y se pierde el «un único ejecutable».
- **Importación a mano (como hasta ahora):** es justo lo que el propietario no quiere.
- **Opción B, no tocar una serie importada a mano y avisar de que hay versión nueva:** más pantallas y más casos para un uso que no se espera, porque las series se mantienen en el repositorio.
