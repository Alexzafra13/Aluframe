# Formato de los archivos de series (versión 1, borrador)

Una **serie** es la receta de un fabricante: cómo se calcula cada ventana a partir de su ancho y su alto. Se escribe como un archivo de datos (YAML) que se importa desde una pantalla, sin tocar código. Ejemplos reales: `data/series/aluprom-21.yaml` y `data/series/sopena-v94.yaml`. Ejemplo ficticio para pruebas: `fixtures/demo/serie-demo.yaml`.

Este documento fija qué puede contener un archivo y qué comprueba el programa al importarlo. Si algo de aquí choca con `SPEC.md` o `docs/BACKEND.md`, se corrige en el mismo cambio.

## 1. En pocas palabras
Un archivo de serie responde a siete preguntas:
1. **Quién es:** serie, fabricante, proveedor, estado.
2. **Cómo se mide:** unidades y regla de redondeo.
3. **Qué números fija el fabricante:** los descuentos (los *parámetros*).
4. **Qué pieza es qué referencia del proveedor:** la tabla `referencias`, que enlaza con la tarifa.
5. **Qué lleva cada ventana:** piezas, vidrio y accesorios, con sus fórmulas.
6. **Qué se elige al presupuestar:** las *opciones de línea* (por ejemplo, el lado de la cremona).
7. **Cómo se dibuja:** el tipo de dibujo y de dónde saca los elementos.

Regla de oro: **lo que el usuario no ha confirmado se escribe `CONFIRMAR` y nunca se inventa.** Una línea con algún `CONFIRMAR` se calcula igualmente, pero sale marcada como incompleta y no admite precio final ni aceptar el presupuesto.

## 2. Estructura completa
```yaml
serie: SOPENA_V94                  # código único, MAYÚSCULAS y guion bajo
fabricante: SOPENA
proveedor: CONFIRMAR               # quién la vende al taller; enlaza con la tarifa
estado: borrador                   # borrador | validada | demo
unidades: cm                       # cm | mm: la unidad de las fórmulas y de la pantalla
redondeo: { precision: 0.1, sentido: abajo }   # abajo | arriba | cercano
longitud_barra: 640                # en la unidad de la serie; CONFIRMAR si falta

parametros:                        # números del fabricante, con nombre
  descuento_hoja_alto: 4.1

referencias:                       # código de la serie -> referencia del proveedor
  MARCO_LATERAL: { referencia: CONFIRMAR }
  BISAGRA:       { referencia: CONFIRMAR }

tipologias:
  europea_abatible_2_hojas:
    nombre: "Ventana europea abatible 2 hojas"
    estado: borrador               # opcional; por defecto el de la serie
    limites:                       # opcional
      ancho: { min: CONFIRMAR, max: CONFIRMAR }
      alto:  { min: CONFIRMAR, max: CONFIRMAR }
    opciones:                      # lo que se elige al presupuestar
      - { codigo: lado_cremona, nombre: "Hoja de la cremona (vista desde dentro)", valores: [izquierda, derecha], por_defecto: derecha }
    dibujo: { tipo: abatible, hojas: 2, bisagras: BISAGRA, cremona: lado_cremona }
    piezas:
      - { perfil: MARCO_LATERAL, cantidad: 2, longitud: "alto", corte: 45 }
    vidrio:
      tipo: CRISTAL_4MM
      piezas:
        - { cantidad: 2, ancho: "(ancho - descuento_cristal_ancho) / 2", alto: "alto - descuento_cristal_alto" }
    accesorios:
      - { articulo: BISAGRA, cantidad: "2 * si(alto > 140, 3, 2)", aplica_a: hojas }
```

