#!/bin/bash
# Script para compilar el backend Rust después de la migración
# 
# Este script compila la nueva funcionalidad agregada (load_schedule_from_iteration)

set -e

echo "=================================="
echo "Compilando Backend Rust"
echo "=================================="
echo

# Limpiar compilaciones previas si hay bloqueos
if [ -f "/workspace/target/.rustc_info.json" ]; then
    echo "Limpiando cache de compilación..."
    rm -rf /workspace/target/.rustc_info.json
fi

# Navegar al directorio del backend
cd /workspace/rust_backend

echo "Compilando con maturin..."
echo

# Compilar e instalar en modo release
maturin develop --release

echo
echo "=================================="
echo "✅ Compilación completada"
echo "=================================="
echo
echo "Verificar instalación:"
echo "  python -c 'import tsi_rust; print(\"load_schedule_from_iteration\" in dir(tsi_rust))'"
echo
