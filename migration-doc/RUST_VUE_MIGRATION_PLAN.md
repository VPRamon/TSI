# Complete Migration Plan: Streamlit → Rust/Vue

**Goal**: Transform the Telescope Scheduling Intelligence Streamlit app into a modern, production-ready application with a Rust backend (Axum + Polars) and Vue 3 TypeScript frontend.

**Current Status**: 
- ✅ Basic scaffolding exists (Rust backend with demo endpoints, Vue frontend seed)
- ❌ No real business logic ported yet
- ❌ Frontend is a minimal demo (mean/std calculator)

---

## 🎯 Architecture Overview

### Target Stack
- **Backend**: Rust (Axum web framework, Polars for data processing, Serde for JSON)
- **Frontend**: Vue 3 (TypeScript, Vite, Plotly.js for charts, Tailwind CSS)
- **Data Flow**: REST API + Server-Sent Events (SSE) for progress updates
- **Deployment**: Docker Compose (backend + frontend), production-ready

### Key Benefits Over Streamlit
- 🚀 **Performance**: 10-100x faster data processing with Rust + Polars
- 📦 **Scalability**: RESTful API enables multiple concurrent users
- 🎨 **UX**: Modern, responsive UI with real-time updates (SSE)
- 🔧 **Maintainability**: Clear separation of concerns (backend/frontend)
- 📊 **Type Safety**: Full TypeScript + Rust type checking

---

## 📋 Features to Migrate

Based on the existing Streamlit app (`src/tsi/`), here are the 7 main pages and their dependencies:

### **1. Landing Page** (Data Upload & Loading)
- Upload CSV (preprocessed schedules)
- Upload JSON (raw schedule + optional visibility/dark periods)
- Load sample dataset
- In-memory JSON preprocessing
- Dark periods auto-detection

### **2. Sky Map** 🌌
- RA/Dec scatter plot (Plotly)
- Color by: priority bins, scheduling status, time filters
- Size by: requested hours
- Filters: priority range, time range, scheduled/unscheduled
- Interactive tooltips with observation details

### **3. Distributions** 📊
- Histograms: priority, visibility hours, requested duration, elevation range
- Summary statistics (mean, median, std, quartiles)
- Bin customization
- Export data

### **4. Visibility Map** 🗺️
- Visualize visibility windows and constraints
- Azimuth/Elevation ranges
- Temporal availability

### **5. Scheduled Timeline** 📅
- Month-by-month view of scheduled observations
- Dark/daytime period overlays
- CSV export
- Interactive filtering

### **6. Insights** 💡
- Automated analytics:
  - Scheduling rate, utilization
  - Conflict detection (impossible observations)
  - Correlation analysis (priority vs visibility, etc.)
  - Top observations by priority/hours
- Downloadable reports (HTML, Markdown)

### **7. Trends** 📈
- Time evolution of scheduling metrics
- Month-over-month comparisons
- Priority distribution over time

### **8. Compare Schedules** ⚖️
- Load second CSV
- Side-by-side comparison
- Diff visualization

---

## 🏗️ Migration Phases

### **Phase 0: Foundation** (Current → Week 1)
**Goal**: Set up robust data layer and core API contracts

#### Tasks:
1. **Data Schema & Types**
   - [ ] Define Rust structs matching the CSV schema (`SchedulingBlock`, `VisibilityPeriod`, etc.)
   - [ ] Implement Serde serialization/deserialization
   - [ ] Add validation (required columns, types)
   - **Files**: `backend/src/models/schedule.rs`, `backend/src/models/mod.rs`
   - **Effort**: 4-6 hours

2. **Data Loading**
   - [ ] CSV parser using Polars (lazy loading for large files)
   - [ ] JSON parser (raw schedule + visibility)
   - [ ] In-memory preprocessing (port `src/core/preprocessing/schedule_preprocessor.py`)
   - [ ] Dark periods loader
   - **Files**: `backend/src/loaders/mod.rs`, `backend/src/loaders/csv.rs`, `backend/src/loaders/json.rs`
   - **Effort**: 8-12 hours

3. **API Contracts**
   - [ ] Define request/response types for all endpoints
   - [ ] OpenAPI spec generation (using `utoipa`)
   - **Files**: `backend/src/models/api.rs`
   - **Effort**: 3-4 hours

4. **State Management**
   - [ ] In-memory data store (Arc<RwLock<AppState>>)
   - [ ] Session management (optional: if multi-user)
   - **Files**: `backend/src/state.rs`
   - **Effort**: 3-4 hours

