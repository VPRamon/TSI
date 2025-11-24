# Sky Map View - Bug Fixes and Enhancements

## Date
November 14, 2025

## Issues Identified

### 1. **Critical: Tailwind CSS Not Configured**
**Problem:** Tailwind CSS was listed as a dependency in `package.json` but was completely unconfigured. The application had no `tailwind.config.js`, `postcss.config.js`, or Tailwind directive imports.

**Impact:** All Tailwind utility classes used throughout the application (especially in SkyMap.vue) were not being processed, resulting in completely unstyled components. Classes like `grid-cols-1`, `md:grid-cols-3`, `bg-white`, `p-6`, `rounded-lg`, etc., were rendering as plain text in the DOM.

**Root Cause:** The project had Tailwind as a dependency but never completed the setup process.

### 2. **Missing Target Information in UI**
**Problem:** The "sky target" feature (target name and ID) was present in the raw JSON data but was being stripped out during backend processing and never displayed in the frontend UI.

**Impact:** Users couldn't see which astronomical target each observation corresponded to, making the sky map less useful for scientific analysis.

**Root Cause:** 
- Backend: The JSON loader extracted position data from the target object but ignored the `name` and `id_` fields
- Frontend: The TypeScript interface didn't include target fields, and the tooltip didn't display them

### 3. **Property Name Mismatches**
**Problem:** The frontend was using snake_case property names (`scheduled_flag`, `right_ascension_deg`, etc.) while the backend was serializing to camelCase (`scheduledFlag`, `raInDeg`, etc.) due to the `#[serde(rename_all = "camelCase")]` attribute.

**Impact:** Data binding failures, empty charts, and undefined values in tooltips.

## Fixes Applied

### Frontend Fixes

#### 1. Tailwind CSS Configuration
Created three new configuration files:

**`frontend/tailwind.config.js`:**
```javascript
export default {
  content: [
    "./index.html",
    "./src/**/*.{vue,js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        primary: { /* color palette */ },
      },
      fontFamily: {
        sans: ['Inter', ...],
        mono: ['Fira Code', ...],
      },
    },
  },
  plugins: [],
}
```

**`frontend/postcss.config.js`:**
```javascript
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
}
```

**`frontend/src/shared/styles/tailwind.css`:**
```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

**Updated `frontend/src/shared/styles/main.css`:**
Added import for Tailwind directives at the top of the file.

#### 2. Updated SkyMap.vue Component

**Interface Updates:**
```typescript
interface SchedulingBlock {
  schedulingBlockId: string  // was: scheduling_block_id
  raInDeg: number           // was: right_ascension_deg
  decInDeg: number          // was: declination_deg
  priorityBin: string       // was: priority_bin
  scheduledFlag: boolean    // was: scheduled_flag
  requestedHours: number    // was: requested_hours
  totalVisibilityHours: number  // was: total_visibility_hours
  elevationRangeDeg?: number    // was: elevation_range_deg
  targetName?: string       // NEW
  targetId?: number         // NEW
}
```

**Enhanced Tooltip:**
```typescript
tooltip: {
  formatter: (params: any) => {
    const block = params.data.block
    let tooltip = ''
    
    // Display target information prominently
    if (block.targetName || block.targetId) {
      tooltip += '<div style="...">🎯 ${block.targetName} (ID: ${block.targetId})</div>'
    }
    
    // ... rest of tooltip with corrected property names
  }
}
```

**Updated all data mapping code** to use camelCase property names throughout:
- Chart data generation
- Filtering logic  
- Legend creation
- All references to block properties

### Backend Fixes

#### 1. Added Target Fields to SchedulingBlock Model

**`backend/src/models/schedule.rs`:**
```rust
pub struct SchedulingBlock {
    // ... existing fields ...
    
    // NEW: Target information
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_name: Option<String>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<i64>,
    
    pub dec_in_deg: f64,
    pub ra_in_deg: f64,
    // ... rest of fields ...
}
```

#### 2. Updated JSON Loader to Extract Target Data

**`backend/src/loaders/json.rs`:**
```rust
#[derive(Debug, serde::Deserialize)]
struct Target {
    #[serde(default, rename = "id_")]
    id: Option<i64>,
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "position_")]
    position: Position,
}

// In load_json function:
let mut block = SchedulingBlock {
    // ... existing fields ...
    target_name: raw.target.name.clone(),
    target_id: raw.target.id,
    // ... rest of initialization ...
}
```

#### 3. Updated All SchedulingBlock Instantiations

Updated 15+ locations across the codebase where `SchedulingBlock` structs are created:
- Test files: `comprehensive_test.rs`, `preprocessing/schedule.rs`
- Loader files: `csv.rs` (3 locations), `parser.rs`
- Analytics files: `distributions.rs`, `conflicts.rs`, `correlations.rs`, `metrics.rs`, `top_observations.rs`
- State management: `state.rs`

All instances now include:
```rust
target_name: None,  // or Some(value) for JSON loader
target_id: None,    // or Some(value) for JSON loader
```

### Testing

#### 1. Created Unit Tests

**`frontend/src/pages/__tests__/SkyMap.spec.ts`:**
- Component rendering tests
- Data loading and API integration tests
- Filter application tests (priority, scheduled status)
- Error handling tests
- Target information display tests
- Filter reset functionality tests

Tests cover:
- Initial loading state
- Successful data fetch
- Priority filtering
- Scheduled status filtering
- Filter reset
- Error handling
- Target info presence
- Observation count display

#### 2. Backend Compilation Test

Successfully compiled backend with all changes:
```bash
cd backend && cargo build
# Result: Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## Expected Behavior After Fixes

