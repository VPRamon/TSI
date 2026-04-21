/**
 * Distributions page - Statistical analysis of schedule properties.
 * Redesigned with consistent layout primitives and improved chart presentation.
 */
import { useParams } from 'react-router-dom';
import { useDistributions, usePlotlyTheme, usePlotlyDownload } from '@/hooks';
import {
  LoadingSpinner,
  ErrorMessage,
  Icon,
  MetricCard,
  PlotlyChart,
  PageHeader,
  PageContainer,
  MetricsGrid,
  ChartPanel,
} from '@/components';
import { STATUS_COLORS } from '@/constants/colors';

interface DistributionDetail {
  label: string;
  value: string;
}

interface DistributionSectionProps {
  title: string;
  details: DistributionDetail[];
  chartData: Plotly.Data[];
  layout: Partial<Plotly.Layout>;
  config: Partial<Plotly.Config>;
}

function DistributionSection({
  title,
  details,
  chartData,
  layout,
  config,
}: DistributionSectionProps) {
  const { onInitialized, downloadButton } = usePlotlyDownload(title);
  return (
    <ChartPanel title={title} headerActions={downloadButton}>
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[240px_minmax(0,1fr)] lg:items-center">
        <aside className="rounded-lg border border-slate-700 bg-slate-900/40 p-4 lg:self-center">
          <div className="divide-y divide-slate-700/60">
            {details.map((detail) => (
              <div
                key={detail.label}
                className="grid grid-cols-[minmax(0,1fr)_auto] items-center gap-4 py-3 first:pt-0 last:pb-0"
              >
                <p className="text-[11px] font-medium uppercase tracking-wide text-slate-500">
                  {detail.label}
                </p>
                <p className="text-right text-lg font-semibold text-white">{detail.value}</p>
              </div>
            ))}
          </div>
        </aside>

        <div className="min-w-0">
          <PlotlyChart
            data={chartData}
            layout={layout}
            config={config}
            height="350px"
            onInitialized={onInitialized}
          />
        </div>
      </div>
    </ChartPanel>
  );
}

function Distributions() {
  const { scheduleId } = useParams();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data, isLoading, error, refetch } = useDistributions(id);

  // Call hooks unconditionally (rules of hooks)
  const { layout: priorityLayout, config } = usePlotlyTheme({
    xAxis: { title: 'Priority' },
    yAxis: { title: 'Count' },
    barMode: 'overlay',
  });

  const { layout: visibilityLayout } = usePlotlyTheme({
    xAxis: { title: 'Total Visibility (hours)' },
    yAxis: { title: 'Count' },
    barMode: 'overlay',
  });

  const { layout: requestedDurationLayout } = usePlotlyTheme({
    xAxis: { title: 'Requested Duration (hours)' },
    yAxis: { title: 'Count' },
    barMode: 'overlay',
  });

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return (
      <ErrorMessage
        title="Failed to load distributions"
        message={(error as Error).message}
        onRetry={() => refetch()}
      />
    );
  }

  if (!data) {
    return <ErrorMessage message="No data available" />;
  }

  // Priority histogram
  const priorityHistogram: Plotly.Data[] = [
    {
      type: 'histogram',
      x: data.blocks.filter((b) => b.scheduled).map((b) => b.priority),
      name: 'Scheduled',
      marker: { color: STATUS_COLORS.scheduled },
      opacity: 0.7,
    },
    {
      type: 'histogram',
      x: data.blocks.filter((b) => !b.scheduled).map((b) => b.priority),
      name: 'Unscheduled',
      marker: { color: STATUS_COLORS.unscheduled },
      opacity: 0.7,
    },
  ];

  // Visibility histogram
  const visibilityHistogram: Plotly.Data[] = [
    {
      type: 'histogram',
      x: data.blocks.filter((b) => b.scheduled).map((b) => b.total_visibility_hours),
      name: 'Scheduled',
      marker: { color: STATUS_COLORS.scheduled },
      opacity: 0.7,
    },
    {
      type: 'histogram',
      x: data.blocks.filter((b) => !b.scheduled).map((b) => b.total_visibility_hours),
      name: 'Unscheduled',
      marker: { color: STATUS_COLORS.unscheduled },
      opacity: 0.7,
    },
  ];

  const requestedDurationHistogram: Plotly.Data[] = [
    {
      type: 'histogram',
      x: data.blocks.filter((b) => b.scheduled).map((b) => b.requested_hours),
      name: 'Scheduled',
      marker: { color: STATUS_COLORS.scheduled },
      opacity: 0.7,
    },
    {
      type: 'histogram',
      x: data.blocks.filter((b) => !b.scheduled).map((b) => b.requested_hours),
      name: 'Unscheduled',
      marker: { color: STATUS_COLORS.unscheduled },
      opacity: 0.7,
    },
  ];

  const priorityDetails: DistributionDetail[] = [
    { label: 'Mean', value: data.priority_stats.mean.toFixed(2) },
    { label: 'Median', value: data.priority_stats.median.toFixed(2) },
    { label: 'Std Dev', value: data.priority_stats.std_dev.toFixed(2) },
    {
      label: 'Range',
      value: `${data.priority_stats.min.toFixed(1)} – ${data.priority_stats.max.toFixed(1)}`,
    },
  ];

  const visibilityDetails: DistributionDetail[] = [
    { label: 'Mean', value: `${data.visibility_stats.mean.toFixed(1)}h` },
    { label: 'Median', value: `${data.visibility_stats.median.toFixed(1)}h` },
    { label: 'Std Dev', value: `${data.visibility_stats.std_dev.toFixed(1)}h` },
    {
      label: 'Range',
      value: `${data.visibility_stats.min.toFixed(0)} – ${data.visibility_stats.max.toFixed(0)}h`,
    },
  ];

  const requestedDurationDetails: DistributionDetail[] = [
    { label: 'Mean', value: `${data.requested_hours_stats.mean.toFixed(1)}h` },
    { label: 'Median', value: `${data.requested_hours_stats.median.toFixed(1)}h` },
    { label: 'Std Dev', value: `${data.requested_hours_stats.std_dev.toFixed(1)}h` },
    {
      label: 'Range',
      value: `${data.requested_hours_stats.min.toFixed(1)} – ${data.requested_hours_stats.max.toFixed(1)}h`,
    },
  ];

  return (
    <PageContainer>
      {/* Header */}
      <PageHeader title="Distributions" description="Statistical analysis of schedule properties" />

      {/* Summary metrics */}
      <MetricsGrid>
        <MetricCard
          label="Total Blocks"
          value={data.total_count}
          icon={<Icon name="chart-bar" />}
        />
        <MetricCard
          label="Scheduled"
          value={data.scheduled_count}
          icon={<Icon name="check-circle" />}
        />
        <MetricCard
          label="Unscheduled"
          value={data.unscheduled_count}
          icon={<Icon name="x-circle" />}
        />
        <MetricCard label="Impossible" value={data.impossible_count} icon={<Icon name="ban" />} />
      </MetricsGrid>

      <DistributionSection
        title="Priority Distribution"
        details={priorityDetails}
        chartData={priorityHistogram}
        layout={priorityLayout}
        config={config}
      />

      <DistributionSection
        title="Visibility Distribution"
        details={visibilityDetails}
        chartData={visibilityHistogram}
        layout={visibilityLayout}
        config={config}
      />

      <DistributionSection
        title="Requested Duration Distribution"
        details={requestedDurationDetails}
        chartData={requestedDurationHistogram}
        layout={requestedDurationLayout}
        config={config}
      />
    </PageContainer>
  );
}

export default Distributions;