**Deliverable**: Backend can load CSV/JSON, store in memory, expose `/api/v1/datasets` endpoints

---

### **Phase 1: Data Upload & Core Endpoints** (Week 1-2)
**Goal**: Landing page functionality + dataset management

#### Backend Tasks:
1. **Upload Endpoints**
   - [ ] `POST /api/v1/datasets/upload/csv` (multipart form)
   - [ ] `POST /api/v1/datasets/upload/json` (raw schedule + optional visibility)
   - [ ] `GET /api/v1/datasets/sample` (load bundled sample)
   - [ ] `GET /api/v1/datasets/current` (get loaded dataset metadata)
   - [ ] `DELETE /api/v1/datasets/current` (clear data)
   - **Files**: `backend/src/routes/datasets.rs`
   - **Effort**: 8-10 hours

2. **Preprocessing Pipeline**
   - [ ] Port `SchedulePreprocessor` logic to Rust
   - [ ] Visibility calculations
   - [ ] Derived columns (scheduled_flag, requested_hours, elevation_range_deg, priority_bin)
   - **Files**: `backend/src/preprocessing/mod.rs`
   - **Effort**: 12-16 hours (complex logic)

3. **Progress SSE**
   - [ ] Enhance `/api/v1/progress` to stream real preprocessing progress
   - **Files**: `backend/src/routes/progress.rs`
   - **Effort**: 2-3 hours

#### Frontend Tasks:
1. **Landing Page UI**
   - [ ] File upload component (CSV + JSON)
   - [ ] Drag & drop support
   - [ ] Sample data loader button
   - [ ] Progress bar (SSE)
   - [ ] Error handling & validation feedback
   - **Files**: `frontend/src/components/LandingPage.vue`, `frontend/src/components/FileUpload.vue`
   - **Effort**: 8-10 hours

2. **Navigation**
   - [ ] Top navigation bar (7 pages)
   - [ ] Dataset title display
   - [ ] Routing (Vue Router)
   - **Files**: `frontend/src/components/Navigation.vue`, `frontend/src/router.ts`
   - **Effort**: 3-4 hours

**Deliverable**: Users can upload CSV/JSON, see preprocessing progress, navigate to pages

---

### **Phase 2: Analytics Backend** (Week 2-3)
**Goal**: Port all core analytics algorithms to Rust

#### Tasks:
1. **Metrics Computation**
   - [ ] Port `compute_metrics` (total blocks, scheduled, utilization, etc.)
   - [ ] Statistical summaries (mean, median, std)
   - **Files**: `backend/src/analytics/metrics.rs`
   - **Effort**: 6-8 hours

2. **Correlation Analysis**
   - [ ] Spearman correlation matrix (Polars)
   - [ ] Correlation insights generation
   - **Files**: `backend/src/analytics/correlations.rs`
   - **Effort**: 4-6 hours

3. **Conflict Detection**
   - [ ] Find impossible observations (visibility < requested time)
   - [ ] Scheduling integrity checks
   - **Files**: `backend/src/analytics/conflicts.rs`
   - **Effort**: 3-4 hours

4. **Top Observations**
   - [ ] Sort and filter by priority, hours, etc.
   - **Files**: `backend/src/analytics/top_observations.rs`
   - **Effort**: 2-3 hours

5. **Distribution Stats**
   - [ ] Histogram binning
   - [ ] Quartiles, percentiles
   - **Files**: `backend/src/analytics/distributions.rs`
   - **Effort**: 4-5 hours

6. **API Endpoints**
   - [ ] `GET /api/v1/analytics/metrics` (overall metrics)
   - [ ] `GET /api/v1/analytics/correlations?columns=priority,visibility_hours`
   - [ ] `GET /api/v1/analytics/conflicts`
   - [ ] `GET /api/v1/analytics/top?by=priority&n=10`
   - [ ] `GET /api/v1/analytics/distribution?column=priority&bins=20`
   - **Files**: `backend/src/routes/analytics.rs`
   - **Effort**: 6-8 hours

**Deliverable**: Backend exposes all analytics APIs, tested with curl/Postman

---

### **Phase 3: Visualization Pages (Part 1)** (Week 3-4)
**Goal**: Sky Map, Distributions, Insights

