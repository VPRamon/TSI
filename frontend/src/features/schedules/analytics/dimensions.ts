/**
 * Generic dimension extraction over `algorithm_config` blobs.
 *
 * A scheduling algorithm may record any number of configuration knobs.
 * These helpers walk an arbitrary `Record<string, unknown>` and split the
 * fields into:
 *
 *   - `numeric`     fields whose values are finite numbers (sweep-able).
 *   - `categorical` fields whose values are strings/bools (facet-able).
 *
 * Used by the comparison charts and the algorithm-analysis panels to
 * render axis pickers and range filters without hard-coding the e/k/b
 * (or any other) knob set.
 */

export type DimensionKind = 'numeric' | 'categorical';

export interface Dimension {
  key: string;
  kind: DimensionKind;
  /** Distinct values observed across the supplied configs (sorted). */
  values: Array<number | string>;
}

export interface DimensionSet {
  numeric: Dimension[];
  categorical: Dimension[];
}

const isFiniteNumber = (v: unknown): v is number =>
  typeof v === 'number' && Number.isFinite(v);

/**
 * Build a {@link DimensionSet} from a list of `algorithm_config` blobs.
 *
 * Fields whose value type is mixed across runs are exposed as categorical
 * (everything is stringified). Fields with fewer than 2 distinct values
 * are dropped — they offer no comparative signal.
 */
export function extractDimensions(
  configs: Array<Record<string, unknown> | undefined | null>,
): DimensionSet {
  const observations = new Map<string, Set<string>>();
  const kinds = new Map<string, DimensionKind>();

  for (const cfg of configs) {
    if (!cfg) continue;
    for (const [k, raw] of Object.entries(cfg)) {
      if (raw === null || raw === undefined) continue;
      const set = observations.get(k) ?? new Set<string>();
      set.add(String(raw));
      observations.set(k, set);
      const previous = kinds.get(k);
      const current: DimensionKind = isFiniteNumber(raw) ? 'numeric' : 'categorical';
      if (previous && previous !== current) kinds.set(k, 'categorical');
      else kinds.set(k, current);
    }
  }

  const numeric: Dimension[] = [];
  const categorical: Dimension[] = [];

  for (const [key, raw] of observations.entries()) {
    if (raw.size < 2) continue;
    const kind = kinds.get(key) ?? 'categorical';
    if (kind === 'numeric') {
      const values = Array.from(raw, Number).sort((a, b) => a - b);
      numeric.push({ key, kind, values });
    } else {
      const values = Array.from(raw).sort();
      categorical.push({ key, kind, values });
    }
  }

  numeric.sort((a, b) => a.key.localeCompare(b.key));
  categorical.sort((a, b) => a.key.localeCompare(b.key));

  return { numeric, categorical };
}

/** Pull a dimension's value from a config, normalising numeric strings. */
export function readDimension(
  cfg: Record<string, unknown> | undefined | null,
  dim: Dimension,
): number | string | null {
  if (!cfg) return null;
  const raw = cfg[dim.key];
  if (raw === null || raw === undefined) return null;
  if (dim.kind === 'numeric') {
    const n = typeof raw === 'number' ? raw : Number(raw);
    return Number.isFinite(n) ? n : null;
  }
  return String(raw);
}
