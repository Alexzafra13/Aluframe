#!/usr/bin/env python3
"""Genera `data/codigos_postales.tsv` a partir de los códigos postales de España de GeoNames.

Herramienta de desarrollo, no forma parte del programa (ADR-0020). Fuente: el archivo `ES.zip`
de https://download.geonames.org/export/zip/ (licencia CC BY 4.0, cita: GeoNames).

Cada renglón del resultado es `código<TAB>localidad|localidad…`: primero el municipio y después
los demás núcleos que GeoNames da para ese código. Se descartan las filas cuya provincia no es
la que marcan las dos primeras cifras del código (GeoNames tiene algunas mal asignadas): la
provincia la decide el programa por esas cifras, que es la regla de Correos.

Uso:
  curl -o ES.zip https://download.geonames.org/export/zip/ES.zip
  scripts/generar_codigos_postales.py ES.zip data/codigos_postales.tsv
"""

import collections
import sys
import unicodedata
import zipfile

PROVINCIAS = [
    "Álava", "Albacete", "Alicante", "Almería", "Ávila", "Badajoz", "Baleares", "Barcelona",
    "Burgos", "Cáceres", "Cádiz", "Castellón", "Ciudad Real", "Córdoba", "A Coruña", "Cuenca",
    "Girona", "Granada", "Guadalajara", "Guipúzcoa", "Huelva", "Huesca", "Jaén", "León",
    "Lleida", "La Rioja", "Lugo", "Madrid", "Málaga", "Murcia", "Navarra", "Ourense",
    "Asturias", "Palencia", "Las Palmas", "Pontevedra", "Salamanca", "Santa Cruz de Tenerife",
    "Cantabria", "Segovia", "Sevilla", "Soria", "Tarragona", "Teruel", "Toledo", "Valencia",
    "Valladolid", "Vizcaya", "Zamora", "Zaragoza", "Ceuta", "Melilla",
]


def clave(texto):
    sin_tildes = unicodedata.normalize("NFD", texto)
    return "".join(c for c in sin_tildes if unicodedata.category(c) != "Mn").casefold().strip()


MINUSCULAS = {"de", "del", "la", "las", "los", "el", "y", "en", "a"}


def arreglar_mayusculas(nombre):
    """GeoNames escribe «San Pedro De Arlanza»; en España se escribe «San Pedro de Arlanza»."""
    palabras = nombre.split(" ")
    return " ".join(
        p.lower() if i > 0 and p.lower() in MINUSCULAS else p for i, p in enumerate(palabras)
    )


def leer(ruta_zip):
    texto = zipfile.ZipFile(ruta_zip).read("ES.txt").decode("utf-8")
    localidades = collections.defaultdict(list)
    for renglon in texto.splitlines():
        campos = renglon.split("\t")
        cp, lugar, provincia, municipio, ine = campos[1], campos[2], campos[5], campos[7], campos[8]
        numero = int(cp[:2]) if cp[:2].isdigit() else 0
        if len(cp) != 5 or not 1 <= numero <= 52:
            continue
        if clave(provincia) != clave(PROVINCIAS[numero - 1]):
            continue
        # El código INE del municipio empieza por el de su provincia: si no cuadra con el código
        # postal, el municipio está mal asignado (46005 figura como «Madrid») y se descarta.
        if not ine.startswith(cp[:2]):
            municipio = ""
        for nombre in (municipio, lugar):
            nombre = arreglar_mayusculas(nombre.strip())
            ya = {clave(n) for n in localidades[cp]}
            if nombre and clave(nombre) not in ya:
                localidades[cp].append(nombre)
    return localidades


def main():
    if len(sys.argv) != 3:
        sys.exit(__doc__)
    localidades = leer(sys.argv[1])
    with open(sys.argv[2], "w", encoding="utf-8", newline="\n") as salida:
        salida.write("# Códigos postales de España: GeoNames (www.geonames.org), CC BY 4.0.\n")
        salida.write("# Generado con scripts/generar_codigos_postales.py; no editar a mano.\n")
        for cp in sorted(localidades):
            salida.write(f"{cp}\t{'|'.join(localidades[cp])}\n")
    print(f"{len(localidades)} códigos postales")


if __name__ == "__main__":
    main()
