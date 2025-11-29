//! Database CRUD operations for schedules, dark periods, and visibility data (SQL Server).

use chrono::{DateTime, Utc};
use tiberius::{Query, Row, numeric::Numeric};
use siderust::{
    astro::ModifiedJulianDate,
    units::time::Seconds,
    units::angular::Degrees,
    coordinates::spherical::direction::ICRS,
};

use super::models::{
    Schedule, SchedulingBlock, ScheduleMetadata, ScheduleInfo, 
    Period, Constraints, ScheduleId, SchedulingBlockId
};
use super::pool;

/// Perform a health check on the database connection.
pub async fn health_check() -> Result<bool, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection from pool: {}", e))?;

    Query::new("SELECT 1 as test")
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Health check query failed: {}", e))?;

    Ok(true)
}

pub async fn store_schedule(
    schedule: &Schedule,
) -> Result<ScheduleMetadata, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    //
    // 1) Check if a schedule with this checksum already exists
    //
    let mut lookup = Query::new(
        r#"
        SELECT schedule_id, schedule_name, upload_timestamp, checksum
        FROM dbo.schedules
        WHERE checksum = @P1
        "#,
    );
    lookup.bind(&schedule.checksum);

    let lookup_stream = lookup
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to run schedule lookup query: {e}"))?;

    // Either we get one row (schedule exists) or None (no schedule yet)
    if let Some(row) = lookup_stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read schedule lookup result: {e}"))?
    {
        let schedule_id: i64 = row
            .get::<i64, _>(0)
            .ok_or_else(|| "Existing schedule_id is NULL".to_string())?;
        let schedule_name: String = row
            .get::<&str, _>(1)
            .unwrap_or_default()
            .to_string();
        let upload_timestamp: DateTime<Utc> = row
            .get::<DateTime<Utc>, _>(2)
            .ok_or_else(|| "Existing upload_timestamp is NULL".to_string())?;
        let checksum: String = row
            .get::<&str, _>(3)
            .unwrap_or_default()
            .to_string();

        // Return metadata for the existing schedule
        return Ok(ScheduleMetadata {
            schedule_id: Some(schedule_id),
            schedule_name,
            upload_timestamp,
            checksum,
        });
    }

    //
    // 2) No existing schedule with this checksum → insert the full schedule
    //    (schedule row + all dependent rows) using your "full insert" function.
    //
    let metadata = insert_full_schedule(&mut *conn, schedule).await?;

    Ok(metadata)
}

/// Insert a complete schedule with all dependent entities.
async fn insert_full_schedule(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    schedule: &Schedule,
) -> Result<ScheduleMetadata, String> {
    // 1. Insert schedule record
    let mut insert = Query::new(
        r#"
        INSERT INTO dbo.schedules (schedule_name, checksum)
        OUTPUT inserted.schedule_id, inserted.upload_timestamp
        VALUES (@P1, @P2)
        "#,
    );
    insert.bind(&schedule.name);
    insert.bind(&schedule.checksum);

    let stream = insert
        .query(conn)
        .await
        .map_err(|e| format!("Failed to insert schedule: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to get schedule insert result: {e}"))?
        .ok_or_else(|| "No schedule_id returned from insert".to_string())?;

    let schedule_id: i64 = row
        .get::<i64, _>(0)
        .ok_or_else(|| "schedule_id is NULL".to_string())?;
    let upload_timestamp: DateTime<Utc> = row
        .get::<DateTime<Utc>, _>(1)
        .ok_or_else(|| "upload_timestamp is NULL".to_string())?;

    // 2. Insert dark periods for this schedule
    for period in &schedule.dark_periods {
        insert_dark_period(conn, schedule_id, period).await?;
    }

    // 3. Insert all scheduling blocks (targets, constraints, visibility, assignments)
    for block in &schedule.blocks {
        insert_scheduling_block(conn, schedule_id, block).await?;
    }

    Ok(ScheduleMetadata {
        schedule_id: Some(schedule_id),
        schedule_name: schedule.name.clone(),
        upload_timestamp,
        checksum: schedule.checksum.clone(),
    })
}

/// Insert a dark period for a schedule.
async fn insert_dark_period(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    schedule_id: i64,
    period: &Period,
) -> Result<(), String> {
    let mut insert = Query::new(
        r#"
        INSERT INTO dbo.schedule_dark_periods (schedule_id, start_time_mjd, stop_time_mjd)
        VALUES (@P1, @P2, @P3)
        "#,
    );
    insert.bind(schedule_id);
    insert.bind(period.start.value());
    insert.bind(period.stop.value());

    insert
        .execute(conn)
        .await
        .map_err(|e| format!("Failed to insert dark period: {e}"))?;

    Ok(())
}

