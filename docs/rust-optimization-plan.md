# Plan de Optimización con Rust + PyO3

## 🔍 Análisis del Proyecto

Proyecto: **Telescope Scheduling Intelligence (TSI)**
- Dashboard Streamlit para análisis de planificadores astronómicos
- ~2,647 observaciones por dataset
- Procesamiento intensivo de datos JSON/CSV
- Parsing de periodos de visibilidad (bottleneck identificado)

## 📊 Áreas Críticas Identificadas

### 1. **ALTA PRIORIDAD: Parsing y Transformación de Datos**

#### 1.1 Procesamiento JSON → DataFrame
**Archivos afectados:**
- `src/core/preprocessing/schedule_preprocessor.py`
- `src/core/loaders/schedule_loader.py`

**Operaciones intensivas:**
```python
# Extracción de scheduling blocks (líneas 125-226 en schedule_preprocessor.py)
def _extract_scheduling_block(self, sb: dict) -> dict
def extract_dataframe(self) -> pd.DataFrame

# Procesamiento de 2,647+ bloques con anidamiento complejo
# Incluye navegación profunda en JSONs, extracción de coordenadas,
# conversión de tipos, manejo de casos especiales
```

**Beneficio esperado en Rust:**
- ⚡ **5-10x más rápido**: Parsing sin overhead del intérprete
- 📉 **Menor uso de memoria**: Sin copias intermedias de diccionarios Python
- 🔒 **Type safety**: Validación en tiempo de compilación

**Estimación:** De ~2-3s a ~0.3-0.5s para datasets grandes

---

#### 1.2 Parsing de Periodos de Visibilidad
**Archivos afectados:**
- `src/core/time/mjd.py` (líneas 33-58)
- `src/core/transformations/preparation.py` (líneas 23-50)

**Problema identificado (benchmark_visibility_strategies.py):**
```
Full parse time: ~40s para 2,647 filas
Speed: ~15ms per row
```

**Operaciones críticas:**
```python
def parse_visibility_periods(visibility_str: str | None) -> list[tuple]:
    # 1. ast.literal_eval() - parsing de strings Python (lento)
    # 2. Conversión MJD → datetime (317k conversiones)
    # 3. Creación de tuplas (pd.Timestamp, pd.Timestamp)
```

**Beneficio esperado en Rust:**
- ⚡ **10-20x más rápido**: Parser especializado sin overhead de AST
- 🎯 **De ~40s a ~2-4s** para full parse
- 💾 **Menor presión GC**: Sin objetos intermedios Python

---

#### 1.3 Conversiones MJD ↔ Datetime
**Archivo:** `src/core/time/mjd.py`

**Operaciones:**
```python
def mjd_to_datetime(mjd: float) -> pd.Timestamp
def datetime_to_mjd(dt: datetime) -> float
```

**Volumen:** 
- 2,647 bloques × ~120 conversiones cada uno = **~317,000 conversiones**
- Cada conversión crea un objeto pandas.Timestamp

**Beneficio esperado en Rust:**
- ⚡ **5-8x más rápido**: Aritmética directa sin pandas overhead
- 📦 **Integración con chrono crate**: Alta performance, battle-tested

---

### 2. **PRIORIDAD MEDIA: Algoritmos Computacionales**

#### 2.1 Análisis de Métricas y Correlaciones
**Archivo:** `src/core/algorithms/analysis.py`

**Operaciones intensivas:**
```python
def compute_metrics(df: pd.DataFrame) -> AnalyticsSnapshot
def compute_correlations(df: pd.DataFrame, columns: Sequence[str]) -> pd.DataFrame
def find_conflicts(df: pd.DataFrame) -> pd.DataFrame
def suggest_candidate_positions(df: pd.DataFrame, row: pd.Series) -> list
```

**Análisis:**
- Iteración sobre 2,647 filas en `find_conflicts` y `suggest_candidate_positions`
- Cálculos de solapamiento temporal (O(n²) en el peor caso)
- Generación de candidatos de scheduling

