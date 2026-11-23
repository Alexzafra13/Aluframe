#!/usr/bin/env python3
"""Genera `data/callejero/<provincia>.tsv` a partir del callejero del censo electoral del INE.

Herramienta de desarrollo, no forma parte del programa (ADR-0023). Fuente: el archivo nacional
`caj_esp_MMAAAA.zip` de https://www.ine.es/prodyser/callejero/ (licencia CC BY 4.0; cita:
«Elaboración propia con datos extraídos del sitio web del INE: www.ine.es» y la fecha).

Del ZIP se usan tres archivos de ancho fijo en Latin-1:
- VIAS: el tipo (CALLE, AVDA…) y el nombre de cada vía.
- TRAM: los tramos de cada vía, con su código postal y su unidad de población.
- UP:   los nombres de los municipios, entidades y núcleos.

Resultado, por municipio:
  =Municipio|Localidad|Localidad…       (la primera es el municipio)
  Tipo<TAB>Nombre<TAB>cp|cp<TAB>0|2      (índices de las localidades de arriba donde está)
Una vía de un diseminado cuenta también como del municipio: «Zafra» encuentra sus fincas.

Uso:
  curl -o caj.zip https://www.ine.es/prodyser/callejero/caj_esp/caj_esp_072025.zip
  scripts/generar_callejero.py caj.zip 06 "julio de 2025" data/callejero/06.tsv
"""

import collections
import re
import sys
import unicodedata
import zipfile

TIPOS = {
    "ALDEA": "Aldea", "AVDA": "Avenida", "BARDA": "Barriada", "BARRO": "Barrio",
    "C.V.": "Camino vecinal", "CALLE": "Calle", "CAMI": "Camino", "CAMIN": "Camino",
    "CERRO": "Cerro", "CJLA": "Callejuela", "CLLJA": "Callejuela", "CLLON": "Callejón",
    "CMNO": "Camino", "COL": "Colonia", "CORDL": "Cordel", "CRTJO": "Cortijo",
    "CSRIO": "Caserío", "CTRA": "Carretera", "CÑADA": "Cañada", "EXPLA": "Explanada",
    "EXTRM": "Extramuros", "EXTRR": "Extrarradio", "FINCA": "Finca", "GRUP": "Grupo",
    "GRUPO": "Grupo", "GTA": "Glorieta", "LUGAR": "Lugar", "PARC": "Parque", "PASEO": "Paseo",
    "PISTA": "Pista", "PLAZA": "Plaza", "PLCET": "Placeta", "PLZLA": "Plazuela",
    "POLIG": "Polígono", "PQUE": "Parque", "PRAJE": "Paraje", "PROL": "Prolongación",
    "PSAJE": "Pasaje", "PZTA": "Plazoleta", "RBLA": "Rambla", "RCON": "Rincón",
    "RONDA": "Ronda", "TRAS": "Trasera", "TRAV": "Travesía", "TRVA": "Travesía",
    "URB": "Urbanización", "VALLE": "Valle", "VIA": "Vía", "VREDA": "Vereda", "ZONA": "Zona",
}
MINUSCULAS = {"a", "al", "de", "del", "e", "el", "en", "la", "las", "lo", "los", "y"}
ROMANOS = re.compile(r"^[IVXLC]+$")


def palabra_bonita(palabra, primera):
    if any(c.isdigit() for c in palabra) or (ROMANOS.match(palabra) and len(palabra) > 1):
        return palabra  # N-430, EX-101, XIII
    if not primera and palabra.lower() in MINUSCULAS:
        return palabra.lower()
    return "-".join(p[:1].upper() + p[1:].lower() for p in palabra.split("-"))


def nombre_bonito(nombre):
    """«SANTOS DE MAIMONA (LOS)» → «Los Santos de Maimona»; «XIII» y «N-430» se quedan."""
    nombre = " ".join(nombre.split())
    articulo = re.match(r"^(.*?)\s*\((EL|LA|LOS|LAS|L')\)$", nombre)
    if articulo:
        nombre = f"{articulo.group(2)} {articulo.group(1)}"
    palabras = nombre.split(" ")
    return " ".join(palabra_bonita(p, i == 0) for i, p in enumerate(palabras))


def clave(texto):
    sin_tildes = unicodedata.normalize("NFD", texto)
    return "".join(c for c in sin_tildes if unicodedata.category(c) != "Mn").casefold()


def con_mas_tildes(a, b):
    """El INE escribe unas veces «GEVORA» y otras «GÉVORA»: se queda la escrita con tildes."""
    return max(a, b, key=lambda n: (sum(not c.isascii() for c in n), n))