/// Insert or retrieve a target.
async fn get_or_create_target(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    name: &str,
    target: &ICRS,
) -> Result<i64, String> {
    // Extract RA/Dec from ICRS (already in Degrees)
    let ra_deg = target.ra().value();
    let dec_deg = target.dec().value();

    // Try to find existing target
    let mut lookup = Query::new(
        r#"
        SELECT target_id FROM dbo.targets
        WHERE ra_deg = @P1 AND dec_deg = @P2 
          AND ra_pm_masyr = 0 AND dec_pm_masyr = 0 AND equinox = 2000.0
        "#,
    );
    lookup.bind(ra_deg);
    lookup.bind(dec_deg);

    let stream = lookup
        .query(conn)
        .await
        .map_err(|e| format!("Failed to lookup target: {e}"))?;

    if let Some(row) = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read target lookup: {e}"))?
    {
        return Ok(row.get::<i64, _>(0).ok_or_else(|| "target_id is NULL".to_string())?);
    }

    // Insert new target
    let mut insert = Query::new(
        r#"
        INSERT INTO dbo.targets (name, ra_deg, dec_deg, ra_pm_masyr, dec_pm_masyr, equinox)
        OUTPUT inserted.target_id
        VALUES (@P1, @P2, @P3, 0, 0, 2000.0)
        "#,
    );
    insert.bind(name);
    insert.bind(ra_deg);
    insert.bind(dec_deg);

    let stream = insert
        .query(conn)
        .await
        .map_err(|e| format!("Failed to insert target: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to get target insert result: {e}"))?
        .ok_or_else(|| "No target_id returned".to_string())?;

    Ok(row.get::<i64, _>(0).ok_or_else(|| "target_id is NULL".to_string())?)
}

/// Insert or retrieve altitude constraints.
async fn get_or_create_altitude_constraints(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    min_alt: f64,
    max_alt: f64,
) -> Result<i64, String> {
    // Try to find existing
    let mut lookup = Query::new(
        r#"
        SELECT altitude_constraints_id FROM dbo.altitude_constraints
        WHERE min_alt_deg = @P1 AND max_alt_deg = @P2
        "#,
    );
    lookup.bind(min_alt);
    lookup.bind(max_alt);

    let stream = lookup
        .query(conn)
        .await
        .map_err(|e| format!("Failed to lookup altitude constraints: {e}"))?;

    if let Some(row) = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read altitude constraints: {e}"))?
    {
        return Ok(row.get::<i64, _>(0).ok_or_else(|| "altitude_constraints_id is NULL".to_string())?);
    }

    // Insert new
    let mut insert = Query::new(
        r#"
        INSERT INTO dbo.altitude_constraints (min_alt_deg, max_alt_deg)
        OUTPUT inserted.altitude_constraints_id
        VALUES (@P1, @P2)
        "#,
    );
    insert.bind(min_alt);
    insert.bind(max_alt);

    let stream = insert
        .query(conn)
        .await
        .map_err(|e| format!("Failed to insert altitude constraints: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to get altitude constraints result: {e}"))?
        .ok_or_else(|| "No altitude_constraints_id returned".to_string())?;

    Ok(row.get::<i64, _>(0).ok_or_else(|| "altitude_constraints_id is NULL".to_string())?)
}

/// Insert or retrieve azimuth constraints.
async fn get_or_create_azimuth_constraints(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    min_az: f64,
    max_az: f64,
) -> Result<i64, String> {
    // Try to find existing
    let mut lookup = Query::new(
        r#"
        SELECT azimuth_constraints_id FROM dbo.azimuth_constraints
        WHERE min_az_deg = @P1 AND max_az_deg = @P2
        "#,
    );
    lookup.bind(min_az);
    lookup.bind(max_az);

    let stream = lookup
        .query(conn)
        .await
        .map_err(|e| format!("Failed to lookup azimuth constraints: {e}"))?;

    if let Some(row) = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read azimuth constraints: {e}"))?
    {
        return Ok(row.get::<i64, _>(0).ok_or_else(|| "azimuth_constraints_id is NULL".to_string())?);
    }

    // Insert new
    let mut insert = Query::new(
        r#"
        INSERT INTO dbo.azimuth_constraints (min_az_deg, max_az_deg)
        OUTPUT inserted.azimuth_constraints_id
        VALUES (@P1, @P2)
        "#,
    );
    insert.bind(min_az);
    insert.bind(max_az);

    let stream = insert
        .query(conn)
        .await
        .map_err(|e| format!("Failed to insert azimuth constraints: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to get azimuth constraints result: {e}"))?
        .ok_or_else(|| "No azimuth_constraints_id returned".to_string())?;

    Ok(row.get::<i64, _>(0).ok_or_else(|| "azimuth_constraints_id is NULL".to_string())?)
}

