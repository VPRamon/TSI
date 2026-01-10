# Azure SQL - Columnas Computadas

Este documento describe todas las columnas computadas que se calculan automáticamente en Azure SQL Server, sin necesidad de que Rust las compute o inserte explícitamente.

## ¿Qué son las Columnas Computadas?

Las columnas computadas son campos que **se derivan automáticamente** de otros campos en la misma tabla mediante expresiones SQL. En Azure SQL Server, se definen con la palabra clave `AS` seguida de una expresión, y pueden tener el modificador `PERSISTED`:

- **`PERSISTED`**: El valor se calcula una vez y se almacena físicamente en disco. Más rápido en consultas.
- **Sin `PERSISTED`**: El valor se calcula en cada consulta. Ahorra espacio de almacenamiento.

---

## Tablas Base (Schema: `dbo`)

### 1. `dbo.schedule_scheduling_blocks`

Tabla de relación muchos-a-muchos entre schedules y scheduling blocks.

#### Columnas Computadas:

| Columna | Expresión SQL | Descripción |
|---------|---------------|-------------|
| `duration_sec` | `(stop_time_mjd - start_time_mjd) * 86400.0` | Duración en segundos del bloque programado, calculada desde los tiempos MJD |

**Detalles:**
```sql
duration_sec AS (
    CASE 
        WHEN start_time_mjd IS NOT NULL 
         AND stop_time_mjd IS NOT NULL 
         AND stop_time_mjd > start_time_mjd 
        THEN (stop_time_mjd - start_time_mjd) * 86400.0 
        ELSE 0 
    END
) PERSISTED
```

- **Input**: `start_time_mjd`, `stop_time_mjd`
- **Output**: Segundos de duración
- **Conversión**: 1 día MJD = 86400 segundos

---

### 2. `dbo.visibility_periods`

Almacena ventanas de visibilidad para cada combinación target + constraints.

#### Columnas Computadas:

| Columna | Expresión SQL | Descripción |
|---------|---------------|-------------|
| `duration_sec` | `(stop_time_mjd - start_time_mjd) * 86400.0` | Duración en segundos del periodo de visibilidad |

**Detalles:**
```sql
duration_sec AS (
    CASE 
        WHEN stop_time_mjd > start_time_mjd 
        THEN (stop_time_mjd - start_time_mjd) * 86400.0 
        ELSE 0 
    END
) PERSISTED
```

- **Input**: `start_time_mjd`, `stop_time_mjd`
- **Output**: Segundos de duración del periodo visible
- **Uso**: Para agregar tiempos totales de visibilidad

---

### 3. `dbo.schedule_dark_periods`

Almacena periodos oscuros (sin luz lunar) para cada schedule.

#### Columnas Computadas:

| Columna | Expresión SQL | Descripción |
|---------|---------------|-------------|
| `duration_sec` | `(stop_time_mjd - start_time_mjd) * 86400.0` | Duración en segundos del periodo oscuro |

**Detalles:**
```sql
duration_sec AS (
    CASE 
        WHEN stop_time_mjd > start_time_mjd 
        THEN (stop_time_mjd - start_time_mjd) * 86400.0 
        ELSE 0 
    END
) PERSISTED
```

- **Input**: `start_time_mjd`, `stop_time_mjd`
- **Output**: Segundos de duración del periodo oscuro
- **Uso**: Para análisis de ventanas de observación sin interferencia lunar

---

## Tablas Analíticas (Schema: `analytics`)

### 4. `analytics.schedule_blocks_analytics`

Tabla desnormalizada principal para consultas rápidas del dashboard.

#### Columnas Computadas:

| Columna | Expresión SQL | Descripción |
|---------|---------------|-------------|
| `requested_hours` | `requested_duration_sec / 3600.0` | Conversión de segundos a horas |
| `elevation_range_deg` | `max_altitude_deg - min_altitude_deg` | Rango de elevación permitido (en grados) |
| `scheduled_duration_sec` | `(scheduled_stop_mjd - scheduled_start_mjd) * 86400.0` | Duración real programada en segundos |
| `is_impossible` | `CASE WHEN total_visibility_hours = 0 THEN 1 ELSE 0 END` | Flag: 1 si el bloque no tiene visibilidad |

---

#### 4.1. `requested_hours`

**Expresión completa:**
```sql
requested_hours AS (
    CAST(requested_duration_sec AS FLOAT) / 3600.0
) PERSISTED
```

**Propósito:**
- Facilitar cálculos y visualizaciones en horas en lugar de segundos
- Usado en histogramas de distribución de duración solicitada

**Ejemplo:**
- `requested_duration_sec = 7200` → `requested_hours = 2.0`
- `requested_duration_sec = 3600` → `requested_hours = 1.0`

---

#### 4.2. `elevation_range_deg`

**Expresión completa:**
```sql
elevation_range_deg AS (
    COALESCE(max_altitude_deg, 90.0) - COALESCE(min_altitude_deg, 0.0)
) PERSISTED
```

**Propósito:**
- Calcular el rango de elevación permitido para la observación
- Si falta `max_altitude_deg`, asume 90° (cenit)
- Si falta `min_altitude_deg`, asume 0° (horizonte)

**Ejemplo:**
- `min_altitude_deg = 30`, `max_altitude_deg = 70` → `elevation_range_deg = 40`
- `min_altitude_deg = NULL`, `max_altitude_deg = NULL` → `elevation_range_deg = 90`

**Uso típico:**
- Análisis de flexibilidad en constraints de elevación
- Correlaciones con scheduling success rate

---