## 3. Campos
### 3.1 Cabecera
| Campo | Obligatorio | Contenido |
|---|---|---|
| `serie` | sí | Código único: `[A-Z0-9_]`, hasta 40 caracteres. |
| `fabricante` | sí | Texto libre en mayúsculas (Aluval, Sopena). Si existe `data/marcas/<fabricante en minúsculas>.png`, su logotipo sale junto al nombre de la serie (ADR-0029). |
| `proveedor` | sí | Código del proveedor del taller o `CONFIRMAR`. Enlaza con la tarifa importada. |
| `estado` | sí | `borrador`, `validada` o `demo`. Solo `validada` permite precio final; `demo` solo se carga en pruebas. |
| `unidades` | sí | `cm` o `mm`. Se admiten decimales (4,85 se escribe `4.85`). |
| `redondeo` | sí | `precision` (en la unidad de la serie) y `sentido`. Se aplica **una vez**, al resultado final de cada fórmula de longitud, no en los pasos intermedios. |
| `longitud_barra` | sí | Barra nueva estándar, o `CONFIRMAR`. Las barras más cortas se registran en el stock. |
| `parametros` | no | Pares `nombre: número`. Los nombres van en minúsculas con guion bajo. |
| `referencias` | sí | Ver el apartado 5. |
| `tipologias` | sí | Al menos una. |

### 3.2 Tipología
| Campo | Obligatorio | Contenido |
|---|---|---|
| `nombre` | sí | Texto que ve el usuario. |
| `estado` | no | Igual que la serie. Una tipología en `borrador` dentro de una serie `validada` sigue sin dar precio final. |
| `piezas` | sí | Perfiles cortados (apartado 4). |
| `vidrio` | sí | Apartado 6, o `{ estado: CONFIRMAR }`. |
| `accesorios` | sí (puede ir vacío) | Apartado 7. |
| `opciones` | no | Apartado 8. |
| `dibujo` | sí | Apartado 9. |
| `limites` | no | Medidas mínima y máxima de `ancho` y `alto`. Si falta, el programa avisa de que los límites no están definidos y no bloquea. |

## 4. Fórmulas y piezas
Cada pieza es `{ perfil, cantidad, longitud, corte }` y, si hace falta, `posicion` y `perfil_segun`.
- `perfil`: código que debe existir en `referencias`.
- `cantidad`: un entero o una fórmula que da un entero.
- `longitud`: fórmula en la unidad de la serie.
- `corte`: grados (`90`, `45`) o `CONFIRMAR`. Un corte distinto de 90 se dibuja y se imprime en la hoja de corte.
- `perfil_segun` (opcional, ADR-0018): otro perfil cuando una opción de la línea toma un valor. `perfil_segun: { acristalamiento: { climalit: HOJA_LATERAL_CLIMALIT } }` usa ese perfil si en la línea se elige «climalit» y `perfil` en cualquier otro caso. La opción y el valor deben existir en la tipología y el perfil, en `referencias`. Cambia el perfil (y con él la referencia, el precio y la silueta), no la cantidad ni la medida, y la pieza conserva su número.
- `posicion` (opcional, ADR-0017): dónde va el número de la pieza en el dibujo cuando su nombre no lo dice: `marco_lateral`, `marco_superior`, `marco_inferior`, `hoja_lateral`, `hoja_central`, `hoja_superior`, `hoja_inferior` o `centro` (donde se juntan las hojas centrales de una corredera de 4, para el cierre 4 hojas). Sin ella se deduce del nombre (`MARCO_SUPERIOR`, `HOJA_RODAMIENTO`…). Ejemplo: la `HOJA_CIERRE` de Aluval va en el centro de la corredera, `posicion: hoja_central`.

Las fórmulas usan solo lo que permite el evaluador (`docs/BACKEND.md`, apartado 5.1):
- variables `ancho` y `alto` y los `parametros` de la serie;
- números con punto decimal, `+ - * /` y paréntesis;
- `minimo`, `maximo`, `redondear`, `techo`, `suelo`;
- `si(condición, entonces, si_no)`, y solo dentro de `si` las comparaciones `>`, `>=`, `<`, `<=`, `==`.
- `minimo` y `maximo` admiten 2 o más argumentos; `redondear`, `techo` y `suelo` tienen 1 y redondean a un número entero de la unidad de la serie (al más cercano, hacia arriba y hacia abajo).
- Un campo con un nombre que no está en este documento es un error (evita erratas como `acesorios`).

