#!/usr/bin/env python3
"""Extrae las siluetas (secciones) de los perfiles de un catálogo PDF de fabricante.

Herramienta de desarrollo, no forma parte del programa (ADR-0017). Necesita `pdftocairo` y
`pdftotext` (poppler). Para cada dibujo de perfil de las páginas indicadas:
  1. convierte la página a SVG y se queda con los trazos grises del perfil (sin cotas ni texto);
  2. agrupa los trazos que se tocan: cada grupo es un perfil;
  3. lee el nombre que hay debajo de cada grupo («MARCO LATERAL»);
  4. guarda `marco-lateral.svg` con solo la silueta, en coordenadas propias y sin estilos.

La referencia de 5 cifras del catálogo de Aluval no es texto sino trazos (letra de CAD): se
renombra cada archivo a su referencia a mano, comprobándola en la página del catálogo.

Uso:
  scripts/extraer_siluetas.py catalogo.pdf 4-6 data/perfiles/aluval
Escribe también `indice.txt` (archivo, nombre, página) para revisar a mano qué se ha sacado.
"""

import re
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET
from pathlib import Path

GRIS = "rgb(50.196838%, 50.196838%, 50.196838%)"
NUMERO = re.compile(r"-?\d+(?:\.\d+)?")
MATRIZ = re.compile(r"matrix\(([^)]*)\)")
SEPARACION = 2.5  # puntos: trazos más cerca que esto son del mismo perfil


def transformar(punto, matriz):
    a, b, c, d, e, f = matriz
    x, y = punto
    return (a * x + c * y + e, b * x + d * y + f)


def leer_trazos(svg):
    """Trazos grises del perfil, ya en coordenadas de página: (d_transformado, caja)."""
    trazos = []
    for elemento in ET.parse(svg).iter("{http://www.w3.org/2000/svg}path"):
        if elemento.get("fill") != GRIS and elemento.get("stroke") != GRIS:
            continue
        coincidencia = MATRIZ.search(elemento.get("transform", ""))
        matriz = [float(v) for v in coincidencia.group(1).split(",")] if coincidencia else [1, 0, 0, 1, 0, 0]
        d = elemento.get("d", "")
        partes = re.split(r"([MLCZ])", d)
        salida, puntos, orden = [], [], None
        for parte in partes:
            parte = parte.strip()
            if parte in ("M", "L", "C", "Z"):
                orden = parte
                salida.append(parte)
                continue
            numeros = [float(n) for n in NUMERO.findall(parte)]
            for i in range(0, len(numeros) - 1, 2):
                x, y = transformar((numeros[i], numeros[i + 1]), matriz)
                puntos.append((x, y))
                salida.append(f"{x:.2f} {y:.2f}")
        if not puntos:
            continue
        xs, ys = [p[0] for p in puntos], [p[1] for p in puntos]
        relleno = elemento.get("fill") == GRIS
        trazos.append((" ".join(salida), (min(xs), min(ys), max(xs), max(ys)), relleno))
    return trazos


def se_tocan(a, b):
    return not (a[2] + SEPARACION < b[0] or b[2] + SEPARACION < a[0]
                or a[3] + SEPARACION < b[1] or b[3] + SEPARACION < a[1])


def agrupar(trazos):
    padre = list(range(len(trazos)))

    def raiz(i):
        while padre[i] != i:
            padre[i] = padre[padre[i]]
            i = padre[i]
        return i

    orden = sorted(range(len(trazos)), key=lambda i: trazos[i][1][0])
    for posicion, i in enumerate(orden):
        for j in orden[posicion + 1:]:
            if trazos[j][1][0] > trazos[i][1][2] + SEPARACION:
                break
            if se_tocan(trazos[i][1], trazos[j][1]):
                padre[raiz(i)] = raiz(j)
    grupos = {}
    for i in range(len(trazos)):
        grupos.setdefault(raiz(i), []).append(trazos[i])
    return list(grupos.values())


def caja_de(grupo):
    cajas = [t[1] for t in grupo]
    return (min(c[0] for c in cajas), min(c[1] for c in cajas),
            max(c[2] for c in cajas), max(c[3] for c in cajas))


def leer_palabras(pdf, pagina):
    salida = subprocess.run(
        ["pdftotext", "-bbox", "-f", str(pagina), "-l", str(pagina), pdf, "-"],
        check=True, capture_output=True, text=True).stdout
    palabras = []
    for m in re.finditer(r'xMin="([\d.]+)" yMin="([\d.]+)" xMax="([\d.]+)" yMax="([\d.]+)">([^<]*)<', salida):
        x0, y0, x1, y1 = (float(v) for v in m.groups()[:4])
        palabras.append((x0, y0, x1, y1, m.group(5)))
    return palabras


