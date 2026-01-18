/**
 * Error message component.
 */
interface ErrorMessageProps {
  title?: string;
  message: string;
  onRetry?: () => void;
}

function ErrorMessage({ title = 'Error', message, onRetry }: ErrorMessageProps) {
  return (
    <div className="flex flex-col items-center justify-center p-8 text-center">
      <div className="w-16 h-16 mb-4 flex items-center justify-center rounded-full bg-red-500/10">
        <span className="text-3xl">⚠️</span>
      </div>
      <h3 className="text-lg font-semibold text-white mb-2">{title}</h3>
      <p className="text-slate-400 mb-4">{message}</p>
      {onRetry && (
        <button
          onClick={onRetry}
          className="px-4 py-2 bg-primary-600 hover:bg-primary-700 text-white rounded-lg transition-colors"
        >
          Try Again
        </button>
      )}
    </div>
  );
}

export default ErrorMessage;