No se admiten texto, variables nuevas ni funciones propias. Una fórmula con un nombre desconocido es un **error de importación**, no se interpreta. Las opciones de lista (apartado 8) no se usan dentro de las fórmulas: deciden qué partes existen (`solo_si`) y qué perfil lleva una pieza (`perfil_segun`). Las **opciones de medida** sí: su código es una variable más de las fórmulas de su tipología (ADR-0030).

Precisión: se calcula con enteros en décimas de milímetro. `ancho / 2 - 2` con `ancho = 150` da 73; con `ancho = 151` da 73,5; con `(ancho - 4.85) / 2` se aplica el redondeo de la serie al final.

- **Cantidad 0 = la pieza no va** con esa medida (ADR-0031). Con `si()` se elige entre dos perfiles según la medida: `cantidad: "si(alto <= 140, 1, 0)"` y `cantidad: "si(alto > 140, 1, 0)"`. Las piezas se numeran entre las que van.

## 5. Referencias: el enlace con la tarifa
La tabla `referencias` traduce cada **código de la serie** (`MARCO_LATERAL`, `BISAGRA`) a la **referencia real del proveedor** (la que aparece en su tarifa).

```yaml
referencias:
  MARCO_LATERAL: { referencia: CONFIRMAR }
  RUEDA_CORREDERA: { referencia: "R-1234" }     # ejemplo de forma, no es un dato real
```
- El programa busca esa referencia en la tarifa vigente del `proveedor` de la serie y obtiene el artículo, su unidad (barra, metro, unidad, caja) y su precio.
- Todo código usado en `piezas`, `accesorios` o `vidrio.tipo` debe estar en `referencias`; si falta, es un error de importación.
- `referencia: CONFIRMAR` o una referencia que no aparece en la tarifa **no es un error**: la línea sale incompleta con el aviso "falta el precio de MARCO_LATERAL".
- Una misma serie puede usar referencias de más de un proveedor: se añade `proveedor` dentro de la entrada (`{ referencia: "...", proveedor: OTRO }`).
- El **color** no cambia la referencia: el acabado aplica su recargo sobre el precio del blanco (`docs/DATOS.md`, apartado 3.3).
- `silueta` (opcional, ADR-0017): el dibujo de la sección del perfil, `fabricante/archivo` en minúsculas (`aluval/21210` es `data/perfiles/aluval/21210.svg`). Sale junto al número de la pieza para reconocerla. **No interviene en el precio**: se puede poner aunque la `referencia` siga en `CONFIRMAR`. También sirve para los herrajes, con los dibujos genéricos de `data/perfiles/herrajes/` (`herrajes/cierre-embutido`, ADR-0029); el modelo concreto lo dice el artículo del catálogo.

## 6. Vidrio
```yaml
vidrio:
  tipo: CRISTAL_4MM                  # código de referencias; el usuario puede cambiarlo en la línea
  piezas:
    - { cantidad: 2, ancho: "ancho / 2 - descuento_cristal_ancho", alto: "alto - descuento_cristal_alto" }
```
- Cada entrada de `piezas` es un tipo de cristal distinto; `cantidad`, `ancho` y `alto` son fórmulas.
- El **tipo de vidrio** (simple, doble, espesor) se elige en la línea del presupuesto; el de la serie es el valor por defecto.
- Sin el apartado `vidrio`, la tipología no lleva cristal (una mosquitera, ADR-0031).
- Mientras falte el descuento, se escribe `vidrio: { estado: CONFIRMAR }`. Si solo falta una medida de una pieza (el alto del cristal de arriba de un parteluces), esa medida puede ir en `CONFIRMAR`: la pieza no sale y la ventana queda incompleta.
- Opcionales en cada pieza (ADR-0030): `nombre` («Arriba», «Abajo») y `solo_si` (apartado 8).
- El vidrio se pide a medida y no se optimiza. El precio es `m² × precio/m²`, con la superficie mínima facturable del proveedor si la hay (por confirmar).

