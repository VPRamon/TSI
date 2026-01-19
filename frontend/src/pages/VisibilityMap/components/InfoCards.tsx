/**
 * InfoCards - Displays page header and metric cards.
 * 
 * Pure component that only re-renders when the underlying stats change.
 * Since these stats come from mapData (not histogram filters), they won't
 * re-render when bin/priority filters change.
 */
import { memo } from 'react';
import { PageHeader, MetricsGrid, MetricCard } from '@/components';
import { useRemountDetector, useRenderCounter } from '../hooks/useRemountDetector';

interface InfoCardsProps {
  totalCount: number;
  scheduledCount: number;
  priorityMin: number;
  priorityMax: number;
}

/**
 * InfoCards displays the page header and summary metrics.
 * Wrapped in React.memo with primitive props for optimal memoization.
 */
const InfoCards = memo(function InfoCards({
  totalCount,
  scheduledCount,
  priorityMin,
  priorityMax,
}: InfoCardsProps) {
  // DEV: Remount/render detection
  useRemountDetector('InfoCards');
  useRenderCounter('InfoCards');

  return (
    <>
      <PageHeader
        title="Visibility Map"
        description={`Target visibility over the observation period (${totalCount} blocks)`}
      />
      <MetricsGrid>
        <MetricCard label="Total Blocks" value={totalCount} icon="📊" />
        <MetricCard label="Scheduled" value={scheduledCount} icon="✅" />
        <MetricCard
          label="Priority Range"
          value={`${priorityMin.toFixed(1)} - ${priorityMax.toFixed(1)}`}
          icon="⭐"
        />
      </MetricsGrid>
    </>
  );
});

export default InfoCards;
