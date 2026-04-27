/**
 * Help-popover content for the schedule comparison charts. Kept in a
 * dedicated module so the main chart file stays focused on rendering.
 */
import type { HelpContent } from '@/components/charts/HelpPopover';

export const KEY_METRICS_HELP: HelpContent = {
  title: 'Key metrics',
  summary:
    'Side-by-side bars comparing four headline metrics across the selected schedules.',
  bullets: [
    'Each schedule keeps the same colour throughout the page; the reference (left-most) is sky blue.',
    'Scheduling rate and priority capture are percentages of all eligible tasks; higher is better.',
    'Scheduled hours and gap count are absolute numbers; fewer gaps usually means a smoother night.',
    'Hover any bar to read its precise value.',
  ],
};

export const PRIORITY_BOX_HELP: HelpContent = {
  title: 'Scheduled task priority distribution',
  summary:
    'Box-and-whisker plot showing how the priorities of scheduled tasks are spread per schedule.',
  bullets: [
    'The box covers the inter-quartile range (25th–75th percentile); the line inside is the median.',
    'Whiskers extend up to 1.5× IQR; isolated dots are outliers (extreme priorities still scheduled).',
    'A higher box centroid means the schedule is biased towards high-priority tasks.',
    'Use the priority-range slider to focus on a specific band.',
  ],
};

export const TIME_USE_HELP: HelpContent = {
  title: 'Time-use breakdown',
  summary:
    'Horizontal 100 % stacked bar that decomposes the schedule window per outcome category.',
  bullets: [
    'Green = scheduled; amber = feasible but unused; orange = visible but no task fit.',
    'Blue = no target visible during that interval; grey = telescope was non-operable.',
    'Maximising the green segment is the long-term optimisation target.',
    'Hover a segment to see the exact percentage of the window it occupies.',
  ],
};

export const COMPARISON_FILTER_HELP: HelpContent = {
  title: 'Filters',
  summary: 'Restrict the data feeding every chart on this page.',
  bullets: [
    'Priority range filters the priority-distribution box plot and the cumulative-priority bar.',
    'Min scheduled hours hides schedules that fall below the threshold from every chart.',
    'Drag the thumbs; charts update live. Reset by extending each thumb to its bound.',
  ],
};
