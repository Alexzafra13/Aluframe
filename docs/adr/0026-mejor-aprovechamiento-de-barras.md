# ADR-0026: mejor aprovechamiento de las barras, medidas habituales e historial

**Estado:** aceptado (2026-11-23; plan aprobado por el propietario: «como tú mejor lo veas»).

## Contexto
El reparto de piezas en barras era Best Fit Decreasing (BFD, `docs/BACKEND.md` 5.3): rápido y casi siempre bueno, pero a veces abre una barra de más. Ejemplo real de la prueba: 289, 265, 212, 174, 132, 100 y 100 cm abren 3 barras de 640 y caben en 2. El propietario pidió además que el programa aprenda de lo que más se corta y deje retales de esas medidas, empezando por unas medidas habituales que él da (120 × 120 en Aluprom 21) y afinando con el historial.

## Decisión
1. **Dos métodos**, a elegir en Ajustes > Taller (SPEC 6 pide poder cambiar de algoritmo): *Rápido* (BFD tal cual) y *Mejor aprovechamiento* (por defecto).
2. **Mejor aprovechamiento** parte del reparto de BFD y busca otro con, por orden: (a) menos barras nuevas; (b) con las mismas, sobrantes que valgan más. Solo acepta uno estrictamente mejor: **nunca sale peor que BFD** (prueba de propiedades).
3. **La búsqueda se corta por número de pasos** (20 000 por perfil y color), no por tiempo: el resultado es el mismo en cualquier equipo y con cualquier carga (SPEC 6: determinista). Poda por cota inferior de barras, sin probar dos barras con el mismo sobrante y sin repetir repartos de piezas iguales. 80 piezas de un perfil: 9 ms en el ejecutable publicado.
4. **Valor de los sobrantes** (`ValoracionDeSobrantes`), de mayor a menor importancia:
   - *Uso esperado:* para cada sobrante útil, cuántas piezas habituales saldrían de él de media (ponderado por lo que se corta). Un sobrante casi igual (a menos de 10 cm) a otro ya contado vale la mitad: no llenar el almacén de trozos que sirven para lo mismo. Cuentan también los retales del stock que el plan no usa.
   - *A igualdad, lo que sobra más junto* (suma de cuadrados): un retal largo mejor que dos cortos.
   - Por debajo del **mínimo útil** un sobrante es desecho y no suma uso. Hoy es el «retal mínimo» de Ajustes > Taller (0 por decisión del propietario). Cuando cada serie tenga sus medidas mínimas (`limites`, hoy `CONFIRMAR`), el mínimo de cada perfil será su pieza más corta posible, y lo que quede por debajo saldrá en «Terminar orden» desmarcado como desecho (fase 2, pendiente de esos datos).
5. **Medidas habituales**, en Ajustes > Taller: ventanas típicas por serie y tipología (ancho × alto), no piezas sueltas; el programa calcula sus piezas con las fórmulas de la serie. La de partida es la que dio el propietario: Aluprom 21, corredera de 2 hojas, 120 × 120. Cada una pesa como unas pocas ventanas reales.
6. **Historial:** las ventanas de los presupuestos aceptados (su copia congelada, con las piezas ya calculadas). Lo reciente pesa más: el peso se reduce a la mitad cada año. Con muchas ventanas reales, las habituales dejan de influir solas.
7. **La hoja impresa no cambia por lo que pase después:** cada trabajo valora sus sobrantes con el historial hasta el día en que se aceptó. Así lo que corta el operario es lo que se descuenta al terminar la orden.
8. El precio de una ventana sola usa el método elegido sin historial: solo cuentan las barras.

## Consecuencias
- Sin dependencias nuevas; todo en `crates/dominio/src/corte/` (lógica pura) y su uso en el servidor.
- El ahorro grande ya existía (usar los retales del stock antes de abrir barras). Esto ahorra la barra que BFD a veces gasta de más y afina qué sobrante queda; la estadística solo decide entre repartos con las mismas barras nuevas. Nunca se corta de más para fabricar un retal.
- Con las mismas barras, puede preferir no partir un retal del stock si así lo que queda es más aprovechable; nunca a costa de abrir una barra nueva.
- Un presupuesto en «barras enteras» puede salir algo más barato que con BFD si ahorra una barra; los aceptados no cambian (copia congelada).

## Alternativas descartadas
- Programación lineal o un resolvedor externo: una dependencia grande para problemas de decenas de piezas.
- Límite por tiempo: el resultado dependería de la máquina.
- Cortar de más para crear retales de medidas habituales: gasta material hoy por un uso incierto.
