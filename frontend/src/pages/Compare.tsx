/**
 * Compare page - Compare two schedules.
 */
import { useParams } from 'react-router-dom';
import { useCompare } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard } from '@/components';
import { CHANGE_TYPE_COLORS, type ChangeTypeKey } from '@/constants/colors';

function Compare() {
  const { scheduleId, otherId } = useParams();
  const currentId = parseInt(scheduleId ?? '0', 10);
  const comparisonId = parseInt(otherId ?? '0', 10);

  const { data, isLoading, error, refetch } = useCompare(currentId, comparisonId);

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
