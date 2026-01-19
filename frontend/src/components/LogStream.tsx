/**
 * LogStream component - displays live logs from SSE endpoint.
 * Connects to the backend job logs endpoint and renders log entries in real-time.
 */
import { useEffect, useState, useRef } from 'react';

export interface LogEntry {
  timestamp: string;
  level: 'info' | 'success' | 'warning' | 'error';
  message: string;
}

export interface LogStreamProps {
  /** Job ID to stream logs for */
  jobId: string;
  /** Base URL for the API (default: http://localhost:3001) */
  apiBaseUrl?: string;
  /** Callback when job completes successfully */
  onComplete?: (result: unknown) => void;
  /** Callback when job fails */
  onError?: (error: string) => void;
  /** Maximum height of the log container */
  maxHeight?: string;
}

function LogStream({
  jobId,
  apiBaseUrl = 'http://localhost:3001',
  onComplete,
  onError,
  maxHeight = '300px',
}: LogStreamProps) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [status, setStatus] = useState<'connecting' | 'streaming' | 'completed' | 'failed'>('connecting');
  const logsEndRef = useRef<HTMLDivElement>(null);
  const eventSourceRef = useRef<EventSource | null>(null);

  useEffect(() => {
    if (!jobId) return;

    console.log('[LogStream] Connecting to SSE for job:', jobId);
    // Connect to SSE endpoint
    const url = `${apiBaseUrl}/v1/jobs/${jobId}/logs`;
    console.log('[LogStream] SSE URL:', url);
    const eventSource = new EventSource(url);
    eventSourceRef.current = eventSource;

    eventSource.onopen = () => {
      console.log('[LogStream] SSE connection opened');
      setStatus('streaming');
    };

    eventSource.onmessage = (event) => {
      console.log('[LogStream] Received message:', event.data);
      try {
        const log = JSON.parse(event.data) as LogEntry;
        setLogs((prev) => [...prev, log]);
      } catch (err) {
        console.error('Failed to parse log entry:', err);
      }
    };

    eventSource.addEventListener('complete', (event) => {
      console.log('[LogStream] Received complete event:', event.data);
      try {
        const data = JSON.parse(event.data);
        console.log('[LogStream] Parsed complete data:', data);
        console.log('[LogStream] Status value:', data.status, 'Type:', typeof data.status);
        if (data.status === 'completed') {
          console.log('[LogStream] Job completed successfully');
          setStatus('completed');
          onComplete?.(data.result);
        } else if (data.status === 'failed') {
          console.log('[LogStream] Job failed');
          setStatus('failed');
          onError?.('Job failed');
        } else {
          console.warn('[LogStream] Unknown status:', data.status);
        }
      } catch (err) {
        console.error('Failed to parse complete event:', err);
      }
      eventSource.close();
    });

    eventSource.onerror = (err) => {
      console.error('SSE error:', err);
      setStatus('failed');
      onError?.('Connection to log stream failed');
      eventSource.close();
    };

    // Cleanup on unmount
    return () => {
      if (eventSourceRef.current) {
        eventSourceRef.current.close();
      }
    };
  }, [jobId, apiBaseUrl, onComplete, onError]);

  // Auto-scroll to bottom when new logs arrive
  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  const getLevelColor = (level: LogEntry['level']) => {
    switch (level) {
      case 'success':
        return 'text-green-400';
      case 'warning':
        return 'text-yellow-400';
      case 'error':
        return 'text-red-400';
      default:
        return 'text-slate-300';
    }
  };

  const getStatusBadge = () => {
    switch (status) {
      case 'connecting':
        return (
          <span className="inline-flex items-center gap-1 rounded-full bg-blue-500/10 px-2 py-1 text-xs font-medium text-blue-400">
            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-blue-400"></span>
            Connecting...
          </span>
        );
      case 'streaming':
        return (
          <span className="inline-flex items-center gap-1 rounded-full bg-blue-500/10 px-2 py-1 text-xs font-medium text-blue-400">
            <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-blue-400"></span>
            Processing...
          </span>
        );
      case 'completed':
        return (
          <span className="inline-flex items-center gap-1 rounded-full bg-green-500/10 px-2 py-1 text-xs font-medium text-green-400">
            <span className="h-1.5 w-1.5 rounded-full bg-green-400"></span>
            Completed
          </span>
        );
      case 'failed':
        return (
          <span className="inline-flex items-center gap-1 rounded-full bg-red-500/10 px-2 py-1 text-xs font-medium text-red-400">
            <span className="h-1.5 w-1.5 rounded-full bg-red-400"></span>
            Failed
          </span>
        );
    }
  };

  return (
    <div className="rounded-lg border border-slate-700/50 bg-slate-900/50 p-4">
      {/* Header */}
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-semibold text-slate-200">Processing Logs</h3>
        {getStatusBadge()}
      </div>

      {/* Log container */}
      <div
        className="space-y-1 overflow-y-auto rounded border border-slate-800 bg-slate-950/80 p-3 font-mono text-xs"
        style={{ maxHeight }}
      >
        {logs.length === 0 && status === 'connecting' && (
          <div className="text-slate-500">Waiting for logs...</div>
        )}
        {logs.map((log, idx) => (
          <div key={idx} className="flex gap-2">
            <span className="text-slate-600">
              {new Date(log.timestamp).toLocaleTimeString()}
            </span>
            <span className={getLevelColor(log.level)}>{log.message}</span>
          </div>
        ))}
        <div ref={logsEndRef} />
      </div>
    </div>
  );
}

export default LogStream;