/// Insert or retrieve composite constraints.
async fn get_or_create_constraints(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    constraints: &Constraints,
) -> Result<i64, String> {
    // Get altitude and azimuth constraint IDs
    let altitude_id = get_or_create_altitude_constraints(
        conn,
        constraints.min_alt.value(),
        constraints.max_alt.value(),
    )
    .await?;

    let azimuth_id = get_or_create_azimuth_constraints(
        conn,
        constraints.min_az.value(),
        constraints.max_az.value(),
    )
    .await?;

    let (start_mjd, stop_mjd) = if let Some(period) = &constraints.fixed_time {
        (Some(period.start.value()), Some(period.stop.value()))
    } else {
        (None, None)
    };

    // Try to find existing constraint combo
    let mut lookup = Query::new(
        r#"
        SELECT constraints_id FROM dbo.constraints
        WHERE (start_time_mjd = @P1 OR (start_time_mjd IS NULL AND @P1 IS NULL))
          AND (stop_time_mjd = @P2 OR (stop_time_mjd IS NULL AND @P2 IS NULL))
          AND altitude_constraints_id = @P3
          AND azimuth_constraints_id = @P4
        "#,
    );
    lookup.bind(start_mjd);
    lookup.bind(stop_mjd);
    lookup.bind(altitude_id);
    lookup.bind(azimuth_id);

    let stream = lookup
        .query(conn)
        .await
        .map_err(|e| format!("Failed to lookup constraints: {e}"))?;

    if let Some(row) = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read constraints: {e}"))?
    {
        return Ok(row.get::<i64, _>(0).ok_or_else(|| "constraints_id is NULL".to_string())?);
    }

    // Insert new
    let mut insert = Query::new(
        r#"
        INSERT INTO dbo.constraints (start_time_mjd, stop_time_mjd, altitude_constraints_id, azimuth_constraints_id)
        OUTPUT inserted.constraints_id
        VALUES (@P1, @P2, @P3, @P4)
        "#,
    );
    insert.bind(start_mjd);
    insert.bind(stop_mjd);
    insert.bind(altitude_id);
    insert.bind(azimuth_id);

    let stream = insert
        .query(conn)
        .await
        .map_err(|e| format!("Failed to insert constraints: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to get constraints result: {e}"))?
        .ok_or_else(|| "No constraints_id returned".to_string())?;

    Ok(row.get::<i64, _>(0).ok_or_else(|| "constraints_id is NULL".to_string())?)
}

/// Insert a complete scheduling block with all dependencies.
async fn insert_scheduling_block(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    schedule_id: i64,
    block: &SchedulingBlock,
) -> Result<i64, String> {
    // 1. Get or create target
    let target_id = get_or_create_target(
        conn,
        &format!("SB_{}", block.id.0),
        &block.target,
    )
    .await?;

    // 2. Get or create constraints
    let constraints_id = get_or_create_constraints(conn, &block.constraints).await?;

    // 3. Insert visibility periods for this target+constraints
    for period in &block.visibility_periods {
        insert_visibility_period(conn, target_id, constraints_id, period).await?;
    }

    // 4. Insert the scheduling block itself
    let mut insert = Query::new(
        r#"
        INSERT INTO dbo.scheduling_blocks 
            (target_id, constraints_id, priority, min_observation_sec, requested_duration_sec)
        OUTPUT inserted.scheduling_block_id
        VALUES (@P1, @P2, @P3, @P4, @P5)
        "#,
    );
    insert.bind(target_id);
    insert.bind(constraints_id);
    
    // Convert priority to NUMERIC(4,1)
    let priority_numeric = Numeric::new_with_scale((block.priority * 10.0) as i128, 1);
    insert.bind(priority_numeric);
    insert.bind(block.min_observation.value() as i32);
    insert.bind(block.requested_duration.value() as i32);

    let stream = insert
        .query(conn)
        .await
        .map_err(|e| format!("Failed to insert scheduling block: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to get scheduling block result: {e}"))?
        .ok_or_else(|| "No scheduling_block_id returned".to_string())?;

    let sb_id: i64 = row.get::<i64, _>(0).ok_or_else(|| "scheduling_block_id is NULL".to_string())?;

    // 5. Link to schedule with optional execution window
    let (start_mjd, stop_mjd) = if let Some(period) = &block.scheduled_period {
        (Some(period.start.value()), Some(period.stop.value()))
    } else {
        (None, None)
    };

    let mut link = Query::new(
        r#"
        INSERT INTO dbo.schedule_scheduling_blocks 
            (schedule_id, scheduling_block_id, start_time_mjd, stop_time_mjd)
        VALUES (@P1, @P2, @P3, @P4)
        "#,
    );
    link.bind(schedule_id);
    link.bind(sb_id);
    link.bind(start_mjd);
    link.bind(stop_mjd);

    link.execute(conn)
        .await
        .map_err(|e| format!("Failed to link scheduling block to schedule: {e}"))?;

    Ok(sb_id)
}

