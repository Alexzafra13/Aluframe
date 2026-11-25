set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list

# Arranca el servidor en modo local con datos de desarrollo
dev:
    ALUMFRAME_MODO=local ALUMFRAME_ABRIR_NAVEGADOR=no ALUMFRAME_CARPETA_DATOS={{justfile_directory()}}/datos-dev cargo run -p servidor

# Formato, clippy, tests y tamaño de archivos (también en el hook previo al push y en CI)
check: fmt-check lint lint-size test

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets --locked -- -D warnings

test:
    cargo test --workspace --locked

# Falla si algún archivo .rs supera las 300 líneas
lint-size:
    #!/usr/bin/env bash
    set -euo pipefail
    grandes=$(find crates -name '*.rs' -print0 | xargs -0 wc -l | awk '$2 != "total" && $1 > 300 {print}')
    if [ -n "$grandes" ]; then echo "Archivos de más de 300 líneas:"; echo "$grandes"; exit 1; fi

deny:
    cargo deny check

# Lista las dependencias publicadas hace menos de 14 días
antiguedad:
    python3 -I scripts/comprobar_antiguedad.py

# E2E con el perfil `e2e` (optimizado, sin el LTO lento); CI la pasa con el de `release` (ADR-0035)
e2e:
    cargo build --profile e2e --locked -p servidor
    cd e2e && npm ci --no-audit --no-fund && ALUMFRAME_BIN={{justfile_directory()}}/target/e2e/alumframe npx playwright test --trace off

build:
    cargo build --release --locked -p servidor

release: check build
    @echo "Ejecutable listo en target/release/alumframe"

db-migrate:
    cargo run -p servidor -- preparar-base

cobertura:
    cargo llvm-cov --package dominio --fail-under-lines 90

# Tiempos de compilación y pruebas, líneas y tamaños, como fila de docs/METRICAS.md (--limpia: desde cero)
metricas *opciones:
    scripts/metricas.sh {{opciones}}

# Borra lo compilado (target crece con cada versión de las dependencias y de Rust)
limpiar:
    cargo clean

instalar-hooks:
    git config core.hooksPath .githooks
