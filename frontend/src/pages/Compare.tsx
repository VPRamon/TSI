/**
 * Compare page - Compare two schedules.
 */
import { useParams } from 'react-router-dom';
import { useCompare } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard, PlotlyChart } from '@/components';
import { CHANGE_TYPE_COLORS, type ChangeTypeKey } from '@/constants/colors';
import { usePlotlyTheme } from '@/hooks/usePlotlyTheme';
import { useMemo } from 'react';

function Compare() {
  const { scheduleId, otherId } = useParams();
  const currentId = parseInt(scheduleId ?? '0', 10);
  const comparisonId = parseInt(otherId ?? '0', 10);

  const { data, isLoading, error, refetch } = useCompare(currentId, comparisonId);
  const plotlyTheme = usePlotlyTheme();

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
        title="Failed to load comparison"
        message={(error as Error).message}
        onRetry={() => refetch()}
      />
    );
  }

  if (!data) {
    return <ErrorMessage message="No data available" />;
  }

  // Prepare chart data
  const priorityDistributionData = useMemo(() => {
    const currentPriorities = data.current_blocks.map((b) => b.priority);
    const comparisonPriorities = data.comparison_blocks.map((b) => b.priority);
    
    const currentTrace = {
      x: currentPriorities,
      type: 'histogram' as const,
      name: data.current_name,
      opacity: 1.0,
      marker: {
        color: '#1f77b4',
        line: { color: '#0d5a9e', width: 2 },
      },
      nbinsx: 30,
    };

    const comparisonTrace = {
      x: comparisonPriorities,
      type: 'histogram' as const,
      name: data.comparison_name,
      opacity: 1.0,
      marker: {
        color: '#ff7f0e',
        line: { color: '#cc6600', width: 2 },
      },
      nbinsx: 30,
    };

    // Add larger dataset first for better visibility
    const traces =
      currentPriorities.length >= comparisonPriorities.length
        ? [currentTrace, comparisonTrace]
        : [comparisonTrace, currentTrace];

    return traces;
  }, [data]);

  const schedulingStatusData = useMemo(() => {
    return [
      {
        name: data.current_name,
        x: ['Scheduled', 'Unscheduled'],
        y: [data.current_stats.scheduled_count, data.current_stats.unscheduled_count],
        type: 'bar' as const,
        marker: {
          color: '#1f77b4',
          line: { color: '#0d5a9e', width: 2 },
        },
        text: [
          data.current_stats.scheduled_count.toLocaleString(),
          data.current_stats.unscheduled_count.toLocaleString(),
        ],
        textposition: 'auto' as const,
        textfont: { color: 'white', size: 12 },
      },
      {
        name: data.comparison_name,
        x: ['Scheduled', 'Unscheduled'],
        y: [data.comparison_stats.scheduled_count, data.comparison_stats.unscheduled_count],
        type: 'bar' as const,
        marker: {
          color: '#ff7f0e',
          line: { color: '#cc6600', width: 2 },
        },
        text: [
          data.comparison_stats.scheduled_count.toLocaleString(),
          data.comparison_stats.unscheduled_count.toLocaleString(),
        ],
        textposition: 'auto' as const,
        textfont: { color: 'white', size: 12 },
      },
    ];
  }, [data]);

  const changesData = useMemo(() => {
    const newlyScheduled = data.scheduling_changes.filter((c) => c.change_type === 'newly_scheduled').length;
    const newlyUnscheduled = data.scheduling_changes.filter((c) => c.change_type === 'newly_unscheduled').length;

    return [
      {
        x: ['Newly Scheduled', 'Newly Unscheduled'],
        y: [newlyScheduled, newlyUnscheduled],
        type: 'bar' as const,
        marker: {
          color: ['#2ca02c', '#d62728'],
          line: { color: ['#1a7a1a', '#8b1a1a'], width: 2 },
        },
        text: [newlyScheduled.toLocaleString(), newlyUnscheduled.toLocaleString()],
        textposition: 'auto' as const,
        textfont: { color: 'white', size: 14 },
      },
    ];
  }, [data]);

  const timeDistributionData = useMemo(() => {
    const currentTimes = data.current_blocks.map((b) => b.requested_hours);
    const comparisonTimes = data.comparison_blocks.map((b) => b.requested_hours);

    return [
      {
        y: currentTimes,
        type: 'box' as const,
        name: data.current_name,
        marker: {
          color: '#1f77b4',
          line: { color: '#0d5a9e', width: 2 },
        },
        fillcolor: '#1f77b4',
        line: { color: '#0d5a9e', width: 2 },
        boxmean: 'sd' as const,
      },
      {
        y: comparisonTimes,
        type: 'box' as const,
        name: data.comparison_name,
        marker: {
          color: '#ff7f0e',
          line: { color: '#cc6600', width: 2 },
        },
        fillcolor: '#ff7f0e',
        line: { color: '#cc6600', width: 2 },
        boxmean: 'sd' as const,
      },
    ];
  }, [data]);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Compare Schedules</h1>
        <p className="mt-1 text-slate-400">
          Comparing {data.current_name} vs {data.comparison_name}
        </p>
      </div>

      {/* Side by side stats */}
      <div className="grid grid-cols-1 gap-6 md:grid-cols-2">
        {/* Current schedule */}
        <Card title={data.current_name}>
          <div className="grid grid-cols-2 gap-4">
            <MetricCard label="Scheduled" value={data.current_stats.scheduled_count} icon="✅" />
            <MetricCard
              label="Unscheduled"
              value={data.current_stats.unscheduled_count}
              icon="❌"
            />
            <MetricCard
              label="Mean Priority"
              value={data.current_stats.mean_priority.toFixed(2)}
              icon="⭐"
            />
            <MetricCard
              label="Total Hours"
              value={`${data.current_stats.total_hours.toFixed(1)}h`}
              icon="⏱️"
            />
          </div>
        </Card>

        {/* Comparison schedule */}
        <Card title={data.comparison_name}>
          <div className="grid grid-cols-2 gap-4">
            <MetricCard label="Scheduled" value={data.comparison_stats.scheduled_count} icon="✅" />
            <MetricCard
              label="Unscheduled"
              value={data.comparison_stats.unscheduled_count}
              icon="❌"
            />
            <MetricCard
              label="Mean Priority"
              value={data.comparison_stats.mean_priority.toFixed(2)}
              icon="⭐"
            />
            <MetricCard
              label="Total Hours"
              value={`${data.comparison_stats.total_hours.toFixed(1)}h`}
              icon="⏱️"
            />
          </div>
        </Card>
      </div>

      {/* Overlap summary */}
      <Card title="Block Overlap">
        <div className="grid grid-cols-3 gap-4">
          <MetricCard label="Common Blocks" value={data.common_ids.length} icon="🔗" />
          <MetricCard
            label={`Only in ${data.current_name}`}
            value={data.only_in_current.length}
            icon="➡️"
          />
          <MetricCard
            label={`Only in ${data.comparison_name}`}
            value={data.only_in_comparison.length}
            icon="⬅️"
          />
        </div>
      </Card>

      {/* Visualizations */}
      <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
        {/* Priority Distribution */}
        <Card title="Priority Distribution">
          <PlotlyChart
            data={priorityDistributionData}
            layout={{
              ...plotlyTheme,
              barmode: 'overlay',
              xaxis: { title: { text: 'Priority' } },
              yaxis: { title: { text: 'Count' } },
              height: 450,
              legend: {
                orientation: 'h',
                yanchor: 'bottom',
                y: 1.02,
                xanchor: 'right',
                x: 1,
              },
            }}
            config={{ displayModeBar: false, responsive: true }}
          />
        </Card>

        {/* Scheduling Status */}
        <Card title="Scheduling Status">
          <PlotlyChart
            data={schedulingStatusData}
            layout={{
              ...plotlyTheme,
              barmode: 'group',
              yaxis: { title: { text: 'Number of Blocks' } },
              height: 450,
              legend: {
                orientation: 'h',
                yanchor: 'bottom',
                y: 1.02,
                xanchor: 'right',
                x: 1,
              },
            }}
            config={{ displayModeBar: false, responsive: true }}
          />
        </Card>

        {/* Scheduling Changes */}
        <Card title="Scheduling Changes">
          <PlotlyChart
            data={changesData}
            layout={{
              ...plotlyTheme,
              yaxis: { title: { text: 'Number of Blocks' } },
              height: 350,
              showlegend: false,
            }}
            config={{ displayModeBar: false, responsive: true }}
          />
        </Card>

        {/* Time Distribution */}
        <Card title="Time Distribution">
          <PlotlyChart
            data={timeDistributionData}
            layout={{
              ...plotlyTheme,
              yaxis: { title: { text: 'Requested Hours' } },
              height: 350,
              legend: {
                orientation: 'h',
                yanchor: 'bottom',
                y: 1.02,
                xanchor: 'right',
                x: 1,
              },
            }}
            config={{ displayModeBar: false, responsive: true }}
          />
        </Card>
      </div>

      {/* Scheduling changes */}
      {data.scheduling_changes.length > 0 && (
        <Card title={`Scheduling Changes (${data.scheduling_changes.length})`}>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700">
                  <th className="px-4 py-3 text-left text-slate-400">Block ID</th>
                  <th className="px-4 py-3 text-right text-slate-400">Priority</th>
                  <th className="px-4 py-3 text-center text-slate-400">Change</th>
                </tr>
              </thead>
              <tbody>
                {data.scheduling_changes.slice(0, 20).map((change) => (
                  <tr key={change.scheduling_block_id} className="border-b border-slate-700/50">
                    <td className="px-4 py-3 text-white">{change.scheduling_block_id}</td>
                    <td className="px-4 py-3 text-right text-slate-300">
                      {change.priority.toFixed(2)}
                    </td>
                    <td className="px-4 py-3 text-center">
                      <span
                        className={`rounded px-2 py-1 text-xs ${
                          CHANGE_TYPE_COLORS[change.change_type as ChangeTypeKey] ||
                          'bg-slate-500/20 text-slate-400'
                        }`}
                      >
                        {change.change_type}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {data.scheduling_changes.length > 20 && (
              <p className="mt-4 text-center text-slate-400">
                ... and {data.scheduling_changes.length - 20} more changes
              </p>
            )}
          </div>
        </Card>
      )}
    </div>
  );
}

export default Compare;