/// Insert a visibility period for a target+constraints pair.
async fn insert_visibility_period(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    target_id: i64,
    constraints_id: i64,
    period: &Period,
) -> Result<(), String> {
    let mut insert = Query::new(
        r#"
        INSERT INTO dbo.visibility_periods (target_id, constraints_id, start_time_mjd, stop_time_mjd)
        SELECT @P1, @P2, @P3, @P4
        WHERE NOT EXISTS (
            SELECT 1 
            FROM dbo.visibility_periods
            WHERE target_id = @P5
              AND constraints_id = @P6
              AND start_time_mjd = @P7
              AND stop_time_mjd = @P8
        )
        "#,
    );
    insert.bind(target_id);
    insert.bind(constraints_id);
    insert.bind(period.start.value());
    insert.bind(period.stop.value());
    insert.bind(target_id);
    insert.bind(constraints_id);
    insert.bind(period.start.value());
    insert.bind(period.stop.value());

    insert
        .execute(conn)
        .await
        .map_err(|e| format!("Failed to ensure visibility period: {e}"))?;

    Ok(())
}

/// Fetch a schedule from the database by ID or name.
pub async fn get_schedule(
    schedule_id: Option<i64>,
    schedule_name: Option<&str>,
) -> Result<Schedule, String> {
    if schedule_id.is_none() && schedule_name.is_none() {
        return Err("Must provide either schedule_id or schedule_name".to_string());
    }

    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    // 1. Fetch schedule metadata
    let metadata = if let Some(id) = schedule_id {
        fetch_schedule_metadata_by_id(&mut *conn, id).await?
    } else {
        fetch_schedule_metadata_by_name(&mut *conn, schedule_name.unwrap()).await?
    };

    let db_schedule_id = metadata.schedule_id.ok_or_else(|| "Schedule has no ID".to_string())?;

    // 2. Fetch dark periods
    let dark_periods = fetch_dark_periods(&mut *conn, db_schedule_id).await?;

    // 3. Fetch all scheduling blocks for this schedule
    let blocks = fetch_scheduling_blocks(&mut *conn, db_schedule_id).await?;

    Ok(Schedule {
        id: Some(ScheduleId(db_schedule_id)),
        name: metadata.schedule_name,
        checksum: metadata.checksum,
        dark_periods,
        blocks,
    })
}

/// Fetch schedule metadata by ID.
async fn fetch_schedule_metadata_by_id(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    schedule_id: i64,
) -> Result<ScheduleMetadata, String> {
    let mut query = Query::new(
        r#"
        SELECT schedule_id, schedule_name, upload_timestamp, checksum
        FROM dbo.schedules
        WHERE schedule_id = @P1
        "#,
    );
    query.bind(schedule_id);

    let stream = query
        .query(conn)
        .await
        .map_err(|e| format!("Failed to fetch schedule metadata: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read schedule metadata: {e}"))?
        .ok_or_else(|| format!("Schedule {} not found", schedule_id))?;

    Ok(parse_schedule_metadata_row(row)?)
}

/// Fetch schedule metadata by name.
async fn fetch_schedule_metadata_by_name(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    schedule_name: &str,
) -> Result<ScheduleMetadata, String> {
    let mut query = Query::new(
        r#"
        SELECT schedule_id, schedule_name, upload_timestamp, checksum
        FROM dbo.schedules
        WHERE schedule_name = @P1
        "#,
    );
    query.bind(schedule_name);

    let stream = query
        .query(conn)
        .await
        .map_err(|e| format!("Failed to fetch schedule metadata: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read schedule metadata: {e}"))?
        .ok_or_else(|| format!("Schedule '{}' not found", schedule_name))?;

    Ok(parse_schedule_metadata_row(row)?)
}