**Beneficio esperado en Rust:**
- ⚡ **3-5x más rápido**: Iteración nativa sin overhead Python
- 🔧 **Paralelización**: Uso de rayon crate para procesamiento paralelo
- 🧮 **SIMD optimizations**: Para operaciones vectoriales

---

#### 2.2 Optimización Greedy
**Archivo:** `src/core/algorithms/optimization.py`

**Operación:**
```python
def greedy_schedule(..., max_iterations: int = 1_000) -> OptimizationResult
```

**Análisis:**
- Loop de hasta 1,000 iteraciones
- Evaluación de constraints en cada paso
- Uso intensivo de listas Python

**Beneficio esperado en Rust:**
- ⚡ **8-15x más rápido**: Loop nativo, sin interpretación
- 🔍 **Mejor control de algoritmo**: Implementación más eficiente de constraints

---

### 3. **PRIORIDAD BAJA: Limpieza y Validación**

#### 3.1 Data Cleaning
**Archivo:** `src/core/transformations/data_cleaning.py`

Operaciones simples (remove_duplicates, validate_schema), bien optimizadas en pandas/numpy.
**Beneficio marginal en Rust**, no recomendado migrar inicialmente.

---

## 🏗️ Arquitectura Propuesta

### Opción A: Backend Completo en Rust + Frontend Streamlit (RECOMENDADO) ⭐

**Filosofía:** Rust hace TODO el trabajo pesado, Python/Streamlit solo renderiza UI.

```
TSI/
├── rust_backend/                 # Backend completo en Rust
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs               # PyO3 bindings públicos
│   │   ├── core/
│   │   │   ├── mod.rs
│   │   │   ├── domain.rs        # ScheduleBlock, Observation, etc.
│   │   │   ├── repository.rs    # Trait para data access
│   │   │   └── service.rs       # Business logic
│   │   ├── parsing/
│   │   │   ├── mod.rs
│   │   │   ├── json_parser.rs   # Deserialización JSON optimizada
│   │   │   ├── csv_parser.rs    # CSV reader con Arrow
│   │   │   └── visibility.rs    # Parsing de visibility strings
│   │   ├── preprocessing/
│   │   │   ├── mod.rs
│   │   │   ├── extractor.rs     # Extracción de scheduling blocks
│   │   │   ├── enricher.rs      # Enriquecimiento con visibility
│   │   │   └── validator.rs     # Validación de datos
│   │   ├── time/
│   │   │   ├── mod.rs
│   │   │   └── mjd.rs           # Conversiones MJD <-> DateTime
│   │   ├── algorithms/
│   │   │   ├── mod.rs
│   │   │   ├── analysis.rs      # compute_metrics, find_conflicts
│   │   │   ├── correlations.rs  # Spearman correlations
│   │   │   ├── optimization.rs  # Greedy scheduler
│   │   │   └── suggestions.rs   # suggest_candidate_positions
│   │   ├── transformations/
│   │   │   ├── mod.rs
│   │   │   ├── cleaning.rs      # Data cleaning ops
│   │   │   └── filtering.rs     # Filtrado eficiente
│   │   ├── io/
│   │   │   ├── mod.rs
│   │   │   ├── loaders.rs       # Unified data loading
│   │   │   └── exporters.rs     # CSV/Parquet export
│   │   └── python/              # PyO3 bindings layer
│   │       ├── mod.rs
│   │       ├── schedule.rs      # ScheduleService wrapper
│   │       ├── analysis.rs      # AnalysisService wrapper
│   │       └── conversions.rs   # Rust ↔ Python conversions
│   ├── benches/                 # Criterion benchmarks
│   └── tests/
│       ├── integration/
│       └── unit/
│
├── src/
│   ├── tsi_rust/                # Thin Python wrapper (auto-generated)
│   │   └── __init__.py
│   └── tsi/                     # Streamlit app (UI ONLY)
│       ├── app.py               # Entry point
│       ├── pages/               # Solo código de UI/widgets
│       │   ├── sky_map.py       # Llama rust_backend.plot_data()
│       │   ├── distributions.py
│       │   ├── insights.py
│       │   └── ...
│       ├── plots/               # Plotly wrappers (datos vienen de Rust)
│       ├── components/          # Streamlit components
│       └── state.py             # Session state management
│
├── pyproject.toml               # maturin build config
└── Cargo.toml                   # Workspace root
```

