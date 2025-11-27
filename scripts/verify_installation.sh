#!/bin/bash

# Script de verificación post-implementación
# Verifica que todos los componentes del upload tool estén en su lugar

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  Upload Schedule Tool - Verificación de Instalación           ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

check_file() {
    if [ -f "$1" ]; then
        echo -e "${GREEN}✓${NC} $1"
        return 0
    else
        echo -e "${RED}✗${NC} $1 ${RED}(MISSING)${NC}"
        return 1
    fi
}

check_executable() {
    if [ -x "$1" ]; then
        echo -e "${GREEN}✓${NC} $1 ${GREEN}(executable)${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠${NC} $1 ${YELLOW}(not executable)${NC}"
        return 1
    fi
}

check_command() {
    if command -v "$1" &> /dev/null; then
        echo -e "${GREEN}✓${NC} $1 command available"
        return 0
    else
        echo -e "${RED}✗${NC} $1 ${RED}(not found)${NC}"
        return 1
    fi
}

total_checks=0
passed_checks=0

echo "1. Verificando archivos principales..."
echo "────────────────────────────────────────"

files=(
    "/workspace/rust_backend/src/bin/upload_schedule.rs"
    "/workspace/scripts/upload_schedule.sh"
    "/workspace/scripts/test_upload.sh"
    "/workspace/docs/upload_schedule_rust.md"
    "/workspace/docs/upload_schedule_quickstart.md"
    "/workspace/scripts/verify_schedule_queries.sql"
    "/workspace/UPLOAD_SCHEDULE_SUMMARY.md"
)

for file in "${files[@]}"; do
    ((total_checks++))
    if check_file "$file"; then
        ((passed_checks++))
    fi
done
echo

echo "2. Verificando scripts ejecutables..."
echo "────────────────────────────────────────"

executables=(
    "/workspace/scripts/upload_schedule.sh"
    "/workspace/scripts/test_upload.sh"
)

for exe in "${executables[@]}"; do
    ((total_checks++))
    if check_executable "$exe"; then
        ((passed_checks++))
    fi
done
echo

echo "3. Verificando binario compilado..."
echo "────────────────────────────────────────"

((total_checks++))
if check_file "/workspace/target/release/upload_schedule"; then
    ((passed_checks++))
    size=$(du -h /workspace/target/release/upload_schedule | cut -f1)
    echo "   Tamaño: $size"
fi
echo

echo "4. Verificando dependencias del sistema..."
echo "────────────────────────────────────────"

commands=(
    "cargo"
    "rustc"
    "python3"
)

for cmd in "${commands[@]}"; do
    ((total_checks++))
    if check_command "$cmd"; then
        ((passed_checks++))
    fi
done
echo

echo "5. Verificando estructura de datos..."
echo "────────────────────────────────────────"

data_files=(
    "/workspace/data/schedule.json"
    "/workspace/data/possible_periods.json"
)

for file in "${data_files[@]}"; do
    ((total_checks++))
    if check_file "$file"; then
        ((passed_checks++))
        size=$(du -h "$file" | cut -f1)
        echo "   Tamaño: $size"
    fi
done
echo

echo "6. Verificando schema SQL..."
echo "────────────────────────────────────────"

((total_checks++))
if check_file "/workspace/scripts/schedule-schema-mmsql.sql"; then
    ((passed_checks++))
    tables=$(grep -c "CREATE TABLE" /workspace/scripts/schedule-schema-mmsql.sql)
    echo "   Tablas definidas: $tables"
fi
echo

echo "7. Verificando Cargo.toml..."
echo "────────────────────────────────────────"

((total_checks++))
if check_file "/workspace/rust_backend/Cargo.toml"; then
    ((passed_checks++))
    if grep -q "tiberius" /workspace/rust_backend/Cargo.toml; then
        echo -e "   ${GREEN}✓${NC} Dependencia tiberius encontrada"
    fi
    if grep -q "upload_schedule" /workspace/rust_backend/Cargo.toml; then
        echo -e "   ${GREEN}✓${NC} Binario upload_schedule configurado"
    fi
fi
echo

# Summary
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║  RESUMEN                                                       ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo

percentage=$((passed_checks * 100 / total_checks))

echo "Verificaciones pasadas: $passed_checks/$total_checks ($percentage%)"
echo

if [ $percentage -eq 100 ]; then
    echo -e "${GREEN}✓ Todos los componentes están instalados correctamente${NC}"
    echo
    echo "Próximo paso: Ejecutar el upload"
    echo "  DB_PASSWORD='password' ./scripts/upload_schedule.sh"
    exit 0
elif [ $percentage -ge 80 ]; then
    echo -e "${YELLOW}⚠ La mayoría de los componentes están instalados${NC}"
    echo "  Algunas verificaciones fallaron pero el sistema debería funcionar."
    exit 0
else
    echo -e "${RED}✗ Faltan componentes importantes${NC}"
    echo "  Por favor, revisa los errores arriba."
    exit 1
fi