/// Parse a schedule metadata row.
fn parse_schedule_metadata_row(row: Row) -> Result<ScheduleMetadata, String> {
    let schedule_id: i64 = row
        .get::<i64, _>(0)
        .ok_or_else(|| "schedule_id is NULL".to_string())?;
    let schedule_name: String = row
        .get::<&str, _>(1)
        .unwrap_or_default()
        .to_string();
    let upload_timestamp: DateTime<Utc> = row
        .get::<DateTime<Utc>, _>(2)
        .ok_or_else(|| "upload_timestamp is NULL".to_string())?;
    let checksum: String = row
        .get::<&str, _>(3)
        .unwrap_or_default()
        .to_string();

    Ok(ScheduleMetadata {
        schedule_id: Some(schedule_id),
        schedule_name,
        upload_timestamp,
        checksum,
    })
}

/// Fetch dark periods for a schedule.
async fn fetch_dark_periods(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    schedule_id: i64,
) -> Result<Vec<Period>, String> {
    let mut query = Query::new(
        r#"
        SELECT start_time_mjd, stop_time_mjd
        FROM dbo.schedule_dark_periods
        WHERE schedule_id = @P1
        ORDER BY start_time_mjd
        "#,
    );
    query.bind(schedule_id);

    let stream = query
        .query(conn)
        .await
        .map_err(|e| format!("Failed to fetch dark periods: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read dark periods: {e}"))?;

    let mut periods = Vec::new();
    for row in rows {
        let start: f64 = row.get::<f64, _>(0).ok_or_else(|| "start_time_mjd is NULL".to_string())?;
        let stop: f64 = row.get::<f64, _>(1).ok_or_else(|| "stop_time_mjd is NULL".to_string())?;
        
        if let Some(period) = Period::new(
            ModifiedJulianDate::new(start),
            ModifiedJulianDate::new(stop),
        ) {
            periods.push(period);
        }
    }

    Ok(periods)
}

/// Fetch all scheduling blocks for a schedule.
async fn fetch_scheduling_blocks(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    schedule_id: i64,
) -> Result<Vec<SchedulingBlock>, String> {
    let mut query = Query::new(
        r#"
        SELECT 
            sb.scheduling_block_id,
            sb.priority,
            sb.min_observation_sec,
            sb.requested_duration_sec,
            t.ra_deg,
            t.dec_deg,
            ac.min_alt_deg,
            ac.max_alt_deg,
            az.min_az_deg,
            az.max_az_deg,
            c.start_time_mjd,
            c.stop_time_mjd,
            ssb.start_time_mjd as scheduled_start,
            ssb.stop_time_mjd as scheduled_stop
        FROM dbo.schedule_scheduling_blocks ssb
        JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
        JOIN dbo.targets t ON sb.target_id = t.target_id
        LEFT JOIN dbo.constraints c ON sb.constraints_id = c.constraints_id
        LEFT JOIN dbo.altitude_constraints ac ON c.altitude_constraints_id = ac.altitude_constraints_id
        LEFT JOIN dbo.azimuth_constraints az ON c.azimuth_constraints_id = az.azimuth_constraints_id
        WHERE ssb.schedule_id = @P1
        "#,
    );
    query.bind(schedule_id);

    let stream = query
        .query(conn)
        .await
        .map_err(|e| format!("Failed to fetch scheduling blocks: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read scheduling blocks: {e}"))?;

    let mut blocks = Vec::new();
    for row in rows {
        let sb_id: i64 = row.get::<i64, _>(0).ok_or_else(|| "scheduling_block_id is NULL".to_string())?;
        let priority: Numeric = row.get::<Numeric, _>(1).ok_or_else(|| "priority is NULL".to_string())?;
        let min_obs: i32 = row.get::<i32, _>(2).ok_or_else(|| "min_observation_sec is NULL".to_string())?;
        let req_dur: i32 = row.get::<i32, _>(3).ok_or_else(|| "requested_duration_sec is NULL".to_string())?;
        let ra: f64 = row.get::<f64, _>(4).ok_or_else(|| "ra_deg is NULL".to_string())?;
        let dec: f64 = row.get::<f64, _>(5).ok_or_else(|| "dec_deg is NULL".to_string())?;
        
        let min_alt = row.get::<f64, _>(6).unwrap_or(0.0);
        let max_alt = row.get::<f64, _>(7).unwrap_or(90.0);
        let min_az = row.get::<f64, _>(8).unwrap_or(0.0);
        let max_az = row.get::<f64, _>(9).unwrap_or(360.0);
        
        let constraint_start: Option<f64> = row.get(10);
        let constraint_stop: Option<f64> = row.get(11);
        let scheduled_start: Option<f64> = row.get(12);
        let scheduled_stop: Option<f64> = row.get(13);

        // Build constraints
        let fixed_time = if let (Some(s), Some(e)) = (constraint_start, constraint_stop) {
            Period::new(ModifiedJulianDate::new(s), ModifiedJulianDate::new(e))
        } else {
            None
        };

        let constraints = Constraints {
            min_alt: Degrees::new(min_alt),
            max_alt: Degrees::new(max_alt),
            min_az: Degrees::new(min_az),
            max_az: Degrees::new(max_az),
            fixed_time,
        };

        // Build target (ICRS)
        let target = ICRS::new(Degrees::new(ra), Degrees::new(dec));

        // Build scheduled period
        let scheduled_period = if let (Some(s), Some(e)) = (scheduled_start, scheduled_stop) {
            Period::new(ModifiedJulianDate::new(s), ModifiedJulianDate::new(e))
        } else {
            None
        };

        // Fetch visibility periods for this block
        let visibility_periods = fetch_visibility_periods_for_block(conn, sb_id).await?;

        // Convert priority from Numeric to f32
        let priority_f32 = priority.value() as f64 / 10.0;

        blocks.push(SchedulingBlock {
            id: SchedulingBlockId(sb_id),
            target,
            constraints,
            priority: priority_f32 as f32,
            min_observation: Seconds::new(min_obs as f64),
            requested_duration: Seconds::new(req_dur as f64),
            visibility_periods,
            scheduled_period,
        });
    }

    Ok(blocks)
}