## 7. Accesorios y kits
Cada accesorio es `{ articulo, cantidad, aplica_a }`.
- `articulo`: código de `referencias`.
- `cantidad`: entero o fórmula; `CONFIRMAR` si falta el dato.
- `aplica_a`: `marco`, `hojas` o `ventana` (por defecto `ventana`). Sirve para desglosar la lista y para saber cuántos tornillos van en cada parte.
- Un accesorio vendido por metro (como una pletina de falleba) usa `unidad_cantidad: metro` y una `cantidad` que es la longitud en la unidad de la serie. Su fórmula está pendiente (`CONFIRMAR`).
- Un **kit** es un artículo de la tarifa o del catálogo formado por otros (`docs/DATOS.md`, tabla `kit_componente`); en la serie se pone el kit como un único accesorio y el stock descuenta sus componentes.
- Si dos accesorios comparten código (como `ESCUADRA` para el marco y para las hojas), se suman en la lista de compra y se separan en la hoja de corte.

## 8. Opciones de línea
Son decisiones que toma quien presupuesta o asigna el trabajo, no el operario.
```yaml
opciones:
  - { codigo: lado_cremona, nombre: "Hoja de la cremona (vista desde dentro)", valores: [izquierda, derecha], por_defecto: derecha }
```
- Aparecen como un selector en la línea del presupuesto, se congelan al aceptar y se ven en el dibujo, la hoja de corte y la orden del operario.
- `valores` son textos cerrados. `por_defecto` es obligatorio.
- Una opción de lista cambia el perfil de una pieza (`perfil_segun`, ADR-0018) y decide qué partes existen (`solo_si`, abajo).
- **Opción de medida** (ADR-0030): `tipo: medida`, con `por_defecto`, `min` y `max` en la unidad de la serie. Se escribe en la línea (en cm) y se usa en las fórmulas de su tipología por su `codigo`, como `ancho` y `alto`. Fuera de `min`–`max` el cálculo da error.
  ```yaml
  - { codigo: medida_abajo, nombre: "Medida de abajo", tipo: medida, por_defecto: 50, min: 10, max: 200, solo_si: { parteluces: si } }
  ```
- **`solo_si: { opcion: valor, … }`** (ADR-0030) en una opción, una pieza, un accesorio, una pieza de vidrio, una chapa o el `parteluces`: solo existe (y solo se pregunta) si se cumplen todas las condiciones. Solo vale con opciones de lista. Al importar se calcula también con **cada combinación** de opciones de lista (hasta 256; por encima, cada valor por separado), para que una fórmula que solo se usa con una combinación (parteluces y Climalit) no falle al presupuestar.

### Chapas y parteluces (ADR-0030)
```yaml
chapas:     # paneles opacos; ancho y alto admiten CONFIRMAR (se ve «?» y la ventana queda incompleta)
  - { articulo: CHAPA_LACADA_2_CARAS, nombre: "Chapa de abajo", cantidad: 2, ancho: CONFIRMAR, alto: "medida_abajo", solo_si: { parteluces: si, relleno_abajo: chapa } }
parteluces: { solo_si: { parteluces: si }, abajo: "medida_abajo", altura: CONFIRMAR }
```
- `chapas`: el `articulo` es un código de `referencias`; se dibuja a escala con sus cotas en el editor y en la hoja de taller. Mientras no se decida cómo se cobra, una ventana con chapa no tiene precio final.
- `parteluces` solo da el dibujo: una barra en cada hoja. `abajo` es la medida escrita de lo de abajo; `altura`, la del eje desde abajo de la ventana. Sin `altura`, la barra se coloca aproximada y la cota dice «chapa 50». Su perfil, cristales y chapas se declaran en `piezas`, `vidrio` y `chapas` con el mismo `solo_si`.

## 9. Dibujo
El dibujo se **calcula a partir de las mismas piezas y medidas** que la hoja de corte, así que no puede contradecirlas. Cada tipología declara el tipo de dibujo y los datos que necesita:

