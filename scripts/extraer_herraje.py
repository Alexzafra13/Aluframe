#!/usr/bin/env python3
"""Extrae en vector el dibujo técnico de un herraje de una ficha PDF del fabricante.

Herramienta de desarrollo, no forma parte del programa (ADR-0029). Necesita `pdftocairo`
(poppler). Convierte la página a SVG y se queda con los trazos que caen enteros dentro del
recuadro indicado (en puntos PDF, origen arriba a la izquierda), así que las líneas de cota que
salen del recuadro se descartan. Los colores pasan a los del diseño: tinta y texto apagado.

Uso:
  scripts/extraer_herraje.py ficha.pdf 1 83.2,189.4,114.8,392.0 data/perfiles/herrajes/cierre-embutido.svg --tumbado

`--tumbado` gira el dibujo un cuarto de vuelta: un herraje largo y estrecho (un cierre) aprovecha
mejor así el hueco apaisado del dibujo en la lista.
`--grosor-minimo=N` engorda las líneas más finas que N (en unidades del trazo del PDF): el dibujo
se ve a 56 × 42 px y las líneas de CAD, tan finas, desaparecerían al reducirlo.

Para encontrar el recuadro: `pdftocairo -png -r 72 -f N -l N ficha.pdf pagina` y medir en
píxeles (a 72 ppp, un píxel es un punto).
"""

import re
import subprocess
import sys
import tempfile
from pathlib import Path

TINTA = "#15191D"
APAGADO = "#5C636B"
NUMERO = re.compile(r"-?\d+(?:\.\d+)?(?:e-?\d+)?")
TRAZO = re.compile(r"<path\b[^>]*>")
MATRIZ = re.compile(r'transform="matrix\(([^)]*)\)"')
ATRIBUTO_D = re.compile(r'\sd="([^"]*)"')
COLOR = re.compile(r'(stroke|fill)="rgb\(([\d.]+)%, ([\d.]+)%, ([\d.]+)%\)"')
HOLGURA = 0.6  # puntos: el grosor del trazo no debe sacar una pieza del recuadro


def puntos(d, matriz):
    numeros = [float(n) for n in NUMERO.findall(d)]
    a, b, c, dd, e, f = matriz
    for x, y in zip(numeros[0::2], numeros[1::2]):
        yield (a * x + c * y + e, b * x + dd * y + f)


def dentro(trazo, recuadro):
    d = ATRIBUTO_D.search(trazo)
    if not d:
        return False
    m = MATRIZ.search(trazo)
    matriz = [float(v) for v in m.group(1).split(",")] if m else [1, 0, 0, 1, 0, 0]
    x0, y0, x1, y1 = recuadro
    todos = list(puntos(d.group(1), matriz))
    return bool(todos) and all(
        x0 - HOLGURA <= x <= x1 + HOLGURA and y0 - HOLGURA <= y <= y1 + HOLGURA for x, y in todos
    )


def recolorear(trazo):
    """Negro y grises oscuros a tinta; grises medios a apagado; el blanco se queda."""

    def cambiar(m):
        r, g, b = (float(v) for v in m.group(2, 3, 4))
        luz = (r + g + b) / 3
        color = "#FFFFFF" if luz > 90 else APAGADO if luz > 35 else TINTA
        return f'{m.group(1)}="{color}"'

    return COLOR.sub(cambiar, trazo)


def redondear(d):
    """Una décima de unidad del PDF basta para el dibujo y reduce mucho el archivo."""
    return NUMERO.sub(lambda m: f"{float(m.group()):.1f}".rstrip("0").rstrip("."), d)


def agrupar(trazos):
    """Une en un solo trazo los que comparten estilo y transformación."""
    grupos = {}
    for trazo in trazos:
        d = ATRIBUTO_D.search(trazo).group(1)
        resto = ATRIBUTO_D.sub("", trazo).rstrip("/>").rstrip()
        grupos.setdefault(resto, []).append(redondear(d))
    return [f'{resto} d="{" ".join(ds)}"/>' for resto, ds in grupos.items()]


def engordar(trazo, minimo):
    return re.sub(
        r'stroke-width="([\d.]+)"',
        lambda m: f'stroke-width="{max(float(m.group(1)), minimo):g}"',
        trazo,
    )


def extraer(pdf, pagina, recuadro):
    with tempfile.TemporaryDirectory() as carpeta:
        svg = Path(carpeta) / "pagina.svg"
        subprocess.run(
            ["pdftocairo", "-svg", "-f", str(pagina), "-l", str(pagina), pdf, str(svg)],
            check=True,
        )
        texto = svg.read_text()
    cuerpo = texto.split("</defs>", 1)[-1]
    return agrupar([recolorear(t) for t in TRAZO.findall(cuerpo) if dentro(t, recuadro)])


def main():
    argumentos = [a for a in sys.argv[1:] if not a.startswith("--")]
    grosor = next((float(a.split("=")[1]) for a in sys.argv if a.startswith("--grosor-minimo=")), 0)
    if len(argumentos) != 4:
        sys.exit(__doc__)
    pdf, pagina, recuadro, salida = argumentos
    x0, y0, x1, y1 = (float(v) for v in recuadro.split(","))
    trazos = [engordar(t, grosor) for t in extraer(pdf, int(pagina), (x0, y0, x1, y1))]
    if not trazos:
        sys.exit("No hay trazos dentro del recuadro.")
    ancho, alto = x1 - x0, y1 - y0
    if "--tumbado" in sys.argv:
        # Girar -90°: (x, y) pasa a (y, -x), así que la caja cambia de ejes.
        caja = f"{y0:.2f} {-x1:.2f} {alto:.2f} {ancho:.2f}"
        cuerpo = '<g transform="rotate(-90)">\n' + "\n".join(trazos) + "\n</g>"
    else:
        caja = f"{x0:.2f} {y0:.2f} {ancho:.2f} {alto:.2f}"
        cuerpo = "\n".join(trazos)
    Path(salida).write_text(
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{caja}">\n{cuerpo}\n</svg>\n'
    )
    print(f"{salida}: {len(trazos)} grupos de trazos")


if __name__ == "__main__":
    main()