### Flujo de Datos Optimizado:

```
┌─────────────────────────────────────────────────────────────┐
│                      RUST BACKEND                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  JSON/CSV File → rust_backend::parsing                     │
│                       ↓                                      │
│               rust_backend::preprocessing                   │
│                  (extractor + enricher)                     │
│                       ↓                                      │
│               rust_backend::core::domain                    │
│                 (typed structures)                          │
│                       ↓                                      │
│          ┌────────────┴────────────┐                        │
│          ↓                         ↓                        │
│   rust_backend::algorithms   rust_backend::io              │
│   (análisis completo)        (export Arrow/Polars)         │
│          ↓                         ↓                        │
│    PyO3 bindings            Apache Arrow Table             │
│          ↓                         ↓                        │
└──────────┼─────────────────────────┼────────────────────────┘
           ↓                         ↓
┌──────────┴─────────────────────────┴────────────────────────┐
│                   PYTHON/STREAMLIT                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  import tsi_rust                                            │
│                                                             │
│  # Carga y preprocessing (ultra-rápido)                    │
│  df = tsi_rust.load_schedule("data.json").to_pandas()      │
│                                                             │
│  # Análisis (ejecutado en Rust)                            │
│  metrics = tsi_rust.compute_metrics(df)                    │
│  conflicts = tsi_rust.find_conflicts(df)                   │
│                                                             │
│  # UI rendering (Streamlit)                                │
│  st.plotly_chart(create_sky_map(df))                       │
│  st.dataframe(conflicts)                                   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Ventajas de esta Arquitectura:

1. **🔥 Máximo Performance:**
   - TODO el procesamiento pesado en Rust (parsing, análisis, transformaciones)
   - Python solo renderiza UI (lo que hace mejor)
   - Zero-copy data transfer via Arrow

2. **🧱 Separación Clara:**
   - `rust_backend/`: Lógica de negocio, tipos, algoritmos
   - `src/tsi/`: Solo código Streamlit (UI, widgets, layouts)
   - Testeo independiente de cada capa

3. **📦 Reusabilidad:**
   - Backend Rust puede usarse desde:
     - Python (Streamlit app)
     - CLI nativo Rust (scripts batch)
     - API REST (Actix/Axum en futuro)
     - Otras aplicaciones Python

4. **🚀 Escalabilidad:**
   - Backend compilado es portable (Linux, macOS, Windows)
   - Sin dependencias Python en el core
   - Fácil paralelización con Rayon

5. **🧪 Testing Robusto:**
   - Tests unitarios Rust (rápidos, type-safe)
   - Tests de integración Python
   - Benchmarks con Criterion

### Comparación con Arquitectura Híbrida Parcial:

| Aspecto | Híbrida Parcial | Backend Completo Rust |
|---------|-----------------|------------------------|
| Performance | 3-10x mejora | **10-50x mejora** |
| Mantenibilidad | Python + Rust mezclados | **Separación total** |
| Testing | Complejo (2 lenguajes) | **Simple por capas** |
| Reusabilidad | Solo desde Python | **CLI, API, Python** |
| Compile time | Rápido | Más lento (inicial) |
| Desarrollo UI | Rápido | **Rápido (solo Python)** |

---

### Opción B: Híbrida Incremental (para migración gradual)

Si prefieres migrar gradualmente:

```
TSI/
├── rust_core/                    # Módulo Rust (parcial)
│   ├── src/
│   │   ├── lib.rs               # Solo exports PyO3
│   │   ├── time.rs              # MJD conversions
│   │   └── parsing.rs           # Visibility parsing
│
├── src/
│   ├── core/                    # Mantener Python existente
│   │   ├── preprocessing/       # Usa rust_core donde conviene
│   │   └── algorithms/          # Mix Python + Rust
│   └── tsi/                     # Sin cambios
```

**Recomendación:** Empezar con Opción B (híbrida incremental), probar performance, y migrar a Opción A (backend completo) si los resultados son buenos.

---

## 📦 Dependencias Rust Clave

### Para Backend Completo (RUTA 1):

```toml
[workspace]
members = ["rust_backend"]

