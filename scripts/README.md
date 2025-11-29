# Scripts Directory

Este directorio contiene scripts para interactuar con la base de datos Azure SQL.

## Scripts Disponibles

### 1. schedule-schema-mmsql.sql ⭐ SCRIPT PRINCIPAL DE BASE DE DATOS

Script **COMPLETO** para crear la base de datos desde cero.

**Incluye:**
- ✅ Creación de todas las tablas (9 tablas)
- ✅ Constraints y checks de integridad
- ✅ Índices básicos (5 índices)
- ✅ Índices de optimización de performance (10 índices críticos)
- ✅ Actualización de estadísticas

**Uso:**
```bash
# Opción 1: Azure Portal Query Editor
# - Abre el archivo y copia/pega todo el contenido
# - Ejecuta en Query Editor

# Opción 2: sqlcmd
sqlcmd -S tsi-upgrade.database.windows.net \
       -d db-schedules \
       -U ramon.valles@bootcamp-upgrade.com \
       -P "$DB_PASSWORD" \
       -i schedule-schema-mmsql.sql
```

**⚠️ NOTA:** Este script es **idempotente** pero hace DROP de tablas existentes. Ver `DATABASE_SETUP.md` para más detalles.

**Documentación completa:** [DATABASE_SETUP.md](./DATABASE_SETUP.md)

---

### 2. upload_schedule.sh (Rust) ⭐ RECOMENDADO PARA UPLOADS

Script en **Rust** para subir schedules a Azure SQL de forma eficiente.

**Características:**
- ⚡ Alto rendimiento (~10x más rápido que Python)
- 🔒 Type-safe (seguridad de tipos en compile-time)
- 📦 Binario independiente (no requiere Python)
- 🔄 Maneja duplicados automáticamente (get-or-create pattern)
- 📊 Procesa visibility periods desde possible_periods.json

**Uso:**
```bash
DB_PASSWORD='tu-password' ./scripts/upload_schedule.sh
```

**Documentación completa:** [docs/upload_schedule_rust.md](../docs/upload_schedule_rust.md)

---

### 2. post-query.py (Python)

Script en **Python** para subir schedules usando pyodbc con Azure AD authentication.


**Características:**
- 🔑 Azure Active Directory password authentication
- 🐍 Implementación Python pura
- 📝 Código más legible y fácil de modificar
- 🔍 Incluye función para consultar schedules

**Uso:**
```python
from scripts.post_query import upload_minimal_schedule, get_schedule

schedule_id = upload_minimal_schedule()
get_schedule(schedule_id)
```

**Requisitos:**
- pyodbc
- Microsoft ODBC Driver 18 for SQL Server

---

## Prerequisites
- Python 3.x
- `pyodbc` package (`pip install pyodbc`)
- Valid database credentials in `scripts/db_credentials.py`

## Setup
1. Clone the repository.
2. Install dependencies:
   ```bash
   pip install pyodbc
   ```
3. Create and fill in `scripts/db_credentials.py` with your database info:
   ```python
   server = "<your-server>"
   database = "<your-database>"
   username = "<your-username>"
   password = "<your-password>"
   driver = "{ODBC Driver 18 for SQL Server}"
   ```

## Usage
Run the script to insert a minimal schedule and fetch its details:

```bash
python scripts/post-query.py
```

### What it does
- Inserts a new schedule, target, period, and scheduling block into the database.
- Associates them according to the schema.
- Fetches and prints the schedule, its blocks, targets, and periods.

### Example Output
```
schedule_id = 1
target_id = 1
period_id = 1
scheduling_block_id = 1
Schedule minimalista insertado correctamente.

--- SCHEDULE DATA (schedule_id=1) ---
Schedule: (1, datetime.datetime(...), 'dummy-test-checksum')
Scheduling Blocks:
(...)
Targets:
(...)
Periods:
(...)
--- END SCHEDULE DATA ---
```

## Customization
- Edit the SQL statements in `post-query.py` to insert more complex schedules or query additional data.
- Use the provided functions as templates for other database operations.

## Security
- Credentials are stored in `scripts/db_credentials.py` and ignored by git via `.gitignore`.

## Troubleshooting
- Ensure your database is accessible and the schema matches `scripts/schedule-schema-mmsql.sql`.
- Check for errors in the output for missing drivers or connection issues.

## License
See `LICENSE` in the repository.