| `tipo` | Campos | Cómo se dibuja |
|---|---|---|
| `corredera` | `hojas` (2 o 4), `hoja_movil` (opcional: `izquierda`, `derecha` o `ambas`; por defecto `derecha`) | Marco, hojas con cristal, flecha hacia el centro en cada hoja que se abre y su cierre en el lateral, contra el marco (como el catálogo de Aluval) |
| `abatible` | `hojas`, `bisagras` (código de accesorio), `cremona` (código de opción) | Marco, hojas con cristal, líneas de apertura que apuntan al lado de las bisagras, bisagras repartidas y manilla de la cremona en la hoja elegida |
| `fija` | `hojas` | Marco y cristal (llegará con la primera serie real) |

- Bisagras por hoja = `cantidad` del accesorio indicado ÷ número de hojas. Su posición exacta (distancia a los extremos) está pendiente de confirmar con el taller.
- Tipos nuevos de dibujo son código y exigen una decisión; una tipología nueva de un tipo ya dibujado es solo datos.
- El dibujo es un esquema para entender la ventana, no un plano de fabricación.
- Los números de las piezas del dibujo son los de la hoja de corte (diseño aprobado en `design/03-hoja-taller.png`).

- `tipo: mosquitera` (ADR-0031): cajón arriba, guías, malla y terminal con tirador. El cajón se dibuja más alto si sale una pieza de cajón de 50 (`CAJON_50`). Los números van donde dice `posicion` (`marco_superior`, `marco_lateral`, `marco_inferior`).

## 10. Límites
```yaml
limites:
  ancho: { min: 40, max: 300 }       # ejemplo de forma, no son datos reales
  alto:  { min: 40, max: 250 }
```
Al presupuestar, una medida fuera de límites muestra un aviso visible ("este perfil no admite más de X cm") y deja guardar la línea marcada como **revisar**. No bloquea, porque el taller puede decidir fabricarla con un refuerzo.

## 11. Qué comprueba el programa al importar
La importación muestra una **vista previa** con los errores en español y no guarda nada hasta confirmar. Errores que impiden importar:
- campo obligatorio ausente o con tipo equivocado;
- `serie` repetida con el mismo contenido (se avisa de que no hay cambios) o código con caracteres no permitidos;
- código en `piezas`, `accesorios` o `vidrio.tipo` que no está en `referencias`;
- fórmula con sintaxis incorrecta, nombre desconocido o más de 200 caracteres;
- `cantidad`, `longitud` o medidas de cristal que salen cero o negativas con las medidas de prueba `ancho = 100` y `alto = 100`;
- opción de línea sin `por_defecto` o con un valor por defecto que no está en `valores`;
- `dibujo` que apunta a una opción o a un accesorio que no existen.

Avisos que no impiden importar (la serie queda como borrador):
- cada `CONFIRMAR` (se lista cuántos hay y dónde);
- referencias sin precio en la tarifa vigente;
- tipología sin `limites`.

Si el archivo es válido, se compara con la versión anterior y se muestra qué cambia (descuentos, piezas, accesorios).

## 12. Versiones
- Importar una serie nueva con el mismo código crea la **versión siguiente** (`serie_version`); nunca se edita una versión que ya se usó en un presupuesto.
- Cada versión guarda el texto original del YAML y su huella SHA-256, de modo que se puede exportar exactamente igual.
- Pasar de `borrador` a `validada` es una acción explícita del propietario, con las ventanas reales de comprobación enlazadas (`fixtures/real/`).
- Los presupuestos aceptados guardan la versión de serie con la que se calcularon.

## 13. Pendiente de decidir
- Si `proveedor` puede ser distinto del fabricante en la misma serie (Aluval fabrica, otro distribuye): se admite el campo, falta ver un caso real.
- Fórmula y unidad de la pletina de falleba.
- Tipos de escuadra de la V-94 (exterior e interior de hoja): ¿códigos distintos?
- Posición de las bisagras en el dibujo.
- Si una opción de línea necesitará cambiar cantidades o medidas (lo más probable, el lado del inversor).
