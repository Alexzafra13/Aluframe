"""Lista los paquetes de Cargo.lock publicados hace menos de 14 días (regla de CLAUDE.md)."""

import datetime
import json
import sys
import time
import tomllib
import urllib.request

DIAS_MINIMOS = 14
AGENTE = "alumframe-comprobar-antiguedad"


def fecha_de_publicacion(nombre, version):
    url = f"https://crates.io/api/v1/crates/{nombre}/{version}"
    peticion = urllib.request.Request(url, headers={"User-Agent": AGENTE})
    with urllib.request.urlopen(peticion, timeout=30) as respuesta:
        datos = json.load(respuesta)
    return datetime.datetime.fromisoformat(datos["version"]["created_at"])


def main():
    with open("Cargo.lock", "rb") as archivo:
        paquetes = tomllib.load(archivo)["package"]
    limite = datetime.datetime.now(datetime.timezone.utc) - datetime.timedelta(days=DIAS_MINIMOS)
    recientes = []
    for paquete in paquetes:
        if "registry+" not in paquete.get("source", ""):
            continue
        publicado = fecha_de_publicacion(paquete["name"], paquete["version"])
        if publicado > limite:
            recientes.append(f"{paquete['name']} {paquete['version']} ({publicado.date()})")
        time.sleep(1)
    for linea in recientes:
        print(linea)
    return 1 if recientes else 0


if __name__ == "__main__":
    sys.exit(main())