#### 4.3. `scheduled_duration_sec`

**Expresión completa:**
```sql
scheduled_duration_sec AS (
    CASE 
        WHEN scheduled_start_mjd IS NOT NULL 
         AND scheduled_stop_mjd IS NOT NULL 
        THEN (scheduled_stop_mjd - scheduled_start_mjd) * 86400.0 
        ELSE NULL 
    END
) PERSISTED
```

**Propósito:**
- Calcular la duración **real** del bloque programado en el schedule
- Diferente de `requested_duration_sec` (lo solicitado vs. lo programado)
- `NULL` si el bloque no fue programado (`is_scheduled = 0`)

**Ejemplo:**
- Bloque programado de MJD 59580.5 a 59580.6:
  - `scheduled_duration_sec = (0.1) * 86400 = 8640` segundos = 2.4 horas
- Bloque no programado:
  - `scheduled_duration_sec = NULL`

**Uso típico:**
- Comparar duración solicitada vs. duración real programada
- Validación: `scheduled_duration_sec >= min_observation_sec`

---

#### 4.4. `is_impossible`

**Expresión completa:**
```sql
is_impossible AS (
    CASE 
        WHEN total_visibility_hours = 0 
        THEN 1 
        ELSE 0 
    END
) PERSISTED
```

**Propósito:**
- Flag booleano rápido para identificar bloques **imposibles de programar**
- Un bloque es imposible si no tiene ninguna ventana de visibilidad

**Valores:**
- `1`: Bloque imposible (sin visibilidad)
- `0`: Bloque posible (tiene al menos alguna ventana de visibilidad)

**Uso típico:**
- Filtrar bloques imposibles en el dashboard
- Query de ejemplo: `WHERE is_impossible = 0`
- Índice optimizado: `IX_analytics_impossible`

**Relación con `validation_impossible`:**
- `is_impossible`: Calculado automáticamente de `total_visibility_hours`
- `validation_impossible`: Poblado durante ETL Phase 4 por el motor de validación Rust

---

## Resumen de Ventajas

### ✅ Beneficios de las Columnas Computadas

1. **Simplicidad en Rust**: No necesitas calcular estos valores manualmente
2. **Consistencia**: Siempre sincronizadas con los datos base
3. **Performance**: Con `PERSISTED`, son tan rápidas como columnas normales
4. **Mantenibilidad**: La lógica de cálculo está centralizada en SQL
5. **Índices**: Pueden ser indexadas para queries más rápidas

### 📊 Resumen por Tipo de Cálculo

| Tipo de Cálculo | Columnas | Propósito |
|-----------------|----------|-----------|
| **Conversión MJD → Segundos** | `duration_sec` (×3 tablas), `scheduled_duration_sec` | Convertir tiempos MJD a unidades prácticas |
| **Conversión Segundos → Horas** | `requested_hours` | Facilitar visualizaciones y análisis |
| **Rangos/Diferencias** | `elevation_range_deg` | Calcular métricas derivadas de constraints |
| **Flags Booleanos** | `is_impossible` | Clasificación rápida de bloques |

---

## Interacción con Rust

### Qué inserta Rust:

```rust
// Ejemplo: populate_schedule_analytics
insert.bind(row.scheduled_start_mjd);  // ✅ Rust inserta esto
insert.bind(row.scheduled_stop_mjd);    // ✅ Rust inserta esto
// scheduled_duration_sec NO se inserta - lo calcula Azure automáticamente
```

### Qué lee Rust desde Azure:

```rust
// Ejemplo: fetch_analytics_blocks
let scheduled_duration: Option<f64> = row.get("scheduled_duration_sec"); // ✅ Ya calculado
let requested_hours: f64 = row.get("requested_hours");                   // ✅ Ya calculado
let is_impossible: bool = row.get("is_impossible");                       // ✅ Ya calculado
```

---

## Notas de Implementación

### Constantes de Conversión:

- **1 día MJD = 86400 segundos** (24 horas × 60 min × 60 seg)
- **1 hora = 3600 segundos** (60 min × 60 seg)

### Valores por Defecto (COALESCE):

Cuando falta información de constraints:
- `min_altitude_deg` → asume `0.0°` (horizonte)
- `max_altitude_deg` → asume `90.0°` (cenit)
- `min_azimuth_deg` → asume `0.0°` (norte)
- `max_azimuth_deg` → asume `360.0°` (sin restricción)

### Performance:

Todas las columnas computadas en analytics tienen `PERSISTED`:
- ✅ Cálculo una sola vez durante INSERT
- ✅ Pueden ser indexadas
- ✅ No overhead en queries SELECT
- ⚠️ Ligero overhead en INSERT/UPDATE (despreciable)

---

## Ver Definiciones en Azure

Para inspeccionar las columnas computadas desde SQL:

```sql
-- Ver todas las columnas computadas de una tabla
SELECT 
    c.name AS column_name,
    c.is_computed,
    cc.definition AS computation,
    c.is_persisted
FROM sys.columns c
JOIN sys.computed_columns cc ON c.object_id = cc.object_id 
    AND c.column_id = cc.column_id
WHERE c.object_id = OBJECT_ID('analytics.schedule_blocks_analytics')
  AND c.is_computed = 1;
```

---

## Referencias

- **Archivo de Schema**: `backend/src/db/repositories/azure/azure-setup.sql`
- **Documentación SQL Server**: [Computed Columns](https://learn.microsoft.com/en-us/sql/relational-databases/tables/specify-computed-columns-in-a-table)
- **Módulo Rust Analytics**: `backend/src/db/repositories/azure/analytics.rs`