### Visual Improvements
1. **Proper Styling:** All Tailwind classes now render correctly:
   - Grid layouts work (`grid-cols-1`, `md:grid-cols-3`)
   - Cards have proper backgrounds, padding, shadows
   - Buttons are styled with colors and hover states
   - Forms have proper spacing and borders
   - Responsive design breakpoints work

2. **Sky Target Display:** When hovering over points on the sky map:
   ```
   🎯 T32 (ID: 10)
   ─────────────────
   Block ID: 1000004990
   RA: 158.03°
   Dec: -68.03°
   Priority: 8.50
   Priority Bin: High (10+)
   Status: Scheduled
   Requested: 0.33h
   Visibility: 12.50h
   ```

### Functional Improvements
1. All filters work correctly with proper data binding
2. Chart renders with correct coordinates
3. Legend displays properly
4. No console errors related to undefined properties
5. Target information flows from JSON → Backend → Frontend → UI

## Files Modified

### Frontend (8 files)
1. `frontend/tailwind.config.js` (created)
2. `frontend/postcss.config.js` (created)
3. `frontend/src/shared/styles/tailwind.css` (created)
4. `frontend/src/shared/styles/main.css` (updated)
5. `frontend/src/pages/SkyMap.vue` (major updates)
6. `frontend/src/pages/__tests__/SkyMap.spec.ts` (created)

### Backend (9 files)
1. `backend/src/models/schedule.rs` (added target fields)
2. `backend/src/loaders/json.rs` (extract target data)
3. `backend/src/loaders/csv.rs` (3 struct instantiations)
4. `backend/src/loaders/parser.rs` (struct instantiation)
5. `backend/tests/comprehensive_test.rs` (test block creation)
6. `backend/src/preprocessing/schedule.rs` (test block creation)
7. `backend/src/state.rs` (test block creation)

## Verification Steps

1. **Build and run the application:**
   ```bash
   docker compose up --build
   ```

2. **Check Tailwind is working:**
   - Open browser to `http://localhost:5173/sky-map`
   - Verify grid layout is working
   - Verify buttons have blue backgrounds
   - Verify cards have shadows and rounded corners

3. **Check target display:**
   - Load a dataset with target information
   - Hover over points on the sky map
   - Verify target name and ID appear in tooltip

4. **Test filters:**
   - Adjust priority range sliders
   - Change scheduled status dropdown
   - Change color-by mode
   - Verify chart updates correctly

5. **Run tests:**
   ```bash
   cd frontend
   npm test
   ```

## Regression Prevention

### For Developers

1. **Tailwind Configuration:**
   - Never remove `tailwind.config.js` or `postcss.config.js`
   - Ensure `@tailwind` directives remain in CSS files
   - Verify Tailwind processes files listed in `content` array

2. **Data Structure Consistency:**
   - When adding fields to `SchedulingBlock`, update ALL instantiation sites
   - Maintain camelCase for JSON serialization (Rust `#[serde(rename_all = "camelCase")]`)
   - Keep TypeScript interfaces in sync with Rust structs

3. **Testing:**
   - Run `cargo build` after backend model changes
   - Run `npm test` after frontend interface changes
   - Test in browser after any data structure changes

### For Code Reviewers

1. Check that new `SchedulingBlock` fields are:
   - Added to the struct definition
   - Included in all struct instantiations (search for `SchedulingBlock {`)
   - Added to the `new()` constructor if it exists
   - Reflected in TypeScript interfaces

2. Verify Tailwind usage:
   - Classes are in the configured `content` paths
   - Custom colors/fonts are in `tailwind.config.js` theme
   - No inline styles that could be Tailwind utilities

## Future Enhancements

1. **Target Filtering:** Add ability to filter by target name/ID
2. **Target Search:** Implement autocomplete search for targets
3. **Target Details:** Click on point to show full target details
4. **Export:** Export filtered view with target information
5. **Styling System:** Consider migrating fully to Tailwind or fully to custom CSS to avoid confusion

## Notes

- The existing `styles.css` file has many Tailwind-like utility classes that are NOT Tailwind. These are custom utilities that duplicate Tailwind functionality. Future work could consolidate these.
- The backend serialization uses camelCase due to JavaScript convention, but internally uses snake_case (Rust convention). This is standard practice.
- Target information is optional in the data model (`Option<T>`) to support datasets that don't have target metadata.
