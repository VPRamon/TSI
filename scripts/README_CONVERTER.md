# InternalSDC to TSI Converter

## Overview

`convert_internal_sdc.py` converts scheduler output from the internalSDC format to TSI's expected JSON schemas.

## Input Format (internalSDC)

The input JSON should contain:
- **schedulingBlocks**: Array of `ObservationTask` entries with:
  - `id`: UUID string
  - `priority`: float
  - `estimated_duration`: seconds
  - `target.coordinates::ConstEquatorial`: RA/Dec in degrees
  - `constraint.constraints::AirmassAltitude`: altitude range

- **output.scheduledPeriods**: Map of UUID → scheduled time window (UTC datetime strings)

- **output.possiblePeriodsMap**: Map of constraint+target key → list of possible time windows

## Output Format (TSI)

### schedule.json
Conforms to [backend/docs/schedule.schema.json](../backend/docs/schedule.schema.json):
- Uses Modified Julian Date (MJD) for all time values
- String IDs (preserves UUID from input)
- Required fields: `id`, `priority`, `target_ra`, `target_dec`, `constraints`, `min_observation`, `requested_duration`
- Optional fields: `scheduled_period`, `visibility_periods`

### possible_periods.json
```json
{
  "blocks": {
    "<block_id>": [
      {"start": <mjd>, "stop": <mjd>},
      ...
    ]
  }
}
```

## Usage

```bash
# Basic usage
python scripts/convert_internal_sdc.py input.json -o output_dir/

# With custom schedule name
python scripts/convert_internal_sdc.py input.json -o output_dir/ -n "ScheduleName"

# Example with provided sample
python scripts/convert_internal_sdc.py \
  data/sensitive/sdc_cta_north_AGN_Spectra_N_wobble_Regular_internalSDC.json \
  -o converted/ \
  -n "CTA_North_AGN"
```

## How It Works

1. **Parse schedulingBlocks**: Extracts observation tasks and converts to TSI block format
   - Maps UUID `id` to `original_block_id` (the user-provided identifier)
   - Internal database `id` is not set (server-assigned on upload)
   - Extracts RA/Dec from nested coordinates structure
   - Converts `AirmassAltitude` constraint to `min_alt`/`max_alt`
   - Uses flexible defaults for azimuth (0-360°)

2. **Convert scheduled periods**: Joins `output.scheduledPeriods` with blocks by UUID
   - Parses UTC datetime strings: `"YYYY-MM-DD HH:MM:SS.ffffff"`
   - Converts to MJD floats

3. **Match possible periods**: Associates possible periods with blocks
   - Parses target coordinates from map keys (format: `"2h14m17.939s, 51°44'51.942"`)
   - Matches keys to blocks by RA/Dec proximity (tolerance: 0.01°)
   - Assigns same possible periods to all blocks sharing target+constraint
   - Drops weight values (TSI schema uses unweighted periods)

## Notes

- **Constraint handling**: Currently focused on `AirmassAltitude`. Other constraint types use full azimuth range (0-360°) as flexible defaults.
- **Time zone**: All datetime strings treated as UTC per TSI conventions.
- **Coordinate matching**: Uses simple Euclidean distance in RA/Dec space (adequate for small separations).
- **Multiple blocks per target**: All blocks with matching coordinates receive the same possible periods.
