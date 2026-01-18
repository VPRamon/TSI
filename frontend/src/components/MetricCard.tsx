/**
 * Metric card component for displaying single values.
 */
interface MetricCardProps {
  label: string;
  value: string | number;
  icon?: string;
  trend?: 'up' | 'down' | 'neutral';
  trendValue?: string;
  className?: string;
}

function MetricCard({
  label,
  value,
  icon,
  trend,
  trendValue,
  className = '',
}: MetricCardProps) {
  const trendColors = {
    up: 'text-green-500',
    down: 'text-red-500',
    neutral: 'text-slate-400',
  };

  const trendIcons = {
    up: '↑',
    down: '↓',
    neutral: '→',
  };

  return (
    <div className={`bg-slate-800 rounded-xl border border-slate-700 p-4 ${className}`}>
      <div className="flex items-start justify-between">
        <div>
          <p className="text-sm text-slate-400">{label}</p>
          <p className="text-2xl font-bold text-white mt-1">{value}</p>
          {trend && trendValue && (
            <p className={`text-sm mt-1 ${trendColors[trend]}`}>
              {trendIcons[trend]} {trendValue}
            </p>
          )}
        </div>
        {icon && <span className="text-2xl">{icon}</span>}
      </div>
    </div>
  );
}

export default MetricCard;
