#!/usr/bin/env python3
"""Pasa la imagen de un herraje sacada de un catálogo PDF a un SVG en tinta sobre transparente.

Herramienta de desarrollo, no forma parte del programa (ADR-0029). Para los catálogos que no
traen el herraje en vector (el de Aluval). Necesita Pillow. Primero se saca la imagen original,
sin pérdida, con `pdfimages -png -f N -l N catalogo.pdf /tmp/img` y se elige la que toca.

Uso:
  scripts/imagen_de_herraje.py /tmp/img-015.png data/perfiles/herrajes/rueda.svg

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


def convertir(origen):
    gris = Image.open(origen).convert("L")
    caja = gris.point(lambda g: 255 if g < FONDO - 5 else 0).getbbox()
    gris = gris.crop(caja)
    mascara = gris.point(alfa)
    salida = Image.new("RGBA", gris.size, (*TINTA, 0))
    salida.putalpha(mascara)
    return salida


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    imagen = convertir(sys.argv[1])
    datos = io.BytesIO()
    imagen.save(datos, "PNG", optimize=True)
    ancho, alto = imagen.size
    codificada = base64.b64encode(datos.getvalue()).decode()
    Path(sys.argv[2]).write_text(
        f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {ancho} {alto}">'
        f'<image width="{ancho}" height="{alto}" href="data:image/png;base64,{codificada}"/></svg>\n'
    )
    print(f"{sys.argv[2]}: {ancho} × {alto} px")


if __name__ == "__main__":
    main()