#### Sky Map 🌌
1. **Backend**
   - [ ] `GET /api/v1/visualizations/sky-map?filters=...` (filtered RA/Dec data)
   - [ ] Filter support (priority range, time range, scheduled/unscheduled)
   - **Files**: `backend/src/routes/visualizations.rs`
   - **Effort**: 5-6 hours

2. **Frontend**
   - [ ] Plotly scatter plot (RA vs Dec)
   - [ ] Filter controls (sidebar: priority slider, date picker, radio buttons)
   - [ ] Color mapping (priority bins, status)
   - [ ] Size mapping (requested hours)
   - [ ] Interactive tooltips
   - **Files**: `frontend/src/pages/SkyMap.vue`, `frontend/src/components/SkyMapPlot.vue`
   - **Effort**: 10-12 hours

#### Distributions 📊
1. **Backend**
   - [ ] Already covered in Phase 2 analytics APIs
   - **Effort**: 0 hours (reuse)

2. **Frontend**
   - [ ] Multiple histogram charts (priority, visibility, duration, elevation)
   - [ ] Summary statistics cards
   - [ ] Bin size controls
   - [ ] Export button (CSV)
   - **Files**: `frontend/src/pages/Distributions.vue`, `frontend/src/components/HistogramChart.vue`
   - **Effort**: 8-10 hours

#### Insights 💡
1. **Backend**
   - [ ] `GET /api/v1/analytics/insights` (automated text insights)
   - [ ] `GET /api/v1/reports/markdown` (download)
   - [ ] `GET /api/v1/reports/html` (download)
   - **Files**: `backend/src/analytics/insights.rs`, `backend/src/routes/reports.rs`
   - **Effort**: 6-8 hours

2. **Frontend**
   - [ ] Metrics dashboard (cards with key stats)
   - [ ] Correlation heatmap
   - [ ] Insights list (automated bullet points)
   - [ ] Conflicts table
   - [ ] Top observations table
   - [ ] Download reports buttons
   - **Files**: `frontend/src/pages/Insights.vue`, `frontend/src/components/HeatmapChart.vue`
   - **Effort**: 10-12 hours

**Deliverable**: 3 fully functional pages with real data and charts

---

### **Phase 4: Visualization Pages (Part 2)** (Week 4-5)
**Goal**: Visibility Map, Scheduled Timeline, Trends

#### Visibility Map 🗺️
1. **Backend**
   - [ ] `GET /api/v1/visualizations/visibility-map?block_id=...` (visibility windows)
   - **Files**: `backend/src/routes/visualizations.rs`
   - **Effort**: 5-6 hours

2. **Frontend**
   - [ ] Gantt-style chart (visibility periods)
   - [ ] Azimuth/Elevation constraint visualization
   - [ ] Block selector dropdown
   - **Files**: `frontend/src/pages/VisibilityMap.vue`
   - **Effort**: 8-10 hours

#### Scheduled Timeline 📅
1. **Backend**
   - [ ] `GET /api/v1/visualizations/timeline?month=...` (scheduled observations + dark periods)
   - [ ] `GET /api/v1/export/timeline?format=csv` (streaming CSV download)
   - **Files**: `backend/src/routes/visualizations.rs`, `backend/src/routes/export.rs`
   - **Effort**: 6-8 hours

2. **Frontend**
   - [ ] Month-by-month timeline chart (Plotly)
   - [ ] Dark/daytime overlays
   - [ ] Month selector
   - [ ] Export CSV button
   - **Files**: `frontend/src/pages/ScheduledTimeline.vue`
   - **Effort**: 8-10 hours

#### Trends 📈
1. **Backend**
   - [ ] `GET /api/v1/analytics/trends?metric=...&group_by=month` (time series data)
   - **Files**: `backend/src/routes/analytics.rs`
   - **Effort**: 4-5 hours

2. **Frontend**
   - [ ] Line charts (scheduling rate, utilization over time)
   - [ ] Metric selector dropdown
   - [ ] Grouping controls (day/week/month)
   - **Files**: `frontend/src/pages/Trends.vue`
   - **Effort**: 6-8 hours

**Deliverable**: 3 more pages functional, 6/7 pages complete

---

### **Phase 5: Compare & Polish** (Week 5-6)
**Goal**: Finish all features, polish UX, testing

#### Compare Schedules ⚖️
1. **Backend**
   - [ ] Support multiple datasets in state (primary + comparison)
   - [ ] `POST /api/v1/datasets/comparison/upload`
   - [ ] `GET /api/v1/analytics/compare` (diff metrics)
   - **Files**: `backend/src/state.rs`, `backend/src/routes/comparison.rs`
   - **Effort**: 6-8 hours