[dependencies]
# Python interop
pyo3 = { version = "0.20", features = ["extension-module", "abi3-py310"] }
pyo3-polars = "0.12"              # Integración Polars <-> pandas
numpy = "0.20"                    # NumPy arrays support

# Data structures & processing
polars = { version = "0.35", features = ["lazy", "temporal", "csv", "json"] }
arrow = "50.0"                    # Apache Arrow para interop
ndarray = { version = "0.15", features = ["rayon"] }  # Operaciones numéricas
csv = "1.3"                       # CSV parsing eficiente

# Serialization
serde = { version = "1.0", features = ["derive", "rc"] }
serde_json = "1.0"               # JSON parsing optimizado

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# Performance
rayon = "1.8"                    # Paralelización data-parallel
parking_lot = "0.12"             # Locks más rápidos que std
ahash = "0.8"                    # Hash function rápida

# Error handling
anyhow = "1.0"                   # Error handling ergonómico
thiserror = "1.0"                # Custom error types

# CLI (opcional)
clap = { version = "4.4", features = ["derive"] }  # Si queremos CLI Rust

# Testing & benchmarking
criterion = { version = "0.5", features = ["html_reports"] }  # Benchmarks
proptest = "1.4"                 # Property-based testing
```

### Para Migración Incremental (RUTA 2):

```toml
[dependencies]
pyo3 = { version = "0.20", features = ["extension-module"] }
chrono = "0.4"                   # MJD conversions
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"               # Visibility parsing
rayon = "1.8"                    # Paralelización básica
anyhow = "1.0"                   # Error handling

