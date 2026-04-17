/**
 * Upload schedule card component.
 * Handles file selection and schedule upload with live log streaming.
 */
import { useState, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useCreateSchedule } from '@/hooks';
import { LoadingSpinner, LogStream } from '@/components';
import { OBSERVATORY_SITES, SITE_FROM_FILE, formatSiteLabel } from '@/constants';
import { dateToMjd } from '@/constants/dates';

// SVG Icons
const UploadIcon = () => (
  <svg className="h-8 w-8" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.5}
      d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
    />
  </svg>
);

const FileIcon = () => (
  <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
    />
  </svg>
);

export interface UploadScheduleCardProps {
  /** Callback when upload error occurs */
  onError?: (message: string) => void;
}

function UploadScheduleCard({ onError }: UploadScheduleCardProps) {
  const createSchedule = useCreateSchedule();
  const navigate = useNavigate();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [uploadName, setUploadName] = useState('');
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [uploadError, setUploadError] = useState('');
  const [jobId, setJobId] = useState<string | null>(null);
  const [selectedSiteIdx, setSelectedSiteIdx] = useState<string>('0');
  const [periodOverrideEnabled, setPeriodOverrideEnabled] = useState(false);
  const [periodStart, setPeriodStart] = useState('');
  const [periodEnd, setPeriodEnd] = useState('');
  const [periodError, setPeriodError] = useState('');

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setSelectedFile(file);
      setUploadError('');
    }
  };

  const handleFileUpload = async () => {
    if (!selectedFile) return;

    // Validate schedule period override if enabled.
    if (periodOverrideEnabled) {
      if (!periodStart || !periodEnd) {
        setPeriodError('Both start and end are required when the period override is enabled.');
        return;
      }
      const startDate = new Date(periodStart);
      const endDate = new Date(periodEnd);
      if (isNaN(startDate.getTime()) || isNaN(endDate.getTime())) {
        setPeriodError('Invalid date format.');
        return;
      }
      if (startDate >= endDate) {
        setPeriodError('Start must be strictly before end.');
        return;
      }
    }
    setPeriodError('');

    setIsUploading(true);
    setUploadError('');
    setJobId(null);
    try {
      const content = await selectedFile.text();
      let scheduleJson: unknown;
      try {
        scheduleJson = JSON.parse(content);
      } catch (parseError) {
        const message = parseError instanceof Error ? parseError.message : 'Invalid JSON content';
        throw new Error(`Invalid JSON: ${message}`);
      }
      const trimmedUploadName = uploadName.trim();
      const name = trimmedUploadName || selectedFile.name.replace(/\.json$/i, '');

      const locationOverride =
        selectedSiteIdx !== SITE_FROM_FILE
          ? OBSERVATORY_SITES[parseInt(selectedSiteIdx, 10)]?.location
          : undefined;

      const schedulePeriodOverride =
        periodOverrideEnabled && periodStart && periodEnd
          ? {
              start_mjd: dateToMjd(new Date(periodStart)),
              end_mjd: dateToMjd(new Date(periodEnd)),
            }
          : undefined;

      const response = await createSchedule.mutateAsync({
        name,
        schedule_json: scheduleJson,
        populate_analytics: true,
        location_override: locationOverride,
        schedule_period_override: schedulePeriodOverride,
      });

      // Set job ID to start streaming logs
      setJobId(response.job_id);
    } catch (err) {
      console.error('Failed to upload schedule:', err);
      const message = err instanceof Error ? err.message : 'Unknown upload error';
      setUploadError(message);
      onError?.(message);
      setIsUploading(false);
    }
  };

  const handleComplete = (result: unknown) => {
    setIsUploading(false);
    // Reset form on success
    setUploadName('');
    setSelectedFile(null);
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }

    // Navigate to the schedule validation page if we have a schedule_id
    if (result && typeof result === 'object' && 'schedule_id' in result) {
      const scheduleId = (result as { schedule_id: number }).schedule_id;
      setTimeout(() => navigate(`/schedules/${scheduleId}/validation`), 1500);
    }
  };

  const handleJobError = (error: string) => {
    setIsUploading(false);
    setUploadError(error);
    onError?.(error);
  };

  return (
    <article className="group relative rounded-2xl border border-slate-700/50 bg-slate-800/50 p-8 backdrop-blur-sm transition-all duration-300 focus-within:ring-2 focus-within:ring-blue-500/50 hover:border-slate-600/50 hover:bg-slate-800/70 hover:shadow-xl hover:shadow-blue-500/10">
      <div className="flex h-full flex-col">
        {/* Icon & Title */}
        <div className="mb-6 flex items-start gap-4">
          <div
            className="rounded-xl bg-blue-500/10 p-3 text-blue-400 transition-colors group-hover:bg-blue-500/20"
            aria-hidden="true"
          >
            <UploadIcon />
          </div>
          <div className="flex-1">
            <h2 className="mb-2 text-2xl font-semibold text-white">Import Schedule</h2>
            <p className="text-sm leading-relaxed text-slate-400">
              Import schedule analysis data from a JSON file
            </p>
          </div>
        </div>

        {/* Upload Form */}
        <div className="flex-1 space-y-4">
          <div>
            <label
              htmlFor="schedule-name"
              className="mb-2 block text-sm font-medium text-slate-300"
            >
              Schedule Name
            </label>
            <input
              id="schedule-name"
              type="text"
              value={uploadName}
              onChange={(e) => setUploadName(e.target.value)}
              placeholder="Enter the name you want to see across TSI"
              disabled={isUploading}
              className="w-full rounded-lg border border-slate-700 bg-slate-900/50 px-4 py-2.5 text-white placeholder-slate-500 transition-all focus:border-transparent focus:outline-none focus:ring-2 focus:ring-blue-500/50 disabled:cursor-not-allowed disabled:opacity-50"
            />
            <p className="mt-2 text-xs text-slate-500">
              This label is used throughout the app to identify the imported schedule.
            </p>
          </div>

          {/* Observatory Site */}
          <div>
            <label
              htmlFor="observatory-site"
              className="mb-2 block text-sm font-medium text-slate-300"
            >
              Observatory Site
            </label>
            <select
              id="observatory-site"
              value={selectedSiteIdx}
              onChange={(e) => setSelectedSiteIdx(e.target.value)}
              disabled={isUploading}
              className="w-full rounded-lg border border-slate-700 bg-slate-900/50 px-4 py-2.5 text-white transition-all focus:border-transparent focus:outline-none focus:ring-2 focus:ring-blue-500/50 disabled:cursor-not-allowed disabled:opacity-50"
            >
              {OBSERVATORY_SITES.map((site, i) => (
                <option key={i} value={String(i)}>
                  {formatSiteLabel(site)}
                </option>
              ))}
              <option value={SITE_FROM_FILE}>Use location from file</option>
            </select>
          </div>

          {/* Schedule Period Override */}
          <div className="rounded-lg border border-slate-700/60 bg-slate-900/30 p-4">
            <label className="flex cursor-pointer items-center gap-2 text-sm font-medium text-slate-300">
              <input
                type="checkbox"
                checked={periodOverrideEnabled}
                onChange={(e) => {
                  setPeriodOverrideEnabled(e.target.checked);
                  setPeriodError('');
                }}
                disabled={isUploading}
                className="h-4 w-4 rounded border-slate-600 bg-slate-800 accent-blue-500 disabled:cursor-not-allowed"
              />
              Override Schedule Period
            </label>
            <p className="mt-1 text-xs text-slate-500">
              Optional. Use when the uploaded file has no scheduled blocks or does not define the
              schedule window explicitly.
            </p>
            {periodOverrideEnabled && (
              <div className="mt-3 grid grid-cols-2 gap-3">
                <div>
                  <label
                    htmlFor="period-start"
                    className="mb-1 block text-xs font-medium text-slate-400"
                  >
                    Start (UTC)
                  </label>
                  <input
                    id="period-start"
                    type="datetime-local"
                    step="1"
                    value={periodStart}
                    onChange={(e) => setPeriodStart(e.target.value)}
                    disabled={isUploading}
                    className="w-full rounded-lg border border-slate-700 bg-slate-900/50 px-3 py-2 text-sm text-white transition-all focus:border-transparent focus:outline-none focus:ring-2 focus:ring-blue-500/50 disabled:cursor-not-allowed disabled:opacity-50"
                  />
                </div>
                <div>
                  <label
                    htmlFor="period-end"
                    className="mb-1 block text-xs font-medium text-slate-400"
                  >
                    End (UTC)
                  </label>
                  <input
                    id="period-end"
                    type="datetime-local"
                    step="1"
                    value={periodEnd}
                    onChange={(e) => setPeriodEnd(e.target.value)}
                    disabled={isUploading}
                    className="w-full rounded-lg border border-slate-700 bg-slate-900/50 px-3 py-2 text-sm text-white transition-all focus:border-transparent focus:outline-none focus:ring-2 focus:ring-blue-500/50 disabled:cursor-not-allowed disabled:opacity-50"
                  />
                </div>
                {periodError && (
                  <p className="col-span-2 text-xs text-red-400" role="alert">
                    {periodError}
                  </p>
                )}
              </div>
            )}
          </div>

          {/* File Input */}
          <div>
            <input
              ref={fileInputRef}
              type="file"
              accept=".json,application/json"
              onChange={handleFileSelect}
              disabled={isUploading}
              className="sr-only"
              id="file-upload"
              aria-describedby="file-upload-description"
            />
            <label
              htmlFor="file-upload"
              className={`flex w-full cursor-pointer items-center justify-center gap-2 rounded-lg border-2 border-dashed px-4 py-3 transition-all ${
                isUploading
                  ? 'cursor-not-allowed border-slate-700 bg-slate-800/30'
                  : selectedFile
                    ? 'border-blue-500/50 bg-blue-500/5 hover:bg-blue-500/10'
                    : 'border-slate-600 bg-slate-900/30 hover:border-blue-500/30 hover:bg-slate-900/50'
              }`}
            >
              <FileIcon />
              <span className="text-sm text-slate-300" id="file-upload-description">
                {selectedFile ? selectedFile.name : 'Choose JSON file'}
              </span>
            </label>
          </div>

          {/* Error Message */}
          {uploadError && (
            <div
              className="rounded-lg border border-red-500/40 bg-red-500/10 px-4 py-3 text-sm text-red-200"
              role="alert"
            >
              <strong className="font-medium">Error:</strong> {uploadError}
            </div>
          )}

          {/* Live Log Stream */}
          {jobId && (
            <LogStream
              jobId={jobId}
              apiBaseUrl="/api"
              onComplete={handleComplete}
              onError={handleJobError}
              maxHeight="250px"
            />
          )}
        </div>

        {/* Upload Button */}
        <button
          onClick={handleFileUpload}
          disabled={!selectedFile || isUploading}
          className="mt-6 w-full rounded-lg bg-gradient-to-r from-blue-600 to-blue-500 px-6 py-3.5 font-medium text-white shadow-lg shadow-blue-500/20 transition-all duration-300 hover:from-blue-500 hover:to-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-slate-900 disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:from-blue-600 disabled:hover:to-blue-500"
        >
          {isUploading ? (
            <span className="flex items-center justify-center gap-2">
              <LoadingSpinner size="sm" />
              Uploading...
            </span>
          ) : (
            'Import & Analyze'
          )}
        </button>
      </div>
    </article>
  );
}

export default UploadScheduleCard;