2. **Frontend**
   - [ ] Upload second dataset button
   - [ ] Side-by-side metrics comparison
   - [ ] Diff visualization (added/removed/changed observations)
   - **Files**: `frontend/src/pages/CompareSchedules.vue`
   - **Effort**: 8-10 hours

#### Polish & UX
1. **Frontend**
   - [ ] Loading states and skeletons
   - [ ] Error handling & user-friendly messages
   - [ ] Responsive design (mobile-friendly)
   - [ ] Dark mode toggle (optional)
   - [ ] Accessibility (ARIA labels, keyboard navigation)
   - **Effort**: 8-12 hours

2. **Backend**
   - [ ] Error handling middleware (structured JSON errors)
   - [ ] Request validation
   - [ ] Rate limiting (optional)
   - [ ] Logging and tracing enhancements
   - **Effort**: 4-6 hours

3. **Testing**
   - [ ] Rust unit tests for all analytics functions
   - [ ] Integration tests for API endpoints
   - [ ] Frontend E2E tests (Playwright or Cypress)
   - **Effort**: 12-16 hours

4. **Documentation**
   - [ ] API documentation (OpenAPI/Swagger UI)
   - [ ] README updates (run instructions, architecture)
   - [ ] User guide
   - **Effort**: 4-6 hours

**Deliverable**: Fully functional, polished, tested application

---

### **Phase 6: Deployment & Production** (Week 6)
**Goal**: Production-ready deployment

#### Tasks:
1. **Docker & Compose**
   - [ ] Optimize Dockerfiles (multi-stage builds)
   - [ ] Docker Compose for local development
   - [ ] Production docker-compose.yml
   - **Effort**: 3-4 hours

2. **CI/CD**
   - [ ] GitHub Actions (or GitLab CI)
   - [ ] Automated tests on PR
   - [ ] Docker image builds and pushes
   - **Effort**: 4-5 hours

