-- Queries SQL para verificar el upload de schedules

-- =====================================================
-- 1. Ver todos los schedules subidos
-- =====================================================
SELECT 
    schedule_id,
    upload_timestamp,
    checksum,
    DATEDIFF(hour, upload_timestamp, SYSUTCDATETIME()) as hours_ago
FROM dbo.schedules
ORDER BY upload_timestamp DESC;

-- =====================================================
-- 2. Ver resumen de un schedule específico
-- =====================================================
DECLARE @schedule_id BIGINT = 1; -- Cambiar por el schedule_id deseado

SELECT 
    'Schedule Info' as section,
    s.schedule_id,
    s.upload_timestamp,
    s.checksum,
    COUNT(DISTINCT ssb.scheduling_block_id) as total_blocks
FROM dbo.schedules s
LEFT JOIN dbo.schedule_scheduling_blocks ssb 
    ON s.schedule_id = ssb.schedule_id
WHERE s.schedule_id = @schedule_id
GROUP BY s.schedule_id, s.upload_timestamp, s.checksum;

-- =====================================================
-- 3. Ver scheduling blocks de un schedule
-- =====================================================
DECLARE @schedule_id BIGINT = 1;

SELECT TOP 10
    sb.scheduling_block_id,
    t.name as target_name,
    t.ra_deg,
    t.dec_deg,
    sb.priority,
    sb.min_observation_sec,
    sb.requested_duration_sec,
    p.start_time_mjd as scheduled_start,
    p.stop_time_mjd as scheduled_stop,
    p.duration_sec as scheduled_duration
FROM dbo.schedule_scheduling_blocks ssb
JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
JOIN dbo.targets t ON sb.target_id = t.target_id
LEFT JOIN dbo.periods p ON ssb.scheduled_period_id = p.period_id
WHERE ssb.schedule_id = @schedule_id
ORDER BY sb.priority DESC, sb.scheduling_block_id;

-- =====================================================
-- 4. Ver constraints de los scheduling blocks
-- =====================================================
DECLARE @schedule_id BIGINT = 1;

SELECT TOP 10
    sb.scheduling_block_id,
    t.name as target_name,
    alt.min_alt_deg,
    alt.max_alt_deg,
    az.min_az_deg,
    az.max_az_deg
FROM dbo.schedule_scheduling_blocks ssb
JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
JOIN dbo.targets t ON sb.target_id = t.target_id
LEFT JOIN dbo.constraints c ON sb.constraints_id = c.constraints_id
LEFT JOIN dbo.altitude_constraints alt ON c.altitude_constraints_id = alt.altitude_constraints_id
LEFT JOIN dbo.azimuth_constraints az ON c.azimuth_constraints_id = az.azimuth_constraints_id
WHERE ssb.schedule_id = @schedule_id
ORDER BY sb.scheduling_block_id;

-- =====================================================
-- 5. Ver visibility periods de un scheduling block
-- =====================================================
DECLARE @scheduling_block_id BIGINT = 1000002662;

SELECT TOP 10
    vp.scheduling_block_id,
    p.period_id,
    p.start_time_mjd,
    p.stop_time_mjd,
    p.duration_sec,
    p.duration_sec / 3600.0 as duration_hours
FROM dbo.visibility_periods vp
JOIN dbo.periods p ON vp.period_id = p.period_id
WHERE vp.scheduling_block_id = @scheduling_block_id
ORDER BY p.start_time_mjd;

-- =====================================================
-- 6. Contar visibility periods por scheduling block
-- =====================================================
DECLARE @schedule_id BIGINT = 1;

SELECT 
    vp.scheduling_block_id,
    t.name as target_name,
    COUNT(*) as visibility_periods_count,
    SUM(p.duration_sec) as total_visibility_sec,
    SUM(p.duration_sec) / 3600.0 as total_visibility_hours
FROM dbo.visibility_periods vp
JOIN dbo.schedule_scheduling_blocks ssb 
    ON vp.schedule_id = ssb.schedule_id 
    AND vp.scheduling_block_id = ssb.scheduling_block_id
JOIN dbo.scheduling_blocks sb ON vp.scheduling_block_id = sb.scheduling_block_id
JOIN dbo.targets t ON sb.target_id = t.target_id
JOIN dbo.periods p ON vp.period_id = p.period_id
WHERE vp.schedule_id = @schedule_id
GROUP BY vp.scheduling_block_id, t.name
ORDER BY visibility_periods_count DESC;

-- =====================================================
-- 7. Ver targets únicos en un schedule
-- =====================================================
DECLARE @schedule_id BIGINT = 1;

SELECT 
    t.target_id,
    t.name,
    t.ra_deg,
    t.dec_deg,
    t.equinox,
    COUNT(DISTINCT sb.scheduling_block_id) as blocks_count
