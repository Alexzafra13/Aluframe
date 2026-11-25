#!/usr/bin/env bash
# Mide lo que cuesta trabajar en el proyecto y lo deja como una fila para docs/METRICAS.md
# (ADR-0035). Con `--limpia` también mide una compilación desde cero (borra `target`).
set -euo pipefail
cd "$(dirname "$0")/.."

segundos() {
    local inicio
    inicio=$(date +%s.%N)
    "$@" >/dev/null 2>&1
    awk -v a="$inicio" -v b="$(date +%s.%N)" 'BEGIN { printf "%.0f", b - a }'
}

limpia="—"
if [ "${1:-}" = "--limpia" ]; then
    cargo clean >/dev/null 2>&1
    limpia="$(segundos cargo test --workspace --locked --no-run) s"
fi

cargo test --workspace --locked --no-run >/dev/null 2>&1
touch crates/servidor/src/lib.rs
incremental=$(segundos cargo test --workspace --locked --no-run)

inicio=$(date +%s.%N)
salida=$(cargo test --workspace --locked 2>&1)
pruebas_s=$(awk -v a="$inicio" -v b="$(date +%s.%N)" 'BEGIN { printf "%.0f", b - a }')
pruebas=$(grep -o '[0-9]* passed' <<<"$salida" | awk '{ n += $1 } END { print n }')

lineas=$(find crates -name '*.rs' -print0 | xargs -0 cat | wc -l)
target=$(du -sh target 2>/dev/null | cut -f1)
ejecutable="—"
if [ -f target/release/alumframe ]; then
    ejecutable="$(du -h target/release/alumframe | cut -f1)"
fi
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)

echo "| Fecha | Versión | Compilación limpia | Tras tocar un archivo | Pruebas (n.º y tiempo) | Líneas de Rust | target | Ejecutable |"
echo "|---|---|---|---|---|---|---|---|"
echo "| $(date +%F) | $version | $limpia | $incremental s | $pruebas en $pruebas_s s | $lineas | $target | $ejecutable |"
