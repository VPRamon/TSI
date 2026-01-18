/**
 * Validation page - Schedule validation report.
 */
import { useParams } from 'react-router-dom';
import { useValidationReport } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard } from '@/components';

function Validation() {
  const { scheduleId } = useParams();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data, isLoading, error, refetch } = useValidationReport(id);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return (
      <ErrorMessage
        title="Failed to load validation report"
        message={(error as Error).message}
        onRetry={() => refetch()}
      />
    );
  }

  if (!data) {
    return <ErrorMessage message="No data available" />;
  }

  const criticalityColors = {
    critical: 'bg-red-500/20 text-red-400',
    high: 'bg-orange-500/20 text-orange-400',
    medium: 'bg-yellow-500/20 text-yellow-400',
    low: 'bg-blue-500/20 text-blue-400',
  };

  const getCriticalityColor = (criticality: string) => {
    return criticalityColors[criticality as keyof typeof criticalityColors] || 'bg-slate-500/20 text-slate-400';
  };

  const totalIssues =
    data.impossible_blocks.length +
    data.validation_errors.length +
    data.validation_warnings.length;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white">Validation</h1>
        <p className="text-slate-400 mt-1">
          Schedule validation report and issues
        </p>
      </div>

      {/* Summary metrics */}
      <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
        <MetricCard
          label="Total Blocks"
          value={data.total_blocks}
          icon="📦"
        />
        <MetricCard
          label="Valid Blocks"
          value={data.valid_blocks}
          icon="✅"
        />
        <MetricCard
          label="Impossible"
          value={data.impossible_blocks.length}
          icon="🚫"
        />
        <MetricCard
          label="Errors"
          value={data.validation_errors.length}
          icon="❌"
        />
        <MetricCard
          label="Warnings"
          value={data.validation_warnings.length}
          icon="⚠️"
        />
      </div>

      {/* Overall status */}
      <Card>
        <div className="flex items-center gap-4">
          <div
            className={`w-16 h-16 rounded-full flex items-center justify-center ${
              totalIssues === 0 ? 'bg-green-500/20' : 'bg-yellow-500/20'
            }`}
          >
            <span className="text-3xl">{totalIssues === 0 ? '✅' : '⚠️'}</span>
          </div>
          <div>
            <h3 className="text-xl font-semibold text-white">
              {totalIssues === 0 ? 'All Clear!' : `${totalIssues} Issues Found`}
            </h3>
            <p className="text-slate-400">
              {totalIssues === 0
                ? 'No validation issues detected in this schedule.'
                : 'Review the issues below to improve schedule quality.'}
            </p>
          </div>
        </div>
      </Card>

      {/* Impossible blocks */}
      {data.impossible_blocks.length > 0 && (
        <Card title={`Impossible Blocks (${data.impossible_blocks.length})`}>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700">
                  <th className="text-left py-3 px-4 text-slate-400">Block ID</th>
                  <th className="text-left py-3 px-4 text-slate-400">Issue Type</th>
                  <th className="text-left py-3 px-4 text-slate-400">Description</th>
                  <th className="text-center py-3 px-4 text-slate-400">Criticality</th>
                </tr>
              </thead>
              <tbody>
                {data.impossible_blocks.map((issue, index) => (
                  <tr key={index} className="border-b border-slate-700/50">
                    <td className="py-3 px-4 text-white">
                      {issue.original_block_id || issue.block_id}
                    </td>
                    <td className="py-3 px-4 text-slate-300">{issue.issue_type}</td>
                    <td className="py-3 px-4 text-slate-300">{issue.description}</td>
                    <td className="py-3 px-4 text-center">
                      <span className={`px-2 py-1 rounded text-xs ${getCriticalityColor(issue.criticality)}`}>
                        {issue.criticality}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      )}

      {/* Validation errors */}
      {data.validation_errors.length > 0 && (
        <Card title={`Errors (${data.validation_errors.length})`}>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700">
                  <th className="text-left py-3 px-4 text-slate-400">Block ID</th>
                  <th className="text-left py-3 px-4 text-slate-400">Field</th>
                  <th className="text-left py-3 px-4 text-slate-400">Current</th>
                  <th className="text-left py-3 px-4 text-slate-400">Expected</th>
                  <th className="text-left py-3 px-4 text-slate-400">Description</th>
                </tr>
              </thead>
              <tbody>
                {data.validation_errors.map((issue, index) => (
                  <tr key={index} className="border-b border-slate-700/50">
                    <td className="py-3 px-4 text-white">
                      {issue.original_block_id || issue.block_id}
                    </td>
                    <td className="py-3 px-4 text-slate-300">{issue.field_name || '-'}</td>
                    <td className="py-3 px-4 text-red-400">{issue.current_value || '-'}</td>
                    <td className="py-3 px-4 text-green-400">{issue.expected_value || '-'}</td>
                    <td className="py-3 px-4 text-slate-300">{issue.description}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      )}

      {/* Validation warnings */}
      {data.validation_warnings.length > 0 && (
        <Card title={`Warnings (${data.validation_warnings.length})`}>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700">
                  <th className="text-left py-3 px-4 text-slate-400">Block ID</th>
                  <th className="text-left py-3 px-4 text-slate-400">Issue Type</th>
                  <th className="text-left py-3 px-4 text-slate-400">Description</th>
                </tr>
              </thead>
              <tbody>
                {data.validation_warnings.map((issue, index) => (
                  <tr key={index} className="border-b border-slate-700/50">
                    <td className="py-3 px-4 text-white">
                      {issue.original_block_id || issue.block_id}
                    </td>
                    <td className="py-3 px-4 text-slate-300">{issue.issue_type}</td>
                    <td className="py-3 px-4 text-slate-300">{issue.description}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </Card>
      )}
    </div>
  );
}

export default Validation;