/// Fetch visibility periods for a scheduling block.
async fn fetch_visibility_periods_for_block(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    sb_id: i64,
) -> Result<Vec<Period>, String> {
    // First get target_id and constraints_id for this block
    let mut block_query = Query::new(
        r#"
        SELECT target_id, constraints_id
        FROM dbo.scheduling_blocks
        WHERE scheduling_block_id = @P1
        "#,
    );
    block_query.bind(sb_id);

    let stream = block_query
        .query(conn)
        .await
        .map_err(|e| format!("Failed to fetch block info: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read block info: {e}"))?
        .ok_or_else(|| format!("Block {} not found", sb_id))?;

    let target_id: i64 = row.get::<i64, _>(0).ok_or_else(|| "target_id is NULL".to_string())?;
    let constraints_id: Option<i64> = row.get(1);

    if let Some(cid) = constraints_id {
        // Fetch visibility periods
        let mut vis_query = Query::new(
            r#"
            SELECT start_time_mjd, stop_time_mjd
            FROM dbo.visibility_periods
            WHERE target_id = @P1 AND constraints_id = @P2
            ORDER BY start_time_mjd
            "#,
        );
        vis_query.bind(target_id);
        vis_query.bind(cid);

        let stream = vis_query
            .query(conn)
            .await
            .map_err(|e| format!("Failed to fetch visibility periods: {e}"))?;

        let rows = stream
            .into_first_result()
            .await
            .map_err(|e| format!("Failed to read visibility periods: {e}"))?;

        let mut periods = Vec::new();
        for row in rows {
            let start: f64 = row.get::<f64, _>(0).ok_or_else(|| "start_time_mjd is NULL".to_string())?;
            let stop: f64 = row.get::<f64, _>(1).ok_or_else(|| "stop_time_mjd is NULL".to_string())?;
            
            if let Some(period) = Period::new(
                ModifiedJulianDate::new(start),
                ModifiedJulianDate::new(stop),
            ) {
                periods.push(period);
            }
        }

        Ok(periods)
    } else {
        Ok(Vec::new())
    }
}

/// List all available schedules with metadata.
pub async fn list_schedules() -> Result<Vec<ScheduleInfo>, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let query = Query::new(
        r#"
        SELECT 
            s.schedule_id,
            s.schedule_name,
            s.upload_timestamp,
            s.checksum,
            COUNT(DISTINCT ssb.scheduling_block_id) as total_blocks,
            COUNT(DISTINCT CASE WHEN ssb.start_time_mjd IS NOT NULL THEN ssb.scheduling_block_id END) as scheduled_blocks
        FROM dbo.schedules s
        LEFT JOIN dbo.schedule_scheduling_blocks ssb ON s.schedule_id = ssb.schedule_id
        GROUP BY s.schedule_id, s.schedule_name, s.upload_timestamp, s.checksum
        ORDER BY s.upload_timestamp DESC
        "#,
    );

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to list schedules: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read schedules: {e}"))?;

    let mut schedules = Vec::new();
    for row in rows {
        let schedule_id: i64 = row.get::<i64, _>(0).ok_or_else(|| "schedule_id is NULL".to_string())?;
        let schedule_name: String = row.get::<&str, _>(1).unwrap_or_default().to_string();
        let upload_timestamp: DateTime<Utc> = row.get::<DateTime<Utc>, _>(2)
            .ok_or_else(|| "upload_timestamp is NULL".to_string())?;
        let checksum: String = row.get::<&str, _>(3).unwrap_or_default().to_string();
        let total_blocks: i32 = row.get::<i32, _>(4).unwrap_or(0);
        let scheduled_blocks: i32 = row.get::<i32, _>(5).unwrap_or(0);

        let metadata = ScheduleMetadata {
            schedule_id: Some(schedule_id),
            schedule_name,
            upload_timestamp,
            checksum,
        };

        let total = total_blocks as usize;
        let scheduled = scheduled_blocks as usize;

        schedules.push(ScheduleInfo {
            metadata,
            total_blocks: total,
            scheduled_blocks: scheduled,
            unscheduled_blocks: total.saturating_sub(scheduled),
        });
    }

    Ok(schedules)
}

