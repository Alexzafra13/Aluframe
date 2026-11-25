# Vocabulario de nombres

El código usa el vocabulario del taller, en español y sin tildes ni eñes.

| Concepto | Identificador |
|---|---|
| presupuesto | `presupuesto` |
| trabajo | `trabajo` |
| orden de taller / hoja de corte | `orden_de_taller` / `hoja_de_corte` |
| serie de perfiles | `serie` |
| tipología | `tipologia` |
| pieza | `pieza` |
| barra | `barra` |
| retal | `retal` |
| grosor de sierra | `grosor_de_sierra` |
| hueco (de obra) / luz | `hueco` / `luz` |
| albarán | `albaran` |
| factura / rectificativa | `factura` / `rectificativa` |
| tarifa | `tarifa` |
| año | `anio` |
| marco, hoja, cruce, rodamiento | `marco`, `hoja`, `cruce`, `rodamiento` |
| inversor, escuadra, cremona, pletina de falleba | `inversor`, `escuadra`, `cremona`, `pletina_de_falleba` |
| herraje (lo que no se corta: cierres, kits, ruedas, tornillos) | `herraje` (en las series, `accesorios`) |
| parteluces (travesaño horizontal en la hoja) | `parteluces` |
| chapa (panel opaco de abajo de un parteluces) | `chapa` |
| opción de medida (se escribe en la línea, entra en las fórmulas) | `tipo: medida` |
| condición de una parte de la serie | `solo_si` |
| mosquitera enrollable: cajón, guía lateral, terminal, kit | `mosquitera`, `cajon`, `guia`, `terminal`, `kit` |
| pieza pendiente (sin medida de corte todavía) | `pieza_pendiente` |
| persiana en la ventana / suelta en pared (guías de L) | `persiana`, `montaje: ventana` / `montaje: pared` |
| cajón, guía, cinta, recogedor, lama (de la persiana) | `cajon`, `guia`, `cinta`, `recogedor`, `lama` |
| medida del conjunto / del marco / del hueco | `ancho` y `alto` escritos; `ancho_ventana` (marco con persiana); hueco en pared |
| material a pedir (lo que se encarga a medida: cristal, persiana) | `material_a_pedir`, `MaterialAPedir`; en la serie, `se_pide: true` |
| estado del material: pendiente de pedir, pedido, recibido | `EstadoDeMaterial::{Pendiente, Pedido, Recibido}` |
| dibujo del presupuesto / dibujo del taller | `dibujo` / `dibujo_taller` (`para_taller`) |

## Ejemplos de funciones
`evaluar_formula`, `expandir_ventana`, `optimizar_cortes`, `calcular_cuota_iva`, `aceptar_presupuesto`, `emitir_factura`, `registrar_salida_de_stock`, `asignar_numero_de_documento`, `crear_copia_de_seguridad`, `restaurar_copia`. Preguntas de sí o no: `es_incompleta`, `tiene_stock_suficiente`, `puede_emitirse`.

Los códigos van sin tildes; al mostrarlos, `nombre_legible` repone la tilde de las palabras del taller que la llevan (`CAJON` → «Cajón», `GUIA` → «Guía», `si` → «Sí») y convierte el guion bajo entre cifras en coma (`TORNILLO_4_8x25` → «Tornillo 4,8x25»). Una palabra nueva con tilde se añade en `crates/dominio/src/texto.rs`.
