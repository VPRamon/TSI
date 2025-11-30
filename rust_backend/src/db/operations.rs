//! Database CRUD operations for schedules, dark periods, and visibility data (SQL Server).

use chrono::{DateTime, Utc};
use log::{debug, info};
use siderust::{
    astro::ModifiedJulianDate, coordinates::spherical::direction::ICRS, units::angular::Degrees,
    units::time::Seconds,
};
use std::collections::HashMap;
use tiberius::{numeric::Numeric, Query, Row};

use super::models::{
    Constraints, Period, Schedule, ScheduleId, ScheduleInfo, ScheduleMetadata, SchedulingBlock,
    SchedulingBlockId,
};
use super::pool;

type DbClient = tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct FloatKey(u64);

impl FloatKey {
    fn new(value: f64) -> Self {
        Self(value.to_bits())
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct TargetKey {
    ra: FloatKey,
    dec: FloatKey,
}

impl TargetKey {
    fn from_icrs(target: &ICRS) -> Self {
        Self {
            ra: FloatKey::new(target.ra().value()),
            dec: FloatKey::new(target.dec().value()),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct AltitudeKey {
    min: FloatKey,
    max: FloatKey,
}

impl AltitudeKey {
    fn new(min: f64, max: f64) -> Self {
        Self {
            min: FloatKey::new(min),
            max: FloatKey::new(max),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct AzimuthKey {
    min: FloatKey,
    max: FloatKey,
}

impl AzimuthKey {
    fn new(min: f64, max: f64) -> Self {
        Self {
            min: FloatKey::new(min),
            max: FloatKey::new(max),
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct ConstraintsKey {
    start: Option<FloatKey>,
    stop: Option<FloatKey>,
    altitude: AltitudeKey,
    azimuth: AzimuthKey,
}

impl ConstraintsKey {
    fn new(constraints: &Constraints) -> Self {
        let (start, stop) = if let Some(period) = &constraints.fixed_time {
            (
                Some(FloatKey::new(period.start.value())),
                Some(FloatKey::new(period.stop.value())),
            )
        } else {
            (None, None)
        };

        Self {
            start,
            stop,
            altitude: AltitudeKey::new(constraints.min_alt.value(), constraints.max_alt.value()),
            azimuth: AzimuthKey::new(constraints.min_az.value(), constraints.max_az.value()),
        }
    }
}

struct ScheduleInserter<'a> {
    conn: &'a mut DbClient,
    target_cache: HashMap<TargetKey, i64>,
    altitude_cache: HashMap<AltitudeKey, i64>,
    azimuth_cache: HashMap<AzimuthKey, i64>,
    constraints_cache: HashMap<ConstraintsKey, i64>,
}

impl<'a> ScheduleInserter<'a> {
    fn new(conn: &'a mut DbClient) -> Self {
        Self {
            conn,
            target_cache: HashMap::new(),
            altitude_cache: HashMap::new(),
            azimuth_cache: HashMap::new(),
            constraints_cache: HashMap::new(),
        }
    }

    async fn insert_schedule(&mut self, schedule: &Schedule) -> Result<ScheduleMetadata, String> {
        info!(
            "Uploading schedule '{}' ({} blocks, {} dark periods)",
            schedule.name,
            schedule.blocks.len(),
            schedule.dark_periods.len()
        );
        let (schedule_id, upload_timestamp) = self.insert_schedule_row(schedule).await?;

        // Batch process ALL scheduling blocks at once for maximum performance
        if !schedule.blocks.is_empty() {
            self.insert_scheduling_blocks_bulk(schedule_id, &schedule.blocks)
                .await?;
        }

        info!(
            "Finished uploading schedule '{}' as id {}",
            schedule.name, schedule_id
        );
        Ok(ScheduleMetadata {
            schedule_id: Some(schedule_id),
            schedule_name: schedule.name.clone(),
            upload_timestamp,
            checksum: schedule.checksum.clone(),
        })
    }

    async fn insert_schedule_row(
        &mut self,
        schedule: &Schedule,
    ) -> Result<(i64, DateTime<Utc>), String> {
        debug!(
            "Inserting schedule metadata row for '{}' (checksum {})",
            schedule.name, schedule.checksum
        );
        // Serialize dark periods to JSON
        let dark_periods_json = if schedule.dark_periods.is_empty() {
            None
        } else {
            let periods_array: Vec<serde_json::Value> = schedule
                .dark_periods
                .iter()
                .map(|p| {
                    serde_json::json!({
                        "start": p.start.value(),
                        "stop": p.stop.value()
                    })
                })
                .collect();
            Some(
                serde_json::to_string(&periods_array)
                    .map_err(|e| format!("Failed to serialize dark periods: {e}"))?,
            )
        };

        let mut insert = Query::new(
            r#"
            INSERT INTO dbo.schedules (schedule_name, checksum, dark_periods_json)
            OUTPUT inserted.schedule_id, inserted.upload_timestamp
            VALUES (@P1, @P2, @P3)
            "#,
        );
        insert.bind(&schedule.name);
        insert.bind(&schedule.checksum);
        insert.bind(dark_periods_json.as_deref());

        let stream = insert
            .query(&mut *self.conn)
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

        debug!(
            "Inserted schedule '{}' as id {} (uploaded at {})",
            schedule.name, schedule_id, upload_timestamp
        );
        Ok((schedule_id, upload_timestamp))
    }

    async fn insert_scheduling_blocks_bulk(
        &mut self,
        schedule_id: i64,
        blocks: &[SchedulingBlock],
    ) -> Result<(), String> {
        // Step 1: Batch create all unique targets using VALUES + MERGE
        let mut unique_targets: Vec<(String, f64, f64)> = Vec::new();
        for block in blocks {
            let target_name = format!("SB_{}", block.id.0);
            let ra = block.target.ra().value();
            let dec = block.target.dec().value();
            unique_targets.push((target_name, ra, dec));
        }
        self.batch_create_targets(&unique_targets).await?;

        // Step 2: Batch create all unique constraints
        let mut unique_constraints: Vec<&Constraints> = Vec::new();
        for block in blocks {
            unique_constraints.push(&block.constraints);
        }
        self.batch_create_constraints(&unique_constraints).await?;

        // Step 3: Bulk insert ALL scheduling blocks in one query
        self.bulk_insert_scheduling_blocks(schedule_id, blocks)
            .await?;

        Ok(())
    }

    async fn bulk_insert_scheduling_blocks(
        &mut self,
        schedule_id: i64,
        blocks: &[SchedulingBlock],
    ) -> Result<(), String> {
        if blocks.is_empty() {
            return Ok(());
        }

        debug!(
            "Preparing bulk insert for {} scheduling blocks (schedule_id={})",
            blocks.len(),
            schedule_id
        );

        // SQL Server parameter limit: 2100 params
        // Each scheduling block uses 6 params, so max = 2100/6 = 350
        // Use 300 for safety margin (1800 params)
        const BATCH_SIZE: usize = 300;

        for (chunk_index, chunk) in blocks.chunks(BATCH_SIZE).enumerate() {
            debug!(
                "Inserting scheduling block chunk {} containing {} blocks",
                chunk_index + 1,
                chunk.len()
            );
            // Build VALUES clause for scheduling blocks
            let mut values_clauses = Vec::new();
            let mut json_strings: Vec<Option<String>> = Vec::new();

            for (i, block) in chunk.iter().enumerate() {
                let target_key = TargetKey::from_icrs(&block.target);
                let _target_id = *self
                    .target_cache
                    .get(&target_key)
                    .ok_or_else(|| "Target not in cache after batch creation".to_string())?;

                let constraints_key = ConstraintsKey::new(&block.constraints);
                let _constraints_id = *self
                    .constraints_cache
                    .get(&constraints_key)
                    .ok_or_else(|| "Constraints not in cache after batch creation".to_string())?;

                // Serialize visibility periods to JSON
                let visibility_json = if block.visibility_periods.is_empty() {
                    None
                } else {
                    let periods_array: Vec<serde_json::Value> = block
                        .visibility_periods
                        .iter()
                        .map(|p| {
                            serde_json::json!({
                                "start": p.start.value(),
                                "stop": p.stop.value()
                            })
                        })
                        .collect();
                    Some(
                        serde_json::to_string(&periods_array)
                            .map_err(|e| format!("Failed to serialize visibility periods: {e}"))?,
                    )
                };
                json_strings.push(visibility_json);

                let base = i * 6;
                values_clauses.push(format!(
                    "(@P{}, @P{}, @P{}, @P{}, @P{}, @P{})",
                    base + 1,
                    base + 2,
                    base + 3,
                    base + 4,
                    base + 5,
                    base + 6
                ));
            }

            // Bulk INSERT with OUTPUT to get all scheduling_block_ids
            let sql = format!(
                r#"
                INSERT INTO dbo.scheduling_blocks 
                    (target_id, constraints_id, priority, min_observation_sec, requested_duration_sec, visibility_periods_json)
                OUTPUT inserted.scheduling_block_id
                VALUES {}
                "#,
                values_clauses.join(", ")
            );

            let mut insert = Query::new(sql);
            for (i, block) in chunk.iter().enumerate() {
                let target_key = TargetKey::from_icrs(&block.target);
                let target_id = *self.target_cache.get(&target_key).unwrap();

                let constraints_key = ConstraintsKey::new(&block.constraints);
                let constraints_id = *self.constraints_cache.get(&constraints_key).unwrap();

                insert.bind(target_id);
                insert.bind(constraints_id);
                insert.bind(Numeric::new_with_scale((block.priority * 10.0) as i128, 1));
                insert.bind(block.min_observation.value() as i32);
                insert.bind(block.requested_duration.value() as i32);
                insert.bind(json_strings[i].as_deref());
            }

            let stream = insert
                .query(&mut *self.conn)
                .await
                .map_err(|e| format!("Failed to bulk insert scheduling blocks: {e}"))?;

            let rows = stream
                .into_first_result()
                .await
                .map_err(|e| format!("Failed to read bulk insert results: {e}"))?;

            // Now link all blocks to schedule
            let mut sb_ids: Vec<i64> = Vec::new();
            for row in rows {
                let sb_id: i64 = row
                    .get::<i64, _>(0)
                    .ok_or_else(|| "scheduling_block_id is NULL".to_string())?;
                sb_ids.push(sb_id);
            }

            // Bulk insert into schedule_scheduling_blocks
            let mut link_values = Vec::new();
            let mut link_params = Vec::new();

            for (i, (block, sb_id)) in chunk.iter().zip(sb_ids.iter()).enumerate() {
                let (start_mjd, stop_mjd) = if let Some(period) = &block.scheduled_period {
                    (Some(period.start.value()), Some(period.stop.value()))
                } else {
                    (None, None)
                };

                let base = i * 4;
                link_values.push(format!(
                    "(@P{}, @P{}, @P{}, @P{})",
                    base + 1,
                    base + 2,
                    base + 3,
                    base + 4
                ));
                link_params.push((schedule_id, *sb_id, start_mjd, stop_mjd));
            }

            let link_sql = format!(
                "INSERT INTO dbo.schedule_scheduling_blocks (schedule_id, scheduling_block_id, start_time_mjd, stop_time_mjd) VALUES {}",
                link_values.join(", ")
            );

            let mut link_insert = Query::new(link_sql);
            for (sched_id, sb_id, start, stop) in link_params {
                link_insert.bind(sched_id);
                link_insert.bind(sb_id);
                link_insert.bind(start);
                link_insert.bind(stop);
            }

            link_insert
                .execute(&mut *self.conn)
                .await
                .map_err(|e| format!("Failed to bulk link scheduling blocks: {e}"))?;
        }

        Ok(())
    }

    async fn batch_create_targets(&mut self, targets: &[(String, f64, f64)]) -> Result<(), String> {
        if targets.is_empty() {
            return Ok(());
        }

        debug!(
            "Ensuring {} targets are present in the database",
            targets.len()
        );
        // Process targets one by one using the optimized get_or_create
        // This is safer than bulk MERGE and still uses the cache
        for (name, ra, dec) in targets {
            let target = ICRS::new(Degrees::new(*ra), Degrees::new(*dec));
            self.get_or_create_target(name, &target).await?;
        }

        Ok(())
    }

    async fn batch_create_constraints(
        &mut self,
        constraints_list: &[&Constraints],
    ) -> Result<(), String> {
        if constraints_list.is_empty() {
            return Ok(());
        }

        debug!(
            "Ensuring {} constraint sets are present in the database",
            constraints_list.len()
        );
        // First, create all unique altitude/azimuth constraints
        let mut unique_altitudes = std::collections::HashSet::new();
        let mut unique_azimuths = std::collections::HashSet::new();

        for constraints in constraints_list {
            let alt_key =
                AltitudeKey::new(constraints.min_alt.value(), constraints.max_alt.value());
            let az_key = AzimuthKey::new(constraints.min_az.value(), constraints.max_az.value());
            unique_altitudes.insert(alt_key);
            unique_azimuths.insert(az_key);
        }

        // Batch create altitude constraints
        for key in unique_altitudes {
            let FloatKey(min_bits) = key.min;
            let FloatKey(max_bits) = key.max;
            self.get_or_create_altitude_constraints(
                f64::from_bits(min_bits),
                f64::from_bits(max_bits),
            )
            .await?;
        }

        // Batch create azimuth constraints
        for key in unique_azimuths {
            let FloatKey(min_bits) = key.min;
            let FloatKey(max_bits) = key.max;
            self.get_or_create_azimuth_constraints(
                f64::from_bits(min_bits),
                f64::from_bits(max_bits),
            )
            .await?;
        }

        // Now create all composite constraints
        for constraints in constraints_list {
            self.get_or_create_constraints(constraints).await?;
        }

        Ok(())
    }

    async fn get_or_create_target(&mut self, name: &str, target: &ICRS) -> Result<i64, String> {
        let key = TargetKey::from_icrs(target);
        if let Some(id) = self.target_cache.get(&key) {
            return Ok(*id);
        }

        let ra_deg = target.ra().value();
        let dec_deg = target.dec().value();

        // Use MERGE for atomic get-or-create in one roundtrip
        let mut merge = Query::new(
            r#"
            MERGE dbo.targets AS target
            USING (SELECT @P1 AS ra_deg, @P2 AS dec_deg, @P3 AS name) AS source
            ON (target.ra_deg = source.ra_deg 
                AND target.dec_deg = source.dec_deg
                AND target.ra_pm_masyr = 0 
                AND target.dec_pm_masyr = 0 
                AND target.equinox = 2000.0)
            WHEN NOT MATCHED THEN
                INSERT (name, ra_deg, dec_deg, ra_pm_masyr, dec_pm_masyr, equinox)
                VALUES (source.name, source.ra_deg, source.dec_deg, 0, 0, 2000.0)
            OUTPUT inserted.target_id;
            "#,
        );
        merge.bind(ra_deg);
        merge.bind(dec_deg);
        merge.bind(name);

        let stream = merge
            .query(&mut *self.conn)
            .await
            .map_err(|e| format!("Failed to merge target: {e}"))?;

        let row = stream
            .into_row()
            .await
            .map_err(|e| format!("Failed to get target merge result: {e}"))?
            .ok_or_else(|| "No target_id returned from merge".to_string())?;

        let id = row
            .get::<i64, _>(0)
            .ok_or_else(|| "target_id is NULL".to_string())?;
        self.target_cache.insert(key, id);
        Ok(id)
    }
    async fn get_or_create_altitude_constraints(
        &mut self,
        min_alt: f64,
        max_alt: f64,
    ) -> Result<i64, String> {
        let key = AltitudeKey::new(min_alt, max_alt);
        if let Some(id) = self.altitude_cache.get(&key) {
            return Ok(*id);
        }

        // Use MERGE for atomic get-or-create
        let mut merge = Query::new(
            r#"
            MERGE dbo.altitude_constraints AS target
            USING (SELECT @P1 AS min_alt_deg, @P2 AS max_alt_deg) AS source
            ON (target.min_alt_deg = source.min_alt_deg AND target.max_alt_deg = source.max_alt_deg)
            WHEN NOT MATCHED THEN
                INSERT (min_alt_deg, max_alt_deg)
                VALUES (source.min_alt_deg, source.max_alt_deg)
            OUTPUT inserted.altitude_constraints_id;
            "#,
        );
        merge.bind(min_alt);
        merge.bind(max_alt);

        let stream = merge
            .query(&mut *self.conn)
            .await
            .map_err(|e| format!("Failed to merge altitude constraints: {e}"))?;

        let row = stream
            .into_row()
            .await
            .map_err(|e| format!("Failed to get altitude constraints result: {e}"))?
            .ok_or_else(|| "No altitude_constraints_id returned".to_string())?;

        let id = row
            .get::<i64, _>(0)
            .ok_or_else(|| "altitude_constraints_id is NULL".to_string())?;
        self.altitude_cache.insert(key, id);
        Ok(id)
    }
    async fn get_or_create_azimuth_constraints(
        &mut self,
        min_az: f64,
        max_az: f64,
    ) -> Result<i64, String> {
        let key = AzimuthKey::new(min_az, max_az);
        if let Some(id) = self.azimuth_cache.get(&key) {
            return Ok(*id);
        }

        // Use MERGE for atomic get-or-create
        let mut merge = Query::new(
            r#"
            MERGE dbo.azimuth_constraints AS target
            USING (SELECT @P1 AS min_az_deg, @P2 AS max_az_deg) AS source
            ON (target.min_az_deg = source.min_az_deg AND target.max_az_deg = source.max_az_deg)
            WHEN NOT MATCHED THEN
                INSERT (min_az_deg, max_az_deg)
                VALUES (source.min_az_deg, source.max_az_deg)
            OUTPUT inserted.azimuth_constraints_id;
            "#,
        );
        merge.bind(min_az);
        merge.bind(max_az);

        let stream = merge
            .query(&mut *self.conn)
            .await
            .map_err(|e| format!("Failed to merge azimuth constraints: {e}"))?;

        let row = stream
            .into_row()
            .await
            .map_err(|e| format!("Failed to get azimuth constraints result: {e}"))?
            .ok_or_else(|| "No azimuth_constraints_id returned".to_string())?;

        let id = row
            .get::<i64, _>(0)
            .ok_or_else(|| "azimuth_constraints_id is NULL".to_string())?;
        self.azimuth_cache.insert(key, id);
        Ok(id)
    }

    async fn get_or_create_constraints(
        &mut self,
        constraints: &Constraints,
    ) -> Result<i64, String> {
        let key = ConstraintsKey::new(constraints);
        if let Some(id) = self.constraints_cache.get(&key) {
            return Ok(*id);
        }

        let start_mjd = constraints
            .fixed_time
            .as_ref()
            .map(|period| period.start.value());
        let stop_mjd = constraints
            .fixed_time
            .as_ref()
            .map(|period| period.stop.value());

        let altitude_id = self
            .get_or_create_altitude_constraints(
                constraints.min_alt.value(),
                constraints.max_alt.value(),
            )
            .await?;
        let azimuth_id = self
            .get_or_create_azimuth_constraints(
                constraints.min_az.value(),
                constraints.max_az.value(),
            )
            .await?;

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
            .query(&mut *self.conn)
            .await
            .map_err(|e| format!("Failed to lookup constraints: {e}"))?;

        if let Some(row) = stream
            .into_row()
            .await
            .map_err(|e| format!("Failed to read constraints: {e}"))?
        {
            let id = row
                .get::<i64, _>(0)
                .ok_or_else(|| "constraints_id is NULL".to_string())?;
            self.constraints_cache.insert(key, id);
            return Ok(id);
        }

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
            .query(&mut *self.conn)
            .await
            .map_err(|e| format!("Failed to insert constraints: {e}"))?;

        let row = stream
            .into_row()
            .await
            .map_err(|e| format!("Failed to get constraints result: {e}"))?
            .ok_or_else(|| "No constraints_id returned".to_string())?;

        let id = row
            .get::<i64, _>(0)
            .ok_or_else(|| "constraints_id is NULL".to_string())?;
        self.constraints_cache.insert(key, id);
        Ok(id)
    }
}

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

pub async fn store_schedule(schedule: &Schedule) -> Result<ScheduleMetadata, String> {
    info!(
        "Received request to store schedule '{}' (checksum {}, {} blocks)",
        schedule.name,
        schedule.checksum,
        schedule.blocks.len()
    );
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
        let schedule_name: String = row.get::<&str, _>(1).unwrap_or_default().to_string();
        let upload_timestamp: DateTime<Utc> = row
            .get::<DateTime<Utc>, _>(2)
            .ok_or_else(|| "Existing upload_timestamp is NULL".to_string())?;
        let checksum: String = row.get::<&str, _>(3).unwrap_or_default().to_string();

        info!(
            "Schedule '{}' already present as id {} (upload timestamp: {})",
            schedule_name, schedule_id, upload_timestamp
        );
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
    //    Note: SQL Server uses implicit transactions, so all operations are
    //    automatically wrapped in a transaction and will rollback on error.
    //
    info!(
        "No existing schedule found for checksum {}. Proceeding with full insert",
        schedule.checksum
    );
    let metadata = insert_full_schedule(&mut *conn, schedule).await?;

    if let Some(schedule_id) = metadata.schedule_id {
        info!(
            "Successfully inserted schedule '{}' with id {} ({} blocks)",
            schedule.name,
            schedule_id,
            schedule.blocks.len()
        );
    } else {
        info!(
            "Successfully inserted schedule '{}' (id pending)",
            schedule.name
        );
    }
    Ok(metadata)
}

/// Insert a complete schedule with all dependent entities.
async fn insert_full_schedule(
    conn: &mut DbClient,
    schedule: &Schedule,
) -> Result<ScheduleMetadata, String> {
    let mut inserter = ScheduleInserter::new(conn);
    inserter.insert_schedule(schedule).await
}

/// Fetch a schedule from the database by ID or name.
pub async fn get_schedule(
    schedule_id: Option<i64>,
    schedule_name: Option<&str>,
) -> Result<Schedule, String> {
    if schedule_id.is_none() && schedule_name.is_none() {
        return Err("Must provide either schedule_id or schedule_name".to_string());
    }

    if let Some(id) = schedule_id {
        info!("Loading schedule by id {}", id);
    } else if let Some(name) = schedule_name {
        info!("Loading schedule '{}'", name);
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

    let db_schedule_id = metadata
        .schedule_id
        .ok_or_else(|| "Schedule has no ID".to_string())?;

    // 2. Fetch dark periods
    let dark_periods = fetch_dark_periods(&mut *conn, db_schedule_id).await?;

    // 3. Fetch all scheduling blocks for this schedule
    let blocks = fetch_scheduling_blocks(&mut *conn, db_schedule_id).await?;

    info!(
        "Loaded schedule '{}' (id {}) with {} blocks and {} dark periods",
        metadata.schedule_name,
        db_schedule_id,
        blocks.len(),
        dark_periods.len()
    );

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
    let schedule_name: String = row.get::<&str, _>(1).unwrap_or_default().to_string();
    let upload_timestamp: DateTime<Utc> = row
        .get::<DateTime<Utc>, _>(2)
        .ok_or_else(|| "upload_timestamp is NULL".to_string())?;
    let checksum: String = row.get::<&str, _>(3).unwrap_or_default().to_string();

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
    debug!("Fetching dark periods for schedule_id {}", schedule_id);
    let mut query = Query::new(
        r#"
        SELECT dark_periods_json
        FROM dbo.schedules
        WHERE schedule_id = @P1
        "#,
    );
    query.bind(schedule_id);

    let stream = query
        .query(conn)
        .await
        .map_err(|e| format!("Failed to fetch dark periods: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read dark periods: {e}"))?
        .ok_or_else(|| format!("Schedule {} not found", schedule_id))?;

    let json_str: Option<&str> = row.get(0);

    let mut periods = Vec::new();
    if let Some(json) = json_str {
        let periods_array: Vec<serde_json::Value> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse dark periods JSON: {e}"))?;

        for period_obj in periods_array {
            let start = period_obj["start"]
                .as_f64()
                .ok_or_else(|| "Invalid start value in dark period".to_string())?;
            let stop = period_obj["stop"]
                .as_f64()
                .ok_or_else(|| "Invalid stop value in dark period".to_string())?;

            if let Some(period) = Period::new(
                ModifiedJulianDate::new(start),
                ModifiedJulianDate::new(stop),
            ) {
                periods.push(period);
            }
        }
    }

    debug!(
        "Fetched {} dark periods for schedule_id {}",
        periods.len(),
        schedule_id
    );
    Ok(periods)
}

/// Fetch all scheduling blocks for a schedule.
async fn fetch_scheduling_blocks(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    schedule_id: i64,
) -> Result<Vec<SchedulingBlock>, String> {
    debug!("Fetching scheduling blocks for schedule_id {}", schedule_id);
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
        let sb_id: i64 = row
            .get::<i64, _>(0)
            .ok_or_else(|| "scheduling_block_id is NULL".to_string())?;
        let priority: Numeric = row
            .get::<Numeric, _>(1)
            .ok_or_else(|| "priority is NULL".to_string())?;
        let min_obs: i32 = row
            .get::<i32, _>(2)
            .ok_or_else(|| "min_observation_sec is NULL".to_string())?;
        let req_dur: i32 = row
            .get::<i32, _>(3)
            .ok_or_else(|| "requested_duration_sec is NULL".to_string())?;
        let ra: f64 = row
            .get::<f64, _>(4)
            .ok_or_else(|| "ra_deg is NULL".to_string())?;
        let dec: f64 = row
            .get::<f64, _>(5)
            .ok_or_else(|| "dec_deg is NULL".to_string())?;

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

    debug!(
        "Fetched {} scheduling blocks for schedule_id {}",
        blocks.len(),
        schedule_id
    );
    Ok(blocks)
}

/// Fetch visibility periods for a scheduling block.
async fn fetch_visibility_periods_for_block(
    conn: &mut tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>,
    sb_id: i64,
) -> Result<Vec<Period>, String> {
    let mut block_query = Query::new(
        r#"
        SELECT visibility_periods_json
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

    let json_str: Option<&str> = row.get(0);

    let mut periods = Vec::new();
    if let Some(json) = json_str {
        let periods_array: Vec<serde_json::Value> = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse visibility periods JSON: {e}"))?;

        for period_obj in periods_array {
            let start = period_obj["start"]
                .as_f64()
                .ok_or_else(|| "Invalid start value in visibility period".to_string())?;
            let stop = period_obj["stop"]
                .as_f64()
                .ok_or_else(|| "Invalid stop value in visibility period".to_string())?;

            if let Some(period) = Period::new(
                ModifiedJulianDate::new(start),
                ModifiedJulianDate::new(stop),
            ) {
                periods.push(period);
            }
        }
    }

    Ok(periods)
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
        let schedule_id: i64 = row
            .get::<i64, _>(0)
            .ok_or_else(|| "schedule_id is NULL".to_string())?;
        let schedule_name: String = row.get::<&str, _>(1).unwrap_or_default().to_string();
        let upload_timestamp: DateTime<Utc> = row
            .get::<DateTime<Utc>, _>(2)
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

    let sb_id: i64 = row
        .get::<i64, _>(0)
        .ok_or_else(|| "scheduling_block_id is NULL".to_string())?;
    let priority: Numeric = row
        .get::<Numeric, _>(1)
        .ok_or_else(|| "priority is NULL".to_string())?;
    let min_obs: i32 = row
        .get::<i32, _>(2)
        .ok_or_else(|| "min_observation_sec is NULL".to_string())?;
    let req_dur: i32 = row
        .get::<i32, _>(3)
        .ok_or_else(|| "requested_duration_sec is NULL".to_string())?;
    let ra: f64 = row
        .get::<f64, _>(4)
        .ok_or_else(|| "ra_deg is NULL".to_string())?;
    let dec: f64 = row
        .get::<f64, _>(5)
        .ok_or_else(|| "dec_deg is NULL".to_string())?;

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
        Ok(periods
            .into_iter()
            .map(|p| (p.start.value(), p.stop.value()))
            .collect())
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
            let start: f64 = row
                .get::<f64, _>(0)
                .ok_or_else(|| "start_time_mjd is NULL".to_string())?;
            let stop: f64 = row
                .get::<f64, _>(1)
                .ok_or_else(|| "stop_time_mjd is NULL".to_string())?;
            periods.push((start, stop));
        }

        Ok(periods)
    }
}

/// Fetch visibility (possible) periods for a schedule.
pub async fn fetch_possible_periods(schedule_id: i64) -> Result<Vec<(i64, f64, f64)>, String> {
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
        let sb_id: i64 = row
            .get::<i64, _>(0)
            .ok_or_else(|| "scheduling_block_id is NULL".to_string())?;
        let start: f64 = row
            .get::<f64, _>(1)
            .ok_or_else(|| "start_time_mjd is NULL".to_string())?;
        let stop: f64 = row
            .get::<f64, _>(2)
            .ok_or_else(|| "stop_time_mjd is NULL".to_string())?;
        periods.push((sb_id, start, stop));
    }

    Ok(periods)
}

/// Fetch lightweight sky map blocks for visualization.
/// This is optimized to fetch only the minimal data needed for the sky map page,
/// avoiding the overhead of loading full scheduling blocks with visibility periods.
pub async fn get_sky_map_blocks(
    schedule_id: Option<i64>,
    schedule_name: Option<&str>,
) -> Result<Vec<super::models::SkyMapBlock>, String> {
    use super::models::SkyMapBlock;

    if schedule_id.is_none() && schedule_name.is_none() {
        return Err("Either schedule_id or schedule_name must be provided".to_string());
    }

    // Try with flat periods first, fallback to joined periods if needed
    match get_sky_map_blocks_internal(schedule_id, schedule_name, true).await {
        Ok(blocks) => Ok(blocks),
        Err(e) => {
            let err_msg = e.to_string();
            let missing_flat_columns = err_msg.contains("Invalid column name")
                && (err_msg.contains("start_time_mjd") || err_msg.contains("stop_time_mjd"));

            if missing_flat_columns {
                get_sky_map_blocks_internal(schedule_id, schedule_name, false).await
            } else {
                Err(e)
            }
        }
    }
}

async fn get_sky_map_blocks_internal(
    schedule_id: Option<i64>,
    schedule_name: Option<&str>,
    use_flat_periods: bool,
) -> Result<Vec<super::models::SkyMapBlock>, String> {
    use super::models::SkyMapBlock;

    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let period_columns = if use_flat_periods {
        "ssb.start_time_mjd, ssb.stop_time_mjd"
    } else {
        "p.start_time_mjd, p.stop_time_mjd"
    };
    let period_join = if use_flat_periods {
        ""
    } else {
        "LEFT JOIN dbo.periods p ON ssb.scheduled_period_id = p.period_id"
    };

    let sql = if schedule_id.is_some() {
        format!(
            r#"
            SELECT 
                sb.scheduling_block_id,
                sb.priority,
                sb.requested_duration_sec,
                t.ra_deg,
                t.dec_deg,
                {period_columns}
            FROM dbo.schedule_scheduling_blocks ssb
            JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
            JOIN dbo.targets t ON sb.target_id = t.target_id
            {period_join}
            WHERE ssb.schedule_id = @P1
            ORDER BY sb.scheduling_block_id
            "#
        )
    } else {
        format!(
            r#"
            SELECT 
                sb.scheduling_block_id,
                sb.priority,
                sb.requested_duration_sec,
                t.ra_deg,
                t.dec_deg,
                {period_columns}
            FROM dbo.schedules s
            JOIN dbo.schedule_scheduling_blocks ssb ON s.schedule_id = ssb.schedule_id
            JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
            JOIN dbo.targets t ON sb.target_id = t.target_id
            {period_join}
            WHERE s.schedule_name = @P1
            ORDER BY sb.scheduling_block_id
            "#
        )
    };

    let mut query = Query::new(sql);
    if let Some(id) = schedule_id {
        query.bind(id);
    } else {
        query.bind(schedule_name.unwrap());
    }

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch sky map blocks: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read sky map blocks: {e}"))?;

    let mut blocks = Vec::with_capacity(rows.len());

    for row in rows {
        let id: i64 = row
            .get::<i64, _>(0)
            .ok_or_else(|| "scheduling_block_id is NULL".to_string())?;

        let priority_numeric: Numeric = row
            .get::<Numeric, _>(1)
            .ok_or_else(|| "priority is NULL".to_string())?;
        let priority = priority_numeric
            .value()
            .to_string()
            .parse::<f64>()
            .map_err(|e| format!("Failed to parse priority: {e}"))?;

        let requested_duration: i32 = row
            .get::<i32, _>(2)
            .ok_or_else(|| "requested_duration_sec is NULL".to_string())?;

        let ra: f64 = row
            .get::<f64, _>(3)
            .ok_or_else(|| "ra_deg is NULL".to_string())?;

        let dec: f64 = row
            .get::<f64, _>(4)
            .ok_or_else(|| "dec_deg is NULL".to_string())?;

        // Handle optional scheduled period
        let scheduled_period = match (row.get::<f64, _>(5), row.get::<f64, _>(6)) {
            (Some(start_mjd), Some(stop_mjd)) => {
                super::models::Period::new(
                    ModifiedJulianDate::new(start_mjd),
                    ModifiedJulianDate::new(stop_mjd),
                )
            }
            _ => None,
        };

        blocks.push(SkyMapBlock {
            id: super::models::SchedulingBlockId(id),
            priority,
            priority_bin: String::new(), // Will be computed in compute_sky_map_data
            requested_duration_seconds: requested_duration as f64,
            target_ra_deg: ra,
            target_dec_deg: dec,
            scheduled_period,
        });
    }

    Ok(blocks)
}

/// Compute sky map data with priority bins and metadata.
/// This function takes the raw blocks and computes everything needed for visualization.
pub fn compute_sky_map_data(
    blocks: Vec<super::models::SkyMapBlock>,
) -> Result<super::models::SkyMapData, String> {
    use super::models::{PriorityBinInfo, SkyMapData};

    if blocks.is_empty() {
        return Ok(SkyMapData {
            blocks: vec![],
            priority_bins: vec![],
            priority_min: 0.0,
            priority_max: 10.0,
            ra_min: 0.0,
            ra_max: 360.0,
            dec_min: -90.0,
            dec_max: 90.0,
            total_count: 0,
            scheduled_count: 0,
            scheduled_time_min: None,
            scheduled_time_max: None,
        });
    }

    // Compute statistics
    let mut priority_min = f64::MAX;
    let mut priority_max = f64::MIN;
    let mut ra_min = f64::MAX;
    let mut ra_max = f64::MIN;
    let mut dec_min = f64::MAX;
    let mut dec_max = f64::MIN;
    let mut scheduled_count = 0;
    let mut scheduled_time_min: Option<f64> = None;
    let mut scheduled_time_max: Option<f64> = None;

    for block in &blocks {
        priority_min = priority_min.min(block.priority);
        priority_max = priority_max.max(block.priority);
        ra_min = ra_min.min(block.target_ra_deg);
        ra_max = ra_max.max(block.target_ra_deg);
        dec_min = dec_min.min(block.target_dec_deg);
        dec_max = dec_max.max(block.target_dec_deg);

        if let Some(period) = &block.scheduled_period {
            scheduled_count += 1;
            let start_mjd = period.start.value();
            scheduled_time_min = Some(scheduled_time_min.map_or(start_mjd, |v| v.min(start_mjd)));
            scheduled_time_max = Some(scheduled_time_max.map_or(start_mjd, |v| v.max(start_mjd)));
        }
    }

    // Ensure priority range is valid
    if priority_min == priority_max {
        priority_max = priority_min + 1.0;
    }

    // Compute 4 priority bins proportional to min/max values
    let bin_count = 4;
    let bin_width = (priority_max - priority_min) / bin_count as f64;
    
    // Define colors for the 4 bins (from low to high priority)
    let bin_colors = vec![
        "#2ca02c".to_string(), // Green - Low priority
        "#1f77b4".to_string(), // Blue - Medium-low priority
        "#ff7f0e".to_string(), // Orange - Medium-high priority
        "#d62728".to_string(), // Red - High priority
    ];

    let mut priority_bins = Vec::with_capacity(bin_count);
    for i in 0..bin_count {
        let bin_min = priority_min + (i as f64 * bin_width);
        let bin_max = if i == bin_count - 1 {
            priority_max
        } else {
            priority_min + ((i + 1) as f64 * bin_width)
        };

        let label = format!("Bin {} [{:.1}-{:.1}]", i + 1, bin_min, bin_max);
        
        priority_bins.push(PriorityBinInfo {
            label,
            min_priority: bin_min,
            max_priority: bin_max,
            color: bin_colors[i].clone(),
        });
    }

    // Assign computed bins to blocks
    let total_count = blocks.len();
    let mut blocks_with_bins = blocks;
    for block in &mut blocks_with_bins {
        // Find which bin this block belongs to
        let priority = block.priority;
        let bin_index = if priority >= priority_max {
            bin_count - 1
        } else {
            ((priority - priority_min) / bin_width).floor() as usize
        };
        block.priority_bin = format!("Bin {} [{:.1}-{:.1}]", 
            bin_index + 1, 
            priority_bins[bin_index].min_priority, 
            priority_bins[bin_index].max_priority
        );
    }

    Ok(SkyMapData {
        blocks: blocks_with_bins,
        priority_bins,
        priority_min,
        priority_max,
        ra_min,
        ra_max,
        dec_min,
        dec_max,
        total_count,
        scheduled_count,
        scheduled_time_min,
        scheduled_time_max,
    })
}

/// Get complete sky map data with computed bins and metadata.
/// This is the main entry point for the sky map feature.
pub async fn get_sky_map_data(
    schedule_id: Option<i64>,
    schedule_name: Option<&str>,
) -> Result<super::models::SkyMapData, String> {
    let blocks = get_sky_map_blocks(schedule_id, schedule_name).await?;
    compute_sky_map_data(blocks)
}