def nombre_bajo(caja, palabras):
    """El texto de la línea que hay justo debajo del dibujo (el nombre del perfil)."""
    centro = (caja[0] + caja[2]) / 2
    ancho = (caja[2] - caja[0]) / 2 + 40
    debajo = [p for p in palabras if p[1] >= caja[3] - 2 and p[1] - caja[3] < 40
              and abs((p[0] + p[2]) / 2 - centro) < ancho and re.search(r"[A-ZÁÉÍÓÚÑ]{2}", p[4])]
    if not debajo:
        return None
    primera = min(p[1] for p in debajo)
    # Toda la línea, cifras incluidas («HOJA LATERAL 55», «JUNQUILLO 27»).
    linea = sorted((p for p in palabras if abs(p[1] - primera) < 3
                    and abs((p[0] + p[2]) / 2 - centro) < ancho), key=lambda p: p[0])
    return " ".join(p[4] for p in linea)


def a_nombre_de_archivo(nombre):
    sin_tildes = nombre.lower().translate(str.maketrans("áéíóúñ", "aeioun"))
    return re.sub(r"[^a-z0-9]+", "-", sin_tildes).strip("-")


def desplazar(d, dx, dy):
    """Lleva las coordenadas al origen del dibujo y las deja con un decimal."""
    partes = re.split(r"([MLCZ])", d)
    texto = []
    for parte in partes:
        parte = parte.strip()
        if parte in ("M", "L", "C", "Z"):
            texto.append(parte)
            continue
        numeros = [float(n) for n in NUMERO.findall(parte)]
        for i in range(0, len(numeros) - 1, 2):
            texto.append(f"{numeros[i] - dx:.1f} {numeros[i + 1] - dy:.1f}")
    return " ".join(texto)


def escribir_svg(grupo, destino):
    x0, y0, x1, y1 = caja_de(grupo)
    margen = 1
    dx, dy = x0 - margen, y0 - margen
    # Solo los rellenos (triángulos) forman la silueta maciza; un trazo fino del mismo color
    # tapa las juntas entre triángulos al verla pequeña. Color: tinta del diseño (#15191D).
    d = " ".join(desplazar(t[0], dx, dy) for t in grupo if t[2])
    destino.write_text(
        '<svg xmlns="http://www.w3.org/2000/svg" '
        f'viewBox="0 0 {x1 - x0 + 2 * margen:.1f} {y1 - y0 + 2 * margen:.1f}">'
        f'<path fill="#15191D" stroke="#15191D" stroke-width=".25" stroke-linejoin="round" d="{d}"/>'
        "</svg>\n",
        encoding="utf-8")


def paginas(texto):
    inicio, _, fin = texto.partition("-")
    return range(int(inicio), int(fin or inicio) + 1)


def main():
    if len(sys.argv) != 4:
        print(__doc__)
        sys.exit(1)
    pdf, rango, carpeta = sys.argv[1], sys.argv[2], Path(sys.argv[3])
    carpeta.mkdir(parents=True, exist_ok=True)
    indice = []
    for pagina in paginas(rango):
        with tempfile.TemporaryDirectory() as temporal:
            svg = Path(temporal) / "pagina.svg"
            subprocess.run(["pdftocairo", "-svg", "-f", str(pagina), "-l", str(pagina), pdf, str(svg)],
                           check=True)
            grupos = [g for g in agrupar(leer_trazos(svg)) if any(t[2] for t in g)]
        palabras = leer_palabras(pdf, pagina)
        for grupo in grupos:
            caja = caja_de(grupo)
            if (caja[2] - caja[0]) < 15 and (caja[3] - caja[1]) < 15:
                continue  # restos sueltos, no un perfil
            nombre = nombre_bajo(caja, palabras)
            if not nombre:
                continue
            archivo = a_nombre_de_archivo(nombre)
            escribir_svg(grupo, carpeta / f"{archivo}.svg")
            indice.append(f"{archivo}.svg\t{nombre}\tpágina {pagina}")
    (carpeta / "indice.txt").write_text("\n".join(sorted(indice)) + "\n", encoding="utf-8")
    print(f"{len(indice)} siluetas en {carpeta}")


if __name__ == "__main__":
    main()
