/**
 * Validation page - Schedule validation report.
 * Redesigned with consistent layout primitives and improved table presentation.
 */
import { useParams } from 'react-router-dom';
import { useValidationReport } from '@/hooks';
import {
  LoadingSpinner,
  ErrorMessage,
  MetricCard,
  PageHeader,
  PageContainer,
  MetricsGrid,
} from '@/components';
import { CRITICALITY_CLASSES, type CriticalityKey } from '@/constants/colors';

function Validation() {
  const { scheduleId } = useParams();
  const id = parseInt(scheduleId ?? '0', 10);
  const { data, isLoading, error, refetch } = useValidationReport(id);

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
        title="Failed to load validation report"
        message={(error as Error).message}
        onRetry={() => refetch()}
      />
    );
  }

  if (!data) {
    return <ErrorMessage message="No data available" />;
  }

  const getCriticalityColor = (criticality: string) => {
    return (
      CRITICALITY_CLASSES[criticality as CriticalityKey] || 'bg-slate-500/20 text-slate-400'
    );
  };

  const totalIssues =
    data.impossible_blocks.length + data.validation_errors.length + data.validation_warnings.length;

  return (
    <PageContainer>
      {/* Header */}
      <PageHeader
        title="Validation"
        description="Schedule validation report and issues"
      />

      {/* Summary metrics */}
      <MetricsGrid columns={5}>
        <MetricCard label="Total Blocks" value={data.total_blocks} icon="📦" />
        <MetricCard label="Valid Blocks" value={data.valid_blocks} icon="✅" />
        <MetricCard label="Impossible" value={data.impossible_blocks.length} icon="🚫" />
        <MetricCard label="Errors" value={data.validation_errors.length} icon="❌" />
        <MetricCard label="Warnings" value={data.validation_warnings.length} icon="⚠️" />
      </MetricsGrid>

      {/* Overall status */}
      <div className="flex items-center gap-4 rounded-lg border border-slate-700 bg-slate-800/30 p-4">
        <div
          className={`flex h-14 w-14 shrink-0 items-center justify-center rounded-full ${
            totalIssues === 0 ? 'bg-emerald-500/20' : 'bg-amber-500/20'
          }`}
          aria-hidden="true"
        >
          <span className="text-2xl">{totalIssues === 0 ? '✅' : '⚠️'}</span>
        </div>
        <div>
          <h3 className="text-lg font-semibold text-white">
            {totalIssues === 0 ? 'All Clear!' : `${totalIssues} Issues Found`}
          </h3>
          <p className="text-sm text-slate-400">
            {totalIssues === 0
              ? 'No validation issues detected in this schedule.'
              : 'Review the issues below to improve schedule quality.'}
          </p>
        </div>
      </div>

      {/* Impossible blocks */}
      {data.impossible_blocks.length > 0 && (
        <section className="rounded-lg border border-slate-700 bg-slate-800/30">
          <div className="border-b border-slate-700/50 px-4 py-3">
            <h2 className="text-sm font-medium text-slate-300">
              Impossible Blocks ({data.impossible_blocks.length})
            </h2>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700/50">
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Block ID
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Issue Type
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Description
                  </th>
                  <th className="px-4 py-3 text-center text-xs font-medium uppercase tracking-wide text-slate-500">
                    Criticality
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-700/30">
                {data.impossible_blocks.map((issue, index) => (
                  <tr key={index} className="hover:bg-slate-700/20">
                    <td className="px-4 py-3 font-medium text-white">
                      {issue.original_block_id || issue.block_id}
                    </td>
                    <td className="px-4 py-3 text-slate-300">{issue.issue_type}</td>
                    <td className="max-w-md px-4 py-3 text-slate-300">{issue.description}</td>
                    <td className="px-4 py-3 text-center">
                      <span
                        className={`inline-flex rounded-full px-2 py-0.5 text-xs font-medium ${getCriticalityColor(issue.criticality)}`}
                      >
                        {issue.criticality}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      )}

      {/* Validation errors */}
      {data.validation_errors.length > 0 && (
        <section className="rounded-lg border border-slate-700 bg-slate-800/30">
          <div className="border-b border-slate-700/50 px-4 py-3">
            <h2 className="text-sm font-medium text-slate-300">
              Errors ({data.validation_errors.length})
            </h2>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700/50">
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Block ID
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Field
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Current
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Expected
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Description
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-700/30">
                {data.validation_errors.map((issue, index) => (
                  <tr key={index} className="hover:bg-slate-700/20">
                    <td className="px-4 py-3 font-medium text-white">
                      {issue.original_block_id || issue.block_id}
                    </td>
                    <td className="px-4 py-3 text-slate-300">{issue.field_name || '-'}</td>
                    <td className="px-4 py-3 font-mono text-sm text-red-400">
                      {issue.current_value || '-'}
                    </td>
                    <td className="px-4 py-3 font-mono text-sm text-emerald-400">
                      {issue.expected_value || '-'}
                    </td>
                    <td className="max-w-xs px-4 py-3 text-slate-300">{issue.description}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      )}

      {/* Validation warnings */}
      {data.validation_warnings.length > 0 && (
        <section className="rounded-lg border border-slate-700 bg-slate-800/30">
          <div className="border-b border-slate-700/50 px-4 py-3">
            <h2 className="text-sm font-medium text-slate-300">
              Warnings ({data.validation_warnings.length})
            </h2>
          </div>
          <div className="overflow-x-auto">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-slate-700/50">
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Block ID
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Issue Type
                  </th>
                  <th className="px-4 py-3 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                    Description
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-700/30">
                {data.validation_warnings.map((issue, index) => (
                  <tr key={index} className="hover:bg-slate-700/20">
                    <td className="px-4 py-3 font-medium text-white">
                      {issue.original_block_id || issue.block_id}
                    </td>
                    <td className="px-4 py-3 text-slate-300">{issue.issue_type}</td>
                    <td className="max-w-md px-4 py-3 text-slate-300">{issue.description}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>
      )}
    </PageContainer>
  );
}

export default Validation;
