#!/usr/bin/env bash
# Instala o actualiza AlumFrame como servicio systemd. Ejecutar con sudo desde la carpeta descomprimida.
set -euo pipefail

if [ "$(id -u)" -ne 0 ]; then
  echo "Ejecuta: sudo ./instalar.sh" >&2
  exit 1
fi

origen="$(cd "$(dirname "$0")" && pwd)"

if ! id alumframe >/dev/null 2>&1; then
  useradd --system --home-dir /var/lib/alumframe --shell /usr/sbin/nologin alumframe
fi

install -d -o alumframe -g alumframe -m 750 /var/lib/alumframe
install -d -m 755 /opt/alumframe /etc/alumframe

if [ ! -f /etc/alumframe/alumframe.toml ]; then
  install -m 644 "$origen/alumframe.toml" /etc/alumframe/alumframe.toml
fi

if systemctl is-active --quiet alumframe; then
  systemctl stop alumframe
fi
if [ -f /opt/alumframe/alumframe ]; then
  cp -p /opt/alumframe/alumframe /opt/alumframe/alumframe.anterior
fi
install -m 755 "$origen/alumframe" /opt/alumframe/alumframe
install -m 644 "$origen/alumframe.service" /etc/systemd/system/alumframe.service

systemctl daemon-reload
systemctl enable --now alumframe

ip_local="$(hostname -I 2>/dev/null | awk '{print $1}')"
echo "AlumFrame instalado. Ábrelo en el navegador: http://${ip_local:-IP-DEL-SERVIDOR}:9090"
echo "Estado: systemctl status alumframe   ·   Registros: journalctl -u alumframe"
