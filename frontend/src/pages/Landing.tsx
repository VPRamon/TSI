/**
 * Landing page - Schedule list and upload.
 */
import { useState, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { useSchedules, useCreateSchedule } from '@/hooks';
import { Card, LoadingSpinner, ErrorMessage, MetricCard } from '@/components';

function Landing() {
  const navigate = useNavigate();
  const { data, isLoading, error, refetch } = useSchedules();
  const createSchedule = useCreateSchedule();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [uploadName, setUploadName] = useState('');
  const [isUploading, setIsUploading] = useState(false);

  const handleFileUpload = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setIsUploading(true);
    try {
      const content = await file.text();
      const scheduleJson = JSON.parse(content);
      const name = uploadName || file.name.replace('.json', '');

      await createSchedule.mutateAsync({
        name,
        schedule_json: scheduleJson,
        populate_analytics: true,
      });

      setUploadName('');
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    } catch (err) {
      console.error('Failed to upload schedule:', err);
      alert('Failed to upload schedule: ' + (err as Error).message);
    } finally {
      setIsUploading(false);
    }
  };

  const handleScheduleClick = (scheduleId: number) => {
    navigate(`/schedules/${scheduleId}/sky-map`);
  };

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
        title="Failed to load schedules"
        message={(error as Error).message}
        onRetry={() => refetch()}
      />
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-white">Telescope Scheduling Intelligence</h1>
          <p className="text-slate-400 mt-1">
            Upload and analyze observation schedules
          </p>
        </div>
      </div>

      {/* Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <MetricCard
          label="Total Schedules"
          value={data?.total ?? 0}
          icon="📊"
        />
        <MetricCard
          label="Database Status"
          value="Connected"
          icon="🔗"
        />
        <MetricCard
          label="API Version"
          value="v1"
          icon="🚀"
        />
      </div>

      {/* Upload card */}
      <Card title="Upload New Schedule">
        <div className="space-y-4">
          <div>
            <label htmlFor="schedule-name" className="block text-sm text-slate-400 mb-2">
              Schedule Name (optional)
            </label>
            <input
              id="schedule-name"
              type="text"
              value={uploadName}
              onChange={(e) => setUploadName(e.target.value)}
              placeholder="Enter schedule name..."
              className="w-full px-4 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-primary-500"
            />
          </div>
          <div>
            <label className="block text-sm text-slate-400 mb-2">
              Schedule JSON File
            </label>
            <div className="flex items-center gap-4">
              <input
                ref={fileInputRef}
                type="file"
                accept=".json"
                onChange={handleFileUpload}
                className="hidden"
                id="file-upload"
              />
              <label
                htmlFor="file-upload"
                className={`px-4 py-2 rounded-lg cursor-pointer transition-colors ${
                  isUploading
                    ? 'bg-slate-600 text-slate-400 cursor-not-allowed'
                    : 'bg-primary-600 hover:bg-primary-700 text-white'
                }`}
              >
                {isUploading ? (
                  <span className="flex items-center gap-2">
                    <LoadingSpinner size="sm" />
                    Uploading...
                  </span>
                ) : (
                  'Choose File'
                )}
              </label>
              <span className="text-slate-400 text-sm">
                Upload a JSON schedule file
              </span>
            </div>
          </div>
        </div>
      </Card>

      {/* Schedule list */}
      <Card title="Schedules">
        {data?.schedules.length === 0 ? (
          <div className="text-center py-8">
            <span className="text-4xl mb-4 block">📭</span>
            <p className="text-slate-400">No schedules yet. Upload one to get started!</p>
          </div>
        ) : (
          <div className="space-y-2">
            {data?.schedules.map((schedule) => (
              <button
                key={schedule.schedule_id}
                onClick={() => handleScheduleClick(schedule.schedule_id)}
                className="w-full flex items-center justify-between p-4 bg-slate-700/50 hover:bg-slate-700 rounded-lg transition-colors text-left"
              >
                <div>
                  <p className="font-medium text-white">{schedule.schedule_name}</p>
                  <p className="text-sm text-slate-400">ID: {schedule.schedule_id}</p>
                </div>
                <span className="text-slate-400">→</span>
              </button>
            ))}
          </div>
        )}
      </Card>
    </div>
  );
}

export default Landing;
