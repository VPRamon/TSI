/**
 * Landing page - Schedule list and upload.
 * Redesigned for minimal, modern aesthetic with space theme.
 */
import { useState, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useSchedules, useCreateSchedule } from '@/hooks';
import { LoadingSpinner, ErrorMessage } from '@/components';

// SVG Icons
const UploadIcon = () => (
  <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" />
  </svg>
);

const DatabaseIcon = () => (
  <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
  </svg>
);

const FileIcon = () => (
  <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
  </svg>
);

function Landing() {
  const navigate = useNavigate();
  const { data, isLoading, error, refetch } = useSchedules();
  const createSchedule = useCreateSchedule();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [uploadName, setUploadName] = useState('');
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [isUploading, setIsUploading] = useState(false);
  const [uploadError, setUploadError] = useState('');

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setSelectedFile(file);
      setUploadError('');
    }
  };

  const handleFileUpload = async () => {
    if (!selectedFile) return;

    setIsUploading(true);
    setUploadError('');
    try {
      const content = await selectedFile.text();
      let scheduleJson: unknown;
      try {
        scheduleJson = JSON.parse(content);
      } catch (parseError) {
        const message =
          parseError instanceof Error ? parseError.message : 'Invalid JSON content';
        throw new Error(`Invalid JSON: ${message}`);
      }
      const name = uploadName || selectedFile.name.replace('.json', '');

      await createSchedule.mutateAsync({
        name,
        schedule_json: scheduleJson,
        populate_analytics: true,
      });

      setUploadName('');
      setSelectedFile(null);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    } catch (err) {
      console.error('Failed to upload schedule:', err);
      const message = err instanceof Error ? err.message : 'Unknown upload error';
      setUploadError(message);
    } finally {
      setIsUploading(false);
    }
  };

  const handleScheduleClick = (scheduleId: number) => {
    navigate(`/schedules/${scheduleId}/sky-map`);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <LoadingSpinner size="lg" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center min-h-screen p-4">
        <ErrorMessage
          title="Failed to load schedules"
          message={(error as Error).message}
          onRetry={() => refetch()}
        />
      </div>
    );
  }

  return (
    <div className="min-h-screen relative overflow-hidden">
      {/* Subtle space-themed background */}
      <div className="absolute inset-0 bg-gradient-to-b from-slate-950 via-slate-900 to-slate-950" aria-hidden="true" />
      <div className="absolute inset-0 opacity-30" style={{ backgroundImage: 'radial-gradient(circle at 2px 2px, rgb(148 163 184 / 0.15) 1px, transparent 0)', backgroundSize: '32px 32px' }} aria-hidden="true" />
      
      {/* Content */}
      <div className="relative z-10 max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-16 sm:py-24">
        {/* Header */}
        <header className="text-center mb-16 sm:mb-20">
          <h1 className="text-5xl sm:text-6xl lg:text-7xl font-bold text-white tracking-tight mb-4">
            Telescope Scheduling
            <span className="block text-transparent bg-clip-text bg-gradient-to-r from-blue-400 to-indigo-400 mt-2">
              Intelligence
            </span>
          </h1>
          <p className="text-slate-400 text-lg sm:text-xl mt-6 max-w-2xl mx-auto">
            Upload and analyze astronomical observation schedules with precision
          </p>
        </header>

        {/* Primary Action Cards */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 lg:gap-8 mb-12">
          {/* Upload Card */}
          <article className="group relative bg-slate-800/50 backdrop-blur-sm border border-slate-700/50 rounded-2xl p-8 transition-all duration-300 hover:bg-slate-800/70 hover:border-slate-600/50 hover:shadow-xl hover:shadow-blue-500/10 focus-within:ring-2 focus-within:ring-blue-500/50">
            <div className="flex flex-col h-full">
              {/* Icon & Title */}
              <div className="flex items-start gap-4 mb-6">
                <div className="p-3 bg-blue-500/10 rounded-xl text-blue-400 group-hover:bg-blue-500/20 transition-colors" aria-hidden="true">
                  <UploadIcon />
                </div>
                <div className="flex-1">
                  <h2 className="text-2xl font-semibold text-white mb-2">
                    Upload Schedule
                  </h2>
                  <p className="text-slate-400 text-sm leading-relaxed">
                    Import a new observation schedule from a JSON file
                  </p>
                </div>
              </div>

              {/* Upload Form */}
              <div className="space-y-4 flex-1">
                <div>
                  <label htmlFor="schedule-name" className="block text-sm font-medium text-slate-300 mb-2">
                    Schedule Name (optional)
                  </label>
                  <input
                    id="schedule-name"
                    type="text"
                    value={uploadName}
                    onChange={(e) => setUploadName(e.target.value)}
                    placeholder="Leave blank to use filename"
                    disabled={isUploading}
                    className="w-full px-4 py-2.5 bg-slate-900/50 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500/50 focus:border-transparent transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                  />
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
                    className={`flex items-center justify-center gap-2 w-full px-4 py-3 border-2 border-dashed rounded-lg transition-all cursor-pointer ${
                      isUploading
                        ? 'border-slate-700 bg-slate-800/30 cursor-not-allowed'
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
                  <div className="rounded-lg border border-red-500/40 bg-red-500/10 px-4 py-3 text-sm text-red-200" role="alert">
                    <strong className="font-medium">Error:</strong> {uploadError}
                  </div>
                )}
              </div>

              {/* Upload Button */}
              <button
                onClick={handleFileUpload}
                disabled={!selectedFile || isUploading}
                className="mt-6 w-full px-6 py-3.5 bg-gradient-to-r from-blue-600 to-blue-500 hover:from-blue-500 hover:to-blue-600 text-white font-medium rounded-lg transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:from-blue-600 disabled:hover:to-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-slate-900 shadow-lg shadow-blue-500/20"
              >
                {isUploading ? (
                  <span className="flex items-center justify-center gap-2">
                    <LoadingSpinner size="sm" />
                    Uploading...
                  </span>
                ) : (
                  'Upload & Analyze'
                )}
              </button>
            </div>
          </article>

          {/* Database Card */}
          <article className="group relative bg-slate-800/50 backdrop-blur-sm border border-slate-700/50 rounded-2xl p-8 transition-all duration-300 hover:bg-slate-800/70 hover:border-slate-600/50 hover:shadow-xl hover:shadow-indigo-500/10 focus-within:ring-2 focus-within:ring-indigo-500/50">
            <div className="flex flex-col h-full">
              {/* Icon & Title */}
              <div className="flex items-start gap-4 mb-6">
                <div className="p-3 bg-indigo-500/10 rounded-xl text-indigo-400 group-hover:bg-indigo-500/20 transition-colors" aria-hidden="true">
                  <DatabaseIcon />
                </div>
                <div className="flex-1">
                  <h2 className="text-2xl font-semibold text-white mb-2">
                    Load from Database
                  </h2>
                  <p className="text-slate-400 text-sm leading-relaxed">
                    Access previously uploaded observation schedules
                  </p>
                </div>
              </div>

              {/* Schedule List */}
              <div className="flex-1 mb-6">
                {data?.schedules.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-12 text-center">
                    <div className="w-16 h-16 mb-4 rounded-full bg-slate-700/30 flex items-center justify-center text-3xl" aria-hidden="true">
                      📭
                    </div>
                    <p className="text-slate-400 text-sm">
                      No schedules yet
                    </p>
                    <p className="text-slate-500 text-xs mt-1">
                      Upload one to get started
                    </p>
                  </div>
                ) : (
                  <div className="space-y-2 max-h-64 overflow-y-auto pr-2 scrollbar-thin">
                    {data?.schedules.map((schedule) => (
                      <button
                        key={schedule.schedule_id}
                        onClick={() => handleScheduleClick(schedule.schedule_id)}
                        className="w-full flex items-center justify-between p-4 bg-slate-900/40 hover:bg-slate-900/70 border border-slate-700/30 hover:border-indigo-500/30 rounded-lg transition-all duration-200 text-left group/item focus:outline-none focus:ring-2 focus:ring-indigo-500/50"
                      >
                        <div className="flex-1 min-w-0">
                          <p className="font-medium text-white truncate group-hover/item:text-indigo-300 transition-colors">
                            {schedule.schedule_name}
                          </p>
                          <p className="text-xs text-slate-500 mt-0.5">
                            ID: {schedule.schedule_id}
                          </p>
                        </div>
                        <svg className="w-5 h-5 text-slate-500 group-hover/item:text-indigo-400 group-hover/item:translate-x-1 transition-all flex-shrink-0 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                        </svg>
                      </button>
                    ))}
                  </div>
                )}
              </div>

              {/* Info Footer */}
              {data && data.schedules.length > 0 && (
                <div className="pt-4 border-t border-slate-700/50">
                  <p className="text-xs text-slate-500 text-center">
                    {data.total} {data.total === 1 ? 'schedule' : 'schedules'} available
                  </p>
                </div>
              )}
            </div>
          </article>
        </div>
      </div>
    </div>
  );
}

export default Landing;