# Añadir gradualmente según fase
# pyo3-polars, arrow, ndarray, etc.
```

---

## 🔧 Plan de Implementación (Fases)

### **RUTA 1: Backend Completo en Rust** (Recomendado para proyecto nuevo)

#### **FASE 1A: Fundamentos + Domain Model (Semana 1-2)**
✅ Setup workspace Rust con estructura modular
✅ Definir tipos de dominio (`ScheduleBlock`, `Observation`, `VisibilityPeriod`)
✅ Implementar `rust_backend::time::mjd` (conversiones MJD)
✅ Implementar `rust_backend::parsing::visibility` (parser optimizado)
✅ Tests unitarios Rust + benchmarks

**Entregable:** Core domain types + parsing ultrarrápido

---

#### **FASE 1B: Parsing & Loading (Semana 3)** ✅ COMPLETADA
✅ `rust_backend::parsing::json_parser` (serde_json + custom deserializers)
✅ `rust_backend::parsing::csv_parser` (csv crate + Polars)
✅ `rust_backend::io::loaders` (unified loading interface)
✅ PyO3 bindings: `tsi_rust.load_schedule()` → Polars DataFrame

**Implementado:**
- JSON parser con soporte para estructura anidada compleja
- CSV parser con conversión directa a Polars DataFrame
- Interfaz unificada de carga (`ScheduleLoader`)
- Bindings PyO3: `load_schedule()`, `load_schedule_from_csv()`, `load_schedule_from_json()`
- Conversión automática Polars → pandas
- Columnas derivadas calculadas en Rust (scheduled_flag, requested_hours, priority_bin, etc.)

**Resultados:**
- ✅ CSV: 2647 bloques cargados correctamente
- ✅ JSON: Parser funcional (archivos pequeños/medianos)
- ✅ 23 columnas en DataFrame resultante (incluyendo derivadas)
- ✅ Tests de integración en `/tests/test_phase_1b_loaders.py`

**Entregable:** Carga completa JSON/CSV en Rust, exportable a Python

---

#### **FASE 1C: Preprocessing Pipeline (Semana 4)**
✅ `rust_backend::preprocessing::extractor` (extracción de scheduling blocks)
✅ `rust_backend::preprocessing::enricher` (cálculo de derived columns)
✅ `rust_backend::preprocessing::validator` (validaciones)
✅ Pipeline completo: JSON → ScheduleBlocks → DataFrame enriquecido

**Entregable:** Reemplazo completo de `SchedulePreprocessor` Python

---

#### **FASE 1D: Algorithms Core (Semana 5-6)**
✅ `rust_backend::algorithms::analysis` (compute_metrics, find_conflicts)
✅ `rust_backend::algorithms::correlations` (Spearman con ndarray)
✅ `rust_backend::algorithms::suggestions` (suggest_candidate_positions)
✅ `rust_backend::algorithms::optimization` (greedy scheduler)
✅ Paralelización con Rayon donde sea beneficioso

**Entregable:** Todos los algoritmos de `src/core/algorithms/` en Rust

---

#### **FASE 1E: Transformaciones & Filtering (Semana 7)**
✅ `rust_backend::transformations::filtering` (filter_dataframe optimizado)
✅ `rust_backend::transformations::cleaning` (remove_duplicates, validate_schema)
✅ Integración con Polars para operaciones vectoriales eficientes

**Entregable:** Transformaciones completas en Rust

---

#### **FASE 1F: Python Integration Layer (Semana 8)** ✅ **COMPLETADO**
✅ PyO3 wrappers completos en `rust_backend::python`
✅ Conversiones automáticas Rust ↔ Polars ↔ pandas
✅ API ergonómica con clase `TSIBackend`
✅ Documentación completa (docs/PYTHON_API.md)
✅ 10 ejemplos prácticos (examples/api_examples.py)
✅ 15 tests de integración (15/15 passing)
✅ Type hints completos para IDE support
✅ Funciones de conveniencia para uso rápido

**Entregable:** ✅ API Python completa y ergonómica con 100% tests passing

**Archivos creados:**
- `src/tsi_rust_api.py` (630 líneas)
- `docs/PYTHON_API.md` (700+ líneas)
- `examples/api_examples.py` (350+ líneas)
- `tests/test_phase_1f_integration.py` (290 líneas, 15/15 tests passing)
- `docs/FASE_1F_COMPLETADO.md` (documento de finalización)

---

#### **FASE 1G: Refactor Streamlit App (Semana 9)** 🎯 **SIGUIENTE**
🔲 Actualizar `src/tsi/` para usar `tsi_rust_api` en lugar de `src/core/`
🔲 Eliminar imports de `core.preprocessing`, `core.algorithms`, etc.
🔲 Simplificar código Python (solo UI logic)
🔲 Tests E2E actualizados

**Entregable:** App Streamlit funcionando 100% con backend Rust

---

### **RUTA 2: Migración Incremental** (Recomendado para migración gradual)

#### **FASE 2A: Fundamentos (Semana 1)**
✅ Setup del proyecto Rust con PyO3
✅ Tipos básicos y conversiones
✅ Implementar `rust_core::time::mjd` (conversiones MJD)
✅ Tests de integración Python ↔ Rust

**Entregable:** Módulo Python `tsi_rust` importable con conversiones MJD

---

#### **FASE 2B: Parsing de Visibilidad (Semana 2)**
🎯 **MÁXIMO IMPACTO: 40s → 2-4s**

✅ Implementar parser especializado de visibility strings
✅ Reemplazar `parse_visibility_periods()` en preparation.py
✅ Benchmarks comparativos

**Entregable:** Parsing de visibilidad 10-20x más rápido

---

#### **FASE 2C: Extracción JSON (Semana 3)**
✅ Parser estructurado para JSON de schedules
✅ Integración con PyArrow para output
✅ Reemplazar `_extract_scheduling_block()` y `extract_dataframe()`

**Entregable:** Carga de JSON 5-10x más rápida

---

#### **FASE 2D: Algoritmos (Semana 4)**
✅ `compute_metrics()` en Rust
✅ `find_conflicts()` optimizado
✅ `suggest_candidate_positions()` con rayon (paralelo)

**Entregable:** Análisis de insights 3-5x más rápido

---

#### **FASE 2E: Completar Backend (Semana 5-6)**
✅ Migrar preprocessing completo
✅ Migrar transformations
✅ Consolidar en arquitectura backend completo

**Entregable:** Transición a arquitectura de backend completo Rust

---

### **Comparación de Rutas:**

| Aspecto | RUTA 1 (Backend Completo) | RUTA 2 (Incremental) |
|---------|---------------------------|----------------------|
| **Tiempo total** | 9 semanas | 4-6 semanas |
| **Riesgo** | Medio (refactor grande) | Bajo (cambios pequeños) |
| **Performance final** | **Óptimo (10-50x)** | Bueno (5-20x) |
| **Mantenibilidad** | **Excelente** | Buena → Excelente |
| **Testabilidad** | **Excelente** | Buena |
| **Recomendado para** | Proyecto con tiempo | **Migración rápida** |

---

### **🎯 Recomendación Estratégica:**

**EMPEZAR CON RUTA 2 (Incremental):**
1. ✅ **Semanas 1-2:** FASE 2A + 2B (MJD + Visibility) → Probar impacto real
2. ✅ **Evaluar resultados:** Si speedup es notable, continuar
3. ✅ **Semanas 3-4:** FASE 2C + 2D (JSON + Algoritmos)
4. 🔄 **Decisión:** Si todo funciona bien, migrar a RUTA 1 (backend completo)

**Razón:** Validar hipótesis de performance con inversión mínima antes de commit completo.

---

## 📈 Impacto Esperado Total

| Operación | Tiempo Actual | Tiempo Rust | Speedup |
|-----------|---------------|-------------|---------|
| Carga CSV + prepare | 0.5-1.0s | 0.2-0.3s | **2-3x** |
| Parse visibilidad (50 rows) | 0.75s | 0.05-0.1s | **7-15x** |
| Parse visibilidad (full) | 40s | 2-4s | **10-20x** |
| Extracción JSON | 2-3s | 0.3-0.5s | **5-10x** |
| Análisis de insights | 0.5-1.0s | 0.1-0.2s | **3-5x** |

**Mejora percibida por el usuario:**
- ⚡ Landing page: **~1s → ~0.3s** (sentirse instantáneo)
- 📊 Timeline (50 rows): **~0.75s → ~0.1s** (interactividad fluida)
- 🔍 Análisis completo: **~40s → ~4s** (tolerable sin cache)

---

## 🚀 Comandos de Setup Rápido

### Setup para Backend Completo (RUTA 1):

#### 1. Instalar Rust y herramientas
```bash
# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Instalar maturin
pip install maturin

