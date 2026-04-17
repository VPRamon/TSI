/**
 * useScheduleSync - Sync route scheduleId with Zustand selectedSchedule.
 *
 * This hook fetches the schedule list and finds the current schedule by ID,
 * then updates the Zustand store so the Layout can display the schedule name.
 */
import { useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { useSchedules } from '@/hooks/useApi';
import { useAppStore } from '@/store/appStore';

export function useScheduleSync() {
  const { scheduleId } = useParams();
  const currentId = parseInt(scheduleId ?? '0', 10);
  const { setSelectedSchedule, selectedSchedule } = useAppStore();
  const { data: schedulesData } = useSchedules();

  useEffect(() => {
    if (!currentId || !schedulesData?.schedules) {
      // If no schedule ID in route, clear selection
      if (!currentId && selectedSchedule) {
        setSelectedSchedule(null);
      }
      return;
    }

    // Find the schedule in the list
    const schedule = schedulesData.schedules.find((s) => s.schedule_id === currentId);

    const scheduleChanged =
      !!schedule &&
      (schedule.schedule_id !== selectedSchedule?.schedule_id ||
        schedule.schedule_name !== selectedSchedule?.schedule_name);

    // Keep the selected schedule name in sync after imports or renames.
    if (scheduleChanged) {
      setSelectedSchedule(schedule);
    } else if (!schedule && selectedSchedule?.schedule_id === currentId) {
      // Schedule was deleted or not found
      setSelectedSchedule(null);
    }
  }, [currentId, schedulesData, selectedSchedule, setSelectedSchedule]);

  return {
    scheduleId: currentId,
    scheduleName: selectedSchedule?.schedule_name ?? null,
    schedules: schedulesData?.schedules ?? [],
  };
}