/// Get a specific scheduling block by ID.
pub async fn get_scheduling_block(sb_id: i64) -> Result<SchedulingBlock, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let mut query = Query::new(
        r#"
        SELECT 
            sb.scheduling_block_id,
            sb.priority,
            sb.min_observation_sec,
            sb.requested_duration_sec,
            t.ra_deg,
            t.dec_deg,
            ac.min_alt_deg,
            ac.max_alt_deg,
            az.min_az_deg,
            az.max_az_deg,
            c.start_time_mjd,
            c.stop_time_mjd
        FROM dbo.scheduling_blocks sb
        JOIN dbo.targets t ON sb.target_id = t.target_id
        LEFT JOIN dbo.constraints c ON sb.constraints_id = c.constraints_id
        LEFT JOIN dbo.altitude_constraints ac ON c.altitude_constraints_id = ac.altitude_constraints_id
        LEFT JOIN dbo.azimuth_constraints az ON c.azimuth_constraints_id = az.azimuth_constraints_id
        WHERE sb.scheduling_block_id = @P1
        "#,
    );
    query.bind(sb_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch scheduling block: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read scheduling block: {e}"))?
        .ok_or_else(|| format!("Scheduling block {} not found", sb_id))?;

    let sb_id: i64 = row.get::<i64, _>(0).ok_or_else(|| "scheduling_block_id is NULL".to_string())?;
    let priority: Numeric = row.get::<Numeric, _>(1).ok_or_else(|| "priority is NULL".to_string())?;
    let min_obs: i32 = row.get::<i32, _>(2).ok_or_else(|| "min_observation_sec is NULL".to_string())?;
    let req_dur: i32 = row.get::<i32, _>(3).ok_or_else(|| "requested_duration_sec is NULL".to_string())?;
    let ra: f64 = row.get::<f64, _>(4).ok_or_else(|| "ra_deg is NULL".to_string())?;
    let dec: f64 = row.get::<f64, _>(5).ok_or_else(|| "dec_deg is NULL".to_string())?;
    
    let min_alt = row.get::<f64, _>(6).unwrap_or(0.0);
    let max_alt = row.get::<f64, _>(7).unwrap_or(90.0);
    let min_az = row.get::<f64, _>(8).unwrap_or(0.0);
    let max_az = row.get::<f64, _>(9).unwrap_or(360.0);
    
    let constraint_start: Option<f64> = row.get(10);
    let constraint_stop: Option<f64> = row.get(11);

    let fixed_time = if let (Some(s), Some(e)) = (constraint_start, constraint_stop) {
        Period::new(ModifiedJulianDate::new(s), ModifiedJulianDate::new(e))
    } else {
        None
    };

    let constraints = Constraints {
        min_alt: Degrees::new(min_alt),
        max_alt: Degrees::new(max_alt),
        min_az: Degrees::new(min_az),
        max_az: Degrees::new(max_az),
        fixed_time,
    };

    let target = ICRS::new(Degrees::new(ra), Degrees::new(dec));
    let visibility_periods = fetch_visibility_periods_for_block(&mut *conn, sb_id).await?;
    let priority_f32 = priority.value() as f64 / 10.0;

    Ok(SchedulingBlock {
        id: SchedulingBlockId(sb_id),
        target,
        constraints,
        priority: priority_f32 as f32,
        min_observation: Seconds::new(min_obs as f64),
        requested_duration: Seconds::new(req_dur as f64),
        visibility_periods,
        scheduled_period: None, // Not stored in scheduling_blocks table, only in junction
    })
}

/// Get all scheduling blocks for a schedule.
pub async fn get_blocks_for_schedule(schedule_id: i64) -> Result<Vec<SchedulingBlock>, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    fetch_scheduling_blocks(&mut *conn, schedule_id).await
}

