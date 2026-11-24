#!/usr/bin/env python3
"""Pasa la imagen de un herraje sacada de un catálogo PDF a un SVG en tinta sobre transparente.

Herramienta de desarrollo, no forma parte del programa (ADR-0029). Para los catálogos que no
traen el herraje en vector (el de Aluval). Necesita Pillow. Primero se saca la imagen original,
sin pérdida, con `pdfimages -png -f N -l N catalogo.pdf /tmp/img` y se elige la que toca.

Uso:
  scripts/imagen_de_herraje.py /tmp/img-015.png data/perfiles/herrajes/rueda.svg
  scripts/imagen_de_herraje.py plano.jpg data/perfiles/mosquitera/cajon.svg --recorte 150,88,610,690 --quitar-azul

`--recorte x0,y0,x1,y1` se queda con esa zona (en píxeles) antes de recortar el blanco: sirve para
dejar fuera cotas y textos. `--quitar-azul` borra las líneas de cota azules de los planos.
`--borrar x0,y0,x1,y1` (se puede repetir) pinta de blanco una zona, en píxeles de la imagen
original: para quitar lo que no forma parte del dibujo, como los tornillos de un kit.
También vale para siluetas de perfiles cuando el fabricante solo da una imagen.

Recorta el fondo blanco, pasa cada gris a tinta con transparencia y refuerza las líneas finas
(`CONTRASTE`) para que se lean en el hueco de 56 × 42 px de la lista.
"""

import base64
import io
import sys
from pathlib import Path

from PIL import Image

TINTA = (0x15, 0x19, 0x1D)
FONDO = 240  # gris a partir del cual el píxel es papel
CONTRASTE = 110  # cuanto menor, más opacas salen las líneas grises


def alfa(gris):
    return max(0, min(255, (FONDO - gris) * 255 // CONTRASTE))


def es_azul(r, g, b):
    return b > r + 40 and b > g + 20


def preparar(origen, recorte, quitar_azul, borrar=()):
    imagen = Image.open(origen).convert("RGB")
    for zona in borrar:
        imagen.paste((255, 255, 255), zona)
    if recorte:
        imagen = imagen.crop(recorte)
    if quitar_azul:
        imagen.putdata([(255, 255, 255) if es_azul(*p) else p for p in imagen.getdata()])
    return imagen


def convertir(origen, recorte=None, quitar_azul=False, borrar=()):
    gris = preparar(origen, recorte, quitar_azul, borrar).convert("L")
    caja = gris.point(lambda g: 255 if g < FONDO - 5 else 0).getbbox()
    gris = gris.crop(caja)
    mascara = gris.point(alfa)
    salida = Image.new("RGBA", gris.size, (*TINTA, 0))
    salida.putalpha(mascara)
    return salida


def leer_opciones(argumentos):
    recorte, quitar_azul, rutas, borrar = None, False, [], []
    pendientes = list(argumentos)
    while pendientes:
        arg = pendientes.pop(0)
        if arg == "--recorte" and pendientes:
            recorte = tuple(int(v) for v in pendientes.pop(0).split(","))
        elif arg == "--borrar" and pendientes:
            borrar.append(tuple(int(v) for v in pendientes.pop(0).split(",")))
        elif arg == "--quitar-azul":
            quitar_azul = True
        else:
            rutas.append(arg)
    return rutas, recorte, quitar_azul, borrar


def main():
    rutas, recorte, quitar_azul, borrar = leer_opciones(sys.argv[1:])
    if len(rutas) != 2:
        sys.exit(__doc__)
    imagen = convertir(rutas[0], recorte, quitar_azul, borrar)
    datos = io.BytesIO()
    imagen.save(datos, "PNG", optimize=True)
    ancho, alto = imagen.size
    codificada = base64.b64encode(datos.getvalue()).decode()
    Path(rutas[1]).write_text(
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {ancho} {alto}">'
        f'<image width="{ancho}" height="{alto}" href="data:image/png;base64,{codificada}"/></svg>\n'
    )
    print(f"{rutas[1]}: {ancho} × {alto} px")


if __name__ == "__main__":
    main()