3. **Deployment**
   - [ ] Cloud deployment (AWS, GCP, Azure, or DigitalOcean)
   - [ ] HTTPS setup (Let's Encrypt)
   - [ ] Domain configuration
   - **Effort**: 4-6 hours

4. **Monitoring**
   - [ ] Logging aggregation (optional: ELK, Loki)
   - [ ] Metrics (Prometheus + Grafana, optional)
   - [ ] Health checks
   - **Effort**: 3-4 hours (optional, can defer)

**Deliverable**: App running in production, accessible via URL

---

## 📊 Summary & Timeline

### Total Effort Estimate
| Phase | Backend | Frontend | Testing/Docs | Total Hours |
|-------|---------|----------|--------------|-------------|
| 0: Foundation | 18-26h | 0h | 0h | **18-26h** |
| 1: Upload & Core | 22-29h | 11-14h | 2-3h | **35-46h** |
| 2: Analytics Backend | 25-34h | 0h | 4-6h | **29-40h** |
| 3: Viz Part 1 | 11-14h | 28-34h | 4-6h | **43-54h** |
| 4: Viz Part 2 | 15-19h | 22-28h | 4-6h | **41-53h** |
| 5: Compare & Polish | 18-26h | 16-22h | 20-28h | **54-76h** |
| 6: Deployment | 14-19h | 0h | 0h | **14-19h** |
| **TOTAL** | **123-167h** | **77-98h** | **34-49h** | **234-314h** |

### Timeline (Conservative)
- **Solo developer, part-time (20h/week)**: 12-16 weeks (~3-4 months)
- **Solo developer, full-time (40h/week)**: 6-8 weeks (~1.5-2 months)
- **Team of 2 (1 backend, 1 frontend)**: 4-6 weeks (~1-1.5 months)

### Quick Wins for Early Demos
If you need to show progress quickly, prioritize:
1. **Week 1**: Phase 0 + Phase 1 (data upload working)
2. **Week 2**: Phase 2 + Sky Map (first visualization)
3. **Week 3**: Distributions + Insights (analytics demo)

---

## 🛠️ Technical Decisions & Recommendations

### Backend Architecture
```
backend/src/
├── main.rs                 # Server setup
├── lib.rs                  # Library re-exports
├── models/
│   ├── mod.rs
│   ├── schedule.rs         # Core data types
│   └── api.rs              # Request/response types
├── loaders/
│   ├── mod.rs
│   ├── csv.rs              # Polars CSV loader
│   └── json.rs             # JSON + preprocessing
├── preprocessing/
│   ├── mod.rs
│   └── schedule.rs         # Port of Python preprocessor
├── analytics/
│   ├── mod.rs
│   ├── metrics.rs
│   ├── correlations.rs
│   ├── conflicts.rs
│   ├── distributions.rs
│   └── insights.rs
├── routes/
│   ├── mod.rs
│   ├── health.rs
│   ├── datasets.rs         # Upload, load, delete
│   ├── analytics.rs        # Metrics, correlations, etc.
│   ├── visualizations.rs   # Chart data endpoints
│   ├── reports.rs          # HTML/Markdown downloads
│   └── comparison.rs       # Compare schedules
├── state.rs                # In-memory data store
└── utils.rs                # Helpers, MJD conversion, etc.
```

### Frontend Architecture
```
frontend/src/
├── main.ts                 # Vue app entry
├── App.vue                 # Root component
├── router.ts               # Vue Router config
├── api/
│   └── client.ts           # Axios API client
├── composables/
│   ├── useDatasets.ts      # Dataset management
│   └── useAnalytics.ts     # Analytics hooks
├── components/
│   ├── Navigation.vue
│   ├── FileUpload.vue
│   ├── DataTable.vue
│   ├── ProgressBar.vue
│   └── charts/
│       ├── ScatterPlot.vue
│       ├── Histogram.vue
│       ├── Heatmap.vue
│       └── Timeline.vue
└── pages/
    ├── LandingPage.vue
    ├── SkyMap.vue
    ├── Distributions.vue
    ├── VisibilityMap.vue
    ├── ScheduledTimeline.vue
    ├── Insights.vue
    ├── Trends.vue
    └── CompareSchedules.vue
```

### Key Libraries
**Backend (Cargo.toml additions)**:
```toml
polars = { version = "0.34", features = ["lazy", "csv-file", "parquet", "json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
axum = { version = "0.7", features = ["multipart", "ws"] }
tower-http = { version = "0.5", features = ["cors", "trace", "fs"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
thiserror = "1.0"
utoipa = { version = "4.0", features = ["axum_extras"] }
utoipa-swagger-ui = "6.0"
```

**Frontend (package.json additions)**:
```json
{
  "dependencies": {
    "vue": "^3.3",
    "vue-router": "^4.2",
    "axios": "^1.5",
    "plotly.js-dist-min": "^2.26",
    "@tailwindcss/forms": "^0.5"
  },
  "devDependencies": {
    "@vitejs/plugin-vue": "^4.4",
    "typescript": "^5.2",
    "vite": "^4.5",
    "tailwindcss": "^3.3"
  }
}
```

---

## 🚀 Getting Started (Implementation Order)

### Step 1: Start with Phase 0 Foundation
I recommend starting immediately with:
1. Define Rust data models (`backend/src/models/schedule.rs`)
2. Implement CSV loader with Polars
3. Add basic state management
4. Create OpenAPI spec skeleton

**Command to run**:
```bash
cd backend
cargo add polars --features lazy,csv-file,json
cargo add serde --features derive
cargo add serde_json
cargo add utoipa --features axum_extras
cargo add utoipa-swagger-ui
```

### Step 2: Parallel Frontend Setup
While backend foundation is being built:
1. Set up Vue Router
2. Create page components (empty shells)
3. Build file upload UI
4. Connect to existing `/health` endpoint

**Command to run**:
```bash
cd frontend
npm install vue-router axios plotly.js-dist-min @tailwindcss/forms
```

### Step 3: Iterate Phase by Phase
- Complete each phase fully before moving to next
- Test each API endpoint with curl/Postman before building frontend
- Build frontend page-by-page after backend endpoint is ready

---

## 📝 Next Steps (What I Can Do Now)

I can help you start implementing immediately. Choose one:

### Option A: Start Phase 0 (Recommended)
I'll create:
1. Rust data models matching your CSV schema
2. Polars CSV loader
3. State management structure
4. Initial API contracts

### Option B: Start with a Single Feature (Quick Demo)
Pick one page to fully implement end-to-end:
- Sky Map (most visually impressive)
- Insights (most useful for users)
- Distributions (simplest to start)

### Option C: Review & Refine Plan
We can:
- Adjust priorities
- Add/remove features
- Clarify technical questions

**What would you like me to implement first?**