FROM dbo.schedule_scheduling_blocks ssb
JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
JOIN dbo.targets t ON sb.target_id = t.target_id
WHERE ssb.schedule_id = @schedule_id
GROUP BY t.target_id, t.name, t.ra_deg, t.dec_deg, t.equinox
ORDER BY blocks_count DESC;

-- =====================================================
-- 8. Estadísticas generales del schedule
-- =====================================================
DECLARE @schedule_id BIGINT = 1;

SELECT 
    'Total Blocks' as metric,
    CAST(COUNT(DISTINCT ssb.scheduling_block_id) as VARCHAR) as value
FROM dbo.schedule_scheduling_blocks ssb
WHERE ssb.schedule_id = @schedule_id

UNION ALL

SELECT 
    'Unique Targets',
    CAST(COUNT(DISTINCT sb.target_id) as VARCHAR)
FROM dbo.schedule_scheduling_blocks ssb
JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
WHERE ssb.schedule_id = @schedule_id

UNION ALL

SELECT 
    'Total Visibility Periods',
    CAST(COUNT(*) as VARCHAR)
FROM dbo.visibility_periods vp
WHERE vp.schedule_id = @schedule_id

UNION ALL

SELECT 
    'Scheduled Blocks',
    CAST(COUNT(*) as VARCHAR)
FROM dbo.schedule_scheduling_blocks ssb
WHERE ssb.schedule_id = @schedule_id
    AND ssb.scheduled_period_id IS NOT NULL

UNION ALL

SELECT 
    'Total Scheduled Time (hours)',
    CAST(SUM(p.duration_sec) / 3600.0 as VARCHAR)
FROM dbo.schedule_scheduling_blocks ssb
JOIN dbo.periods p ON ssb.scheduled_period_id = p.period_id
WHERE ssb.schedule_id = @schedule_id;

-- =====================================================
-- 9. Ver distribución de prioridades
-- =====================================================
DECLARE @schedule_id BIGINT = 1;

SELECT 
    sb.priority,
    COUNT(*) as blocks_count,
    AVG(sb.requested_duration_sec) / 60.0 as avg_duration_minutes,
    SUM(sb.requested_duration_sec) / 3600.0 as total_duration_hours
FROM dbo.schedule_scheduling_blocks ssb
JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
WHERE ssb.schedule_id = @schedule_id
GROUP BY sb.priority
ORDER BY sb.priority DESC;

-- =====================================================
-- 10. Buscar scheduling blocks sin visibility periods
-- =====================================================
DECLARE @schedule_id BIGINT = 1;

SELECT 
    sb.scheduling_block_id,
    t.name as target_name,
    sb.priority,
    sb.requested_duration_sec / 60.0 as requested_minutes
FROM dbo.schedule_scheduling_blocks ssb
JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
JOIN dbo.targets t ON sb.target_id = t.target_id
WHERE ssb.schedule_id = @schedule_id
    AND NOT EXISTS (
        SELECT 1 
        FROM dbo.visibility_periods vp 
        WHERE vp.schedule_id = ssb.schedule_id 
            AND vp.scheduling_block_id = ssb.scheduling_block_id
    )
ORDER BY sb.priority DESC;

-- =====================================================
-- 11. Timeline de observaciones programadas
-- =====================================================
DECLARE @schedule_id BIGINT = 1;

SELECT 
    p.start_time_mjd,
    p.stop_time_mjd,
    p.duration_sec / 60.0 as duration_minutes,
    t.name as target_name,
    sb.priority
FROM dbo.schedule_scheduling_blocks ssb
JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
JOIN dbo.targets t ON sb.target_id = t.target_id
JOIN dbo.periods p ON ssb.scheduled_period_id = p.period_id
WHERE ssb.schedule_id = @schedule_id
ORDER BY p.start_time_mjd;

-- =====================================================
-- 12. Ver uso de constraints
-- =====================================================
SELECT 
    'Altitude Constraints' as constraint_type,
    COUNT(*) as usage_count
FROM dbo.constraints c
WHERE c.altitude_constraints_id IS NOT NULL

UNION ALL

SELECT 
    'Azimuth Constraints',
    COUNT(*)
FROM dbo.constraints c
WHERE c.azimuth_constraints_id IS NOT NULL

UNION ALL

SELECT 
    'Time Constraints',
    COUNT(*)
FROM dbo.constraints c
WHERE c.time_constraints_id IS NOT NULL;

-- =====================================================
-- 13. Limpiar un schedule específico (CUIDADO!)
-- =====================================================
-- DESCOMENTAR SOLO SI ESTÁS SEGURO
-- DECLARE @schedule_id BIGINT = 1;
-- 
-- DELETE FROM dbo.visibility_periods WHERE schedule_id = @schedule_id;
-- DELETE FROM dbo.schedule_scheduling_blocks WHERE schedule_id = @schedule_id;
-- DELETE FROM dbo.schedules WHERE schedule_id = @schedule_id;
-- 
-- PRINT 'Schedule deleted successfully';
