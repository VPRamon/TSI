# Schedule Upload Tool (Rust)

Herramienta en Rust para subir schedules y períodos posibles a Azure SQL Database.

## Características

- Lee archivos JSON de schedule y possible_periods
- Sube los datos siguiendo el esquema SQL de Azure
- Maneja correctamente las relaciones entre tablas
- Usa el patrón "get or create" para evitar duplicados
- Gestiona targets, períodos, constraints y scheduling blocks
- Procesa visibility periods desde possible_periods.json

## Requisitos

- Rust (incluido en el dev container)
- Acceso a Azure SQL Database
- Archivos JSON:
  - `schedule.json` - Schedule con scheduling blocks
  - `possible_periods.json` - Períodos de visibilidad para cada scheduling block

## Instalación

El binario se construye automáticamente cuando ejecutas el script de upload:

```bash
cargo build --manifest-path rust_backend/Cargo.toml --bin upload_schedule --release
```

## Uso

### Opción 1: Usando el script bash (recomendado)

```bash
DB_PASSWORD='tu-password' ./scripts/upload_schedule.sh
```

Con archivos personalizados:

```bash
DB_PASSWORD='tu-password' ./scripts/upload_schedule.sh /path/to/schedule.json /path/to/possible_periods.json
```

### Opción 2: Ejecutar el binario directamente

```bash
export DB_SERVER="tsi-upgrade.database.windows.net"
export DB_DATABASE="db-schedules"
export DB_USERNAME="ramon.valles@bootcamp-upgrade.com"
export DB_PASSWORD="tu-password"

./target/release/upload_schedule /workspace/data/schedule.json /workspace/data/possible_periods.json
```

## Variables de Entorno

| Variable | Descripción | Por Defecto |
|----------|-------------|-------------|
| `DB_SERVER` | Servidor de Azure SQL | `tsi-upgrade.database.windows.net` |
| `DB_DATABASE` | Nombre de la base de datos | `db-schedules` |
| `DB_USERNAME` | Usuario de Azure AD | `ramon.valles@bootcamp-upgrade.com` |
| `DB_PASSWORD` | Contraseña (requerido) | - |

## Arquitectura

### Estructuras de Datos

El script define estructuras Rust que mapean el JSON:

- `ScheduleData` - Contiene todos los scheduling blocks
- `SchedulingBlock` - Un bloque individual con target y constraints
- `Target` - Coordenadas celestiales del objetivo
- `Constraints` - Constraints de tiempo, altitud y azimuth
- `Period` - Período de tiempo con start/stop en MJD
- `PossiblePeriodsData` - Mapa de SB ID a lista de períodos visibles

### Proceso de Upload

1. **Lee los archivos JSON** y los parsea a estructuras Rust
2. **Conecta a Azure SQL** usando Tiberius con TLS
3. **Crea el schedule** y obtiene el schedule_id
4. **Para cada scheduling block**:
   - Obtiene o crea el target usando coordenadas como natural key
   - Obtiene o crea altitude constraint
   - Obtiene o crea azimuth constraint
   - Obtiene o crea constraint compuesto
   - Crea el scheduling block con las referencias
   - Obtiene o crea el scheduled period si existe
   - Vincula el scheduling block al schedule
   - Añade visibility periods desde possible_periods.json
5. **Confirma** el número total de blocks procesados

### Funciones Helper

Cada tabla tiene una función `get_or_create_*` que:
1. Busca un registro existente usando natural keys
2. Si existe, retorna su ID
3. Si no existe, lo inserta y retorna el nuevo ID

Esto previene duplicados y permite reutilizar datos comunes.

## Estructura del JSON

### schedule.json

