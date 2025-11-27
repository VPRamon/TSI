# Upload Schedule Tool - Quick Reference

## Uso Rápido

### 1. Configurar credenciales

```bash
export DB_PASSWORD="tu-password"
```

### 2. Ejecutar el upload

```bash
./scripts/upload_schedule.sh
```

### 3. Con archivos personalizados

```bash
DB_PASSWORD='password' ./scripts/upload_schedule.sh /ruta/schedule.json /ruta/possible_periods.json
```

## Verificación

Después del upload, puedes verificar con el script Python:

```python
from scripts.post_query import get_schedule

# Usa el schedule_id retornado por el script
get_schedule(schedule_id)
```

## Estructura Esperada

### schedule.json
- Array de SchedulingBlock
- Cada block tiene: target, constraints, priority, scheduled_period (opcional)

### possible_periods.json  
- Objeto con "SchedulingBlock" como key
- Valor es un map de scheduling_block_id → array de períodos

## Dependencias Rust

Las siguientes dependencias se añadieron a `rust_backend/Cargo.toml`:

```toml
tiberius = { version = "0.12", default-features = false, features = ["tds73", "rustls"] }
tokio = { version = "1", features = ["full"] }
tokio-util = { version = "0.7", features = ["compat"] }
```

## Binario

Ubicación: `/workspace/target/release/upload_schedule`

Tamaño: ~15-20 MB (optimizado)

Uso de memoria: ~50-100 MB durante ejecución

## Troubleshooting Rápido

**No conecta a la BD:**
```bash
# Verifica conectividad
ping tsi-upgrade.database.windows.net
```

**Error de parsing JSON:**
```bash
# Valida JSON
python -m json.tool /workspace/data/schedule.json > /dev/null
python -m json.tool /workspace/data/possible_periods.json > /dev/null
```

**Build fails:**
```bash
# Limpia y reconstruye
cargo clean --manifest-path rust_backend/Cargo.toml
cargo build --manifest-path rust_backend/Cargo.toml --bin upload_schedule --release
```