def leer(zip_, prefijo, provincia):
    nombre = next(n for n in zip_.namelist() if n.split("/")[-1].startswith(prefijo + "."))
    for renglon in zip_.read(nombre).decode("latin-1").splitlines():
        if renglon.startswith(provincia):
            yield renglon


def leer_unidades(zip_, provincia):
    """Por (municipio, código de unidad): el nombre de la entidad o del núcleo."""
    unidades = {}
    for r in leer(zip_, "UP", provincia):
        nombres = [n for n in re.split(r"\s{2,}", r[23:].strip()) if n]
        if nombres:
            unidades[(r[2:5], r[5:12])] = nombres[-1]
    return unidades


def leer_vias(zip_, provincia):
    vias = {}
    for r in leer(zip_, "VIAS", provincia):
        tipo, nombre = r[52:57].strip(), r[57:107].strip()
        if "DISEMINAD" in nombre or not nombre:
            continue  # «DISEMINADO» no es una calle sino «fuera del pueblo»
        if tipo not in TIPOS:
            print(f"aviso: tipo de vía desconocido {tipo!r} ({nombre})", file=sys.stderr)
        vias[(r[2:5], r[5:10])] = f"{TIPOS.get(tipo, tipo.title())}\t{nombre_bonito(nombre)}"
    return vias


def localidades_del_tramo(municipio, cun, unidades):
    # El código de unidad son 5 cifras de entidad y 2 de núcleo; «99» es su diseminado.
    entidad = unidades.get((municipio, cun[:5] + "00"))
    nucleo = unidades.get((municipio, cun))
    nombres = [entidad] if entidad else []
    if cun[5:7] == "99":
        nombres.append(unidades.get((municipio, "0000000")))
    elif nucleo:
        nombres.append(nucleo)
    return [nombre_bonito(n) for n in nombres if n]


def leer_tramos(zip_, provincia, unidades):
    codigos = collections.defaultdict(set)
    lugares = collections.defaultdict(set)
    for r in leer(zip_, "TRAM", provincia):
        clave, cp = (r[2:5], r[20:25]), r[42:47]
        if cp.isdigit() and cp != "00000":
            codigos[clave].add(cp)
        lugares[clave].update(localidades_del_tramo(r[2:5], r[13:20], unidades))
    return codigos, lugares


def escribir_municipio(salida, nombre, calles):
    escritas = {clave(nombre): nombre}
    for _, _, ls in calles:
        for l in ls:
            escritas[clave(l)] = con_mas_tildes(escritas.get(clave(l), l), l)
    otras = sorted(n for k, n in escritas.items() if k != clave(nombre))
    localidades = [escritas[clave(nombre)]] + otras
    posicion = {clave(l): i for i, l in enumerate(localidades)}
    salida.write("=" + "|".join(localidades) + "\n")
    for calle, cps, ls in sorted(calles):
        indices = sorted({posicion[clave(l)] for l in ls}) or [0]
        salida.write(f"{calle}\t{'|'.join(sorted(cps))}\t{'|'.join(map(str, indices))}\n")


def main():
    if len(sys.argv) != 5:
        sys.exit(__doc__)
    ruta_zip, provincia, fecha, ruta_salida = sys.argv[1:]
    zip_ = zipfile.ZipFile(ruta_zip)
    unidades = leer_unidades(zip_, provincia)
    vias = leer_vias(zip_, provincia)
    codigos, lugares = leer_tramos(zip_, provincia, unidades)
    por_municipio = collections.defaultdict(dict)
    for (municipio, cvia), calle in vias.items():
        # Dos vías con el mismo nombre en un municipio (con o sin tildes, o dadas de alta dos
        # veces) se juntan en una: para elegir la calle basta con una.
        anterior, cps, ls = por_municipio[municipio].get(clave(calle), (calle, set(), set()))
        por_municipio[municipio][clave(calle)] = (
            con_mas_tildes(anterior, calle),
            cps | codigos.get((municipio, cvia), set()),
            ls | lugares.get((municipio, cvia), set()),
        )
    with open(ruta_salida, "w", encoding="utf-8", newline="\n") as salida:
        salida.write(f"# Callejero de la provincia {provincia}. Elaboración propia con datos extraídos del "
                     f"sitio web del INE: www.ine.es (callejero del censo electoral, {fecha}). CC BY 4.0.\n")
        salida.write("# Generado con scripts/generar_callejero.py; no editar a mano.\n")
        for municipio in sorted(por_municipio):
            nombre = nombre_bonito(unidades.get((municipio, "0000000"), municipio))
            calles = list(por_municipio[municipio].values())
            escribir_municipio(salida, nombre, calles)


if __name__ == "__main__":
    main()
