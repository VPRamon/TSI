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

    // Only update if different to avoid unnecessary re-renders
    if (schedule && schedule.schedule_id !== selectedSchedule?.schedule_id) {
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