# Verificar instalación
rustc --version
cargo --version
python -m pip show maturin
```

#### 2. Crear estructura del proyecto
```bash
cd /home/ramon/workspace/TSI

# Crear workspace Rust
mkdir -p rust_backend/src/{core,parsing,preprocessing,time,algorithms,transformations,io,python}
mkdir -p rust_backend/{tests,benches}

# Crear Cargo.toml principal
cat > Cargo.toml << 'EOF'
[workspace]
members = ["rust_backend"]
resolver = "2"
EOF

# Crear Cargo.toml del backend
cat > rust_backend/Cargo.toml << 'EOF'
[package]
name = "tsi-rust"
version = "0.1.0"
edition = "2021"

[lib]
name = "tsi_rust"
crate-type = ["cdylib"]  # Para PyO3

[[bin]]
name = "tsi-cli"
path = "src/bin/cli.rs"
required-features = ["cli"]

[dependencies]
pyo3 = { version = "0.20", features = ["extension-module", "abi3-py310"] }
pyo3-polars = "0.12"
polars = { version = "0.35", features = ["lazy", "temporal", "csv", "json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
rayon = "1.8"
anyhow = "1.0"
thiserror = "1.0"

# Opcional: CLI
clap = { version = "4.4", features = ["derive"], optional = true }

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"

[features]
default = []
cli = ["dep:clap"]

[[bench]]
name = "parsing_benchmark"
harness = false
EOF
```

#### 3. Configurar pyproject.toml
```bash
# Añadir configuración maturin
cat >> pyproject.toml << 'EOF'

[tool.maturin]
module-name = "tsi_rust"
bindings = "pyo3"
compatibility = "linux"
features = ["pyo3/extension-module"]

# Build profile
[profile.release]
lto = true              # Link-time optimization
codegen-units = 1       # Mejor optimización
opt-level = 3           # Máxima optimización
strip = true            # Reducir tamaño binario
EOF
```

#### 4. Crear estructura inicial de módulos
```bash
# lib.rs (entry point PyO3)
cat > rust_backend/src/lib.rs << 'EOF'
use pyo3::prelude::*;

pub mod core;
pub mod parsing;
pub mod preprocessing;
pub mod time;
pub mod algorithms;
pub mod transformations;
pub mod io;
pub mod python;

#[pymodule]
fn tsi_rust(_py: Python, m: &PyModule) -> PyResult<()> {
    // Registrar módulos
    m.add_class::<python::schedule::ScheduleLoader>()?;
    m.add_class::<python::analysis::AnalysisService>()?;
    
    // Registrar funciones
    m.add_function(wrap_pyfunction!(time::mjd_to_datetime, m)?)?;
    m.add_function(wrap_pyfunction!(time::datetime_to_mjd, m)?)?;
    
    Ok(())
}
EOF

# Crear mod.rs para cada módulo
for dir in core parsing preprocessing time algorithms transformations io python; do
    echo "pub mod $dir;" > rust_backend/src/$dir/mod.rs
done
```

#### 5. Compilar e instalar
```bash
cd /home/ramon/workspace/TSI

# Development build (rápido, para testing)
maturin develop

# Release build (optimizado, para producción)
maturin develop --release

# Verificar instalación
python -c "import tsi_rust; print('✅ tsi_rust importado correctamente')"
```

#### 6. Probar integración básica
```python
# test_rust_integration.py
import tsi_rust
import pandas as pd

# Test 1: Conversión MJD
mjd = 59580.5
dt = tsi_rust.mjd_to_datetime(mjd)
print(f"✅ MJD {mjd} → {dt}")

# Test 2: Cargar schedule (cuando esté implementado)
# df = tsi_rust.load_schedule("data/schedule.json").to_pandas()
# print(f"✅ Loaded {len(df)} scheduling blocks")
```

---

### Setup para Migración Incremental (RUTA 2):

#### Setup más simple y rápido
```bash
cd /home/ramon/workspace/TSI

# Crear proyecto con maturin
maturin new --bindings pyo3 rust_core
cd rust_core

# Configurar Cargo.toml mínimo
cat > Cargo.toml << 'EOF'
[package]
name = "tsi-rust"
version = "0.1.0"
edition = "2021"

[lib]
name = "tsi_rust"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.20", features = ["extension-module"] }
chrono = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
EOF

# Compilar
maturin develop --release

# Probar
python -c "import tsi_rust; print('✅ OK')"
```

---

## 🧪 Estrategia de Testing

### Tests Rust (nativos)
```bash
cargo test --release
```

### Tests de integración Python
```python
# tests/rust_integration/test_time_conversions.py
import pytest
import tsi_rust
from datetime import datetime

def test_mjd_roundtrip():
    original_mjd = 59580.5
    dt = tsi_rust.mjd_to_datetime(original_mjd)
    back_to_mjd = tsi_rust.datetime_to_mjd(dt)
    assert abs(original_mjd - back_to_mjd) < 1e-6
```

### Benchmarks comparativos
```python
# tests/benchmarks/benchmark_rust_vs_python.py
import time
import tsi_rust
from core.time import mjd as python_mjd

def benchmark_conversions(n=100_000):
    mjd_values = [59580.5 + i for i in range(n)]
    
    # Python
    start = time.time()
    [python_mjd.mjd_to_datetime(m) for m in mjd_values]
    python_time = time.time() - start
    
    # Rust
    start = time.time()
    [tsi_rust.mjd_to_datetime(m) for m in mjd_values]
    rust_time = time.time() - start
    
    print(f"Python: {python_time:.3f}s")
    print(f"Rust:   {rust_time:.3f}s")
    print(f"Speedup: {python_time/rust_time:.1f}x")
```

---

## 🎯 Recomendación Final

### Para Máxima Performance y Arquitectura Limpia:

**EMPEZAR CON RUTA 2 (Incremental), MIGRAR A RUTA 1 (Backend Completo):**

#### Paso 1: Validación (Semanas 1-2) ✅
1. ✅ **FASE 2A + 2B** (MJD + Visibility parsing)
   - Máximo impacto (10-20x speedup)
   - Riesgo bajo, alcance limitado
   - **Objetivo:** Validar que Rust + PyO3 funciona en tu entorno

**Criterios de éxito:**
- ⚡ Parsing visibilidad: 40s → 2-4s (10-20x speedup)
- 🧪 Tests pasan sin regresiones
- 🔄 Integración suave con código Python existente

#### Paso 2: Expansión (Semanas 3-4) ✅
2. ✅ **FASE 2C + 2D** (JSON parsing + Algoritmos)
   - Speedup acumulado: 5-20x en operaciones comunes
   - **Objetivo:** Probar que Rust maneja complejidad del dominio

**Criterios de éxito:**
- 📊 Carga JSON: 2-3s → 0.3-0.5s (5-10x speedup)
- 🔍 Análisis insights: 0.5-1s → 0.1-0.2s (3-5x speedup)

#### Paso 3: Backend Completo (Semanas 5-9) 🚀
3. 🚀 **MIGRAR A RUTA 1** (Backend completo)
   - Refactorizar a arquitectura limpia
   - Separación total: Rust (backend) + Python (UI)
   - **Objetivo:** Sistema mantenible y escalable

**Beneficios finales:**
- ⚡ **10-50x speedup** en operaciones críticas
- 🧱 **Código más limpio:** Backend type-safe, UI simple
- 🔧 **Reusabilidad:** Backend Rust portable (CLI, API, Python)
- 🧪 **Testing robusto:** Cada capa independiente
- 📦 **Escalabilidad:** Paralelización, SIMD, zero-copy

---

### Alternativa: Solo Migración Incremental

Si prefieres quedarte con arquitectura híbrida (Python + Rust):

**EMPEZAR POR:**
1. ✅ **FASE 2A + 2B** (MJD + Visibility parsing)
   - 2 semanas de desarrollo
   - Mejora dramática en UX

**VENTAJAS:**
- ⚡ Mejora notable con inversión mínima
- 🔧 No requiere reestructuración grande
- 🧪 Fácil de testear y validar
- 🔄 Rollback simple (mantener Python como fallback)

**SI FUNCIONA BIEN:**
- Continuar con FASE 2C (JSON parsing)
- Luego FASE 2D (algoritmos)
- **NO migrar** a backend completo (arquitectura estable)

---

### Matriz de Decisión:

| Objetivo | Ruta Recomendada | Tiempo | ROI |
|----------|------------------|--------|-----|
| **Mejora rápida, bajo riesgo** | RUTA 2 (solo 2A+2B) | 2 semanas | ⭐⭐⭐⭐⭐ |
| **Buen performance, estable** | RUTA 2 completa | 4-6 semanas | ⭐⭐⭐⭐ |
| **Máximo performance, escalable** | RUTA 2 → RUTA 1 | 9 semanas | ⭐⭐⭐⭐⭐ |
| **Arquitectura limpia, reusable** | Directo RUTA 1 | 9 semanas | ⭐⭐⭐⭐ |

**Mi recomendación:** 🎯 **RUTA 2 (2A+2B) → Evaluar → RUTA 1 completa**
- Validación rápida con impacto inmediato
- Decisión informada basada en resultados reales
- Máximo beneficio a largo plazo

---

## 📚 Recursos

- [PyO3 User Guide](https://pyo3.rs/)
- [Maturin Documentation](https://www.maturin.rs/)
- [Arrow Rust](https://docs.rs/arrow/latest/arrow/)
- [Polars ↔ Pandas interop](https://pola-rs.github.io/polars/py-polars/html/reference/api.html)

---

**¿Quieres que empiece a implementar la FASE 1 + FASE 2?**