/// Fetch a schedule and return as Polars DataFrame (for Python bindings).
/*
pub async fn fetch_schedule(
    schedule_id: Option<i64>,
    schedule_name: Option<&str>,
) -> Result<DataFrame, String> {
    let schedule = get_schedule(schedule_id, schedule_name).await?;
    
    // Convert to DataFrame format expected by Python
    // This is a simplified version - you may want to flatten the structure more
    let mut sb_ids = Vec::new();
    let mut priorities = Vec::new();
    let mut ras = Vec::new();
    let mut decs = Vec::new();
    let mut scheduled_starts = Vec::new();
    let mut scheduled_stops = Vec::new();
    
    for block in &schedule.blocks {
        sb_ids.push(block.id.0);
        priorities.push(block.priority as f64);
        ras.push(block.target.ra().value());
        decs.push(block.target.dec().value());
        
        if let Some(period) = &block.scheduled_period {
            scheduled_starts.push(Some(period.start.value()));
            scheduled_stops.push(Some(period.stop.value()));
        } else {
            scheduled_starts.push(None);
            scheduled_stops.push(None);
        }
    }
    
    let df = DataFrame::new(vec![
        Series::new("scheduling_block_id", sb_ids),
        Series::new("priority", priorities),
        Series::new("ra_deg", ras),
        Series::new("dec_deg", decs),
        Series::new("scheduled_start_mjd", scheduled_starts),
        Series::new("scheduled_stop_mjd", scheduled_stops),
    ])
    .map_err(|e| format!("Failed to create DataFrame: {e}"))?;
    
    Ok(df)
}*/

/// Fetch dark periods for a schedule (public version for Python).
pub async fn fetch_dark_periods_public(
    schedule_id: Option<i64>,
) -> Result<Vec<(f64, f64)>, String> {
    if let Some(sid) = schedule_id {
        let pool = pool::get_pool()?;
        let mut conn = pool
            .get()
            .await
            .map_err(|e| format!("Failed to get connection: {e}"))?;
        
        let periods = fetch_dark_periods(&mut *conn, sid).await?;
        Ok(periods.into_iter().map(|p| (p.start.value(), p.stop.value())).collect())
    } else {
        // Global dark periods - fetch all unique periods across all schedules
        let pool = pool::get_pool()?;
        let mut conn = pool
            .get()
            .await
            .map_err(|e| format!("Failed to get connection: {e}"))?;
        
        let query = Query::new(
            r#"
            SELECT DISTINCT start_time_mjd, stop_time_mjd
            FROM dbo.schedule_dark_periods
            ORDER BY start_time_mjd
            "#,
        );
        
        let stream = query
            .query(&mut *conn)
            .await
            .map_err(|e| format!("Failed to fetch global dark periods: {e}"))?;
        
        let rows = stream
            .into_first_result()
            .await
            .map_err(|e| format!("Failed to read global dark periods: {e}"))?;
        
        let mut periods = Vec::new();
        for row in rows {
            let start: f64 = row.get::<f64, _>(0).ok_or_else(|| "start_time_mjd is NULL".to_string())?;
            let stop: f64 = row.get::<f64, _>(1).ok_or_else(|| "stop_time_mjd is NULL".to_string())?;
            periods.push((start, stop));
        }
        
        Ok(periods)
    }
}

/// Fetch visibility (possible) periods for a schedule.
pub async fn fetch_possible_periods(
    schedule_id: i64,
) -> Result<Vec<(i64, f64, f64)>, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;
    
    // Get all visibility periods for blocks in this schedule
    let query = Query::new(
        r#"
        SELECT 
            ssb.scheduling_block_id,
            vp.start_time_mjd,
            vp.stop_time_mjd
        FROM dbo.schedule_scheduling_blocks ssb
        JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
        JOIN dbo.visibility_periods vp ON sb.target_id = vp.target_id 
            AND sb.constraints_id = vp.constraints_id
        WHERE ssb.schedule_id = @P1
        ORDER BY ssb.scheduling_block_id, vp.start_time_mjd
        "#,
    );
    
    let mut q = query;
    q.bind(schedule_id);
    
    let stream = q
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch possible periods: {e}"))?;
    
    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read possible periods: {e}"))?;
    
    let mut periods = Vec::new();
    for row in rows {
        let sb_id: i64 = row.get::<i64, _>(0).ok_or_else(|| "scheduling_block_id is NULL".to_string())?;
        let start: f64 = row.get::<f64, _>(1).ok_or_else(|| "start_time_mjd is NULL".to_string())?;
        let stop: f64 = row.get::<f64, _>(2).ok_or_else(|| "stop_time_mjd is NULL".to_string())?;
        periods.push((sb_id, start, stop));
    }
    
    Ok(periods)
}