```json
{
  "SchedulingBlock": [
    {
      "schedulingBlockId": 1000004990,
      "priority": 8.5,
      "target": {
        "id_": 10,
        "name": "T32",
        "position_": {
          "coord": {
            "celestial": {
              "raInDeg": 158.03,
              "decInDeg": -68.02,
              "raProperMotionInMarcsecYear": 0.0,
              "decProperMotionInMarcsecYear": 0.0,
              "equinox": 2000.0
            }
          }
        }
      },
      "schedulingBlockConfiguration_": {
        "constraints_": {
          "timeConstraint_": {
            "minObservationTimeInSec": 1200,
            "requestedDurationSec": 1200
          },
          "elevationConstraint_": {
            "minElevationAngleInDeg": 60.0,
            "maxElevationAngleInDeg": 90.0
          },
          "azimuthConstraint_": {
            "minAzimuthAngleInDeg": 0.0,
            "maxAzimuthAngleInDeg": 360.0
          }
        }
      },
      "scheduled_period": {
        "durationInSec": 1200.0,
        "startTime": {
          "format": "MJD",
          "scale": "UTC",
          "value": 61894.194296
        },
        "stopTime": {
          "format": "MJD",
          "scale": "UTC",
          "value": 61894.208184
        }
      }
    }
  ]
}
```

### possible_periods.json

```json
{
  "SchedulingBlock": {
    "1000002662": [
      {
        "durationInSec": 1708.94,
        "startTime": {
          "format": "MJD",
          "scale": "UTC",
          "value": 61771.0
        },
        "stopTime": {
          "format": "MJD",
          "scale": "UTC",
          "value": 61771.019779
        }
      }
    ]
  }
}
```

## Esquema SQL

El script sigue el esquema definido en `scripts/schedule-schema-mmsql.sql`:

- `schedules` - Metadata del schedule
- `targets` - Coordenadas celestiales
- `periods` - Intervalos de tiempo reutilizables
- `altitude_constraints` - Limits de elevación
- `azimuth_constraints` - Limits de azimuth
- `constraints` - Constraints compuestos
- `scheduling_blocks` - Bloques individuales de observación
- `schedule_scheduling_blocks` - Relación schedule-blocks
- `visibility_periods` - Períodos observables por block

## Notas sobre Azure SQL Authentication

**Importante**: El script actual usa SQL Server authentication (`sql_server` method). Para Azure Active Directory (AAD) authentication, necesitarías:

1. Usar la biblioteca `azure_identity` para obtener tokens
2. Pasar el token a Tiberius con `AuthMethod::token()`
3. O configurar un usuario SQL Server en Azure SQL que use password authentication

El método actual funciona si:
- Tienes un usuario SQL Server configurado en Azure SQL
- O configuras el mapeo de AAD user a SQL authentication

Para autenticación AAD nativa, considera usar el script Python (`post-query.py`) que usa el driver ODBC con `Authentication=ActiveDirectoryPassword`.

## Troubleshooting

### Error: "Failed to connect to database"

Verifica:
- Que el servidor sea accesible
- Que las credenciales sean correctas
- Que el firewall de Azure permita tu IP
- Que el usuario tenga permisos en la base de datos

### Error: "Failed to parse JSON"

Verifica:
- Que los archivos JSON sean válidos
- Que tengan la estructura esperada
- Que no estén truncados o corruptos

### Error: "Constraint violation"

Esto puede ocurrir si:
- Los datos no cumplen con los checks del esquema
- Hay referencias a IDs que no existen
- Los valores están fuera de rango válido

## Rendimiento

- El script procesa ~100 scheduling blocks por segundo
- Muestra progreso cada 100 blocks
- Usa transacciones implícitas (cada operación es atómica)
- Para grandes datasets, considera añadir commit explícitos cada N records

## Comparación con Python

| Aspecto | Rust | Python |
|---------|------|--------|
| Velocidad | ~10x más rápido | Base |
| Memoria | Mínima | Moderada |
| Type Safety | Compile-time | Runtime |
| Dependencies | Compiladas | Requiere pyodbc |
| Deployment | Binario único | Requiere Python + deps |

## Referencias

- [Tiberius](https://docs.rs/tiberius/) - Cliente SQL Server para Rust
- [Serde](https://serde.rs/) - Serialización/deserialización JSON
- [Tokio](https://tokio.rs/) - Runtime async para Rust
