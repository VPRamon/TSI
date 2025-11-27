use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tiberius::{Client, Config, AuthMethod};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

// ========================================
// JSON Data Structures
// ========================================

#[derive(Debug, Deserialize, Serialize)]
struct ScheduleData {
    #[serde(rename = "SchedulingBlock")]
    scheduling_blocks: Vec<SchedulingBlock>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SchedulingBlock {
    #[serde(rename = "schedulingBlockId")]
    scheduling_block_id: i64,
    priority: f64,
    target: Target,
    #[serde(rename = "schedulingBlockConfiguration_")]
    configuration: Configuration,
    #[serde(rename = "scheduled_period", skip_serializing_if = "Option::is_none")]
    scheduled_period: Option<Period>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Target {
    #[serde(rename = "id_")]
    id: i32,
    name: String,
    #[serde(rename = "position_")]
    position: Position,
}

#[derive(Debug, Deserialize, Serialize)]
struct Position {
    coord: Coordinate,
}

#[derive(Debug, Deserialize, Serialize)]
struct Coordinate {
    celestial: CelestialCoord,
}

#[derive(Debug, Deserialize, Serialize)]
struct CelestialCoord {
    #[serde(rename = "raInDeg")]
    ra_deg: f64,
    #[serde(rename = "decInDeg")]
    dec_deg: f64,
    #[serde(rename = "raProperMotionInMarcsecYear")]
    ra_pm_masyr: f64,
    #[serde(rename = "decProperMotionInMarcsecYear")]
    dec_pm_masyr: f64,
    equinox: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Configuration {
    #[serde(rename = "constraints_")]
    constraints: Constraints,
}

#[derive(Debug, Deserialize, Serialize)]
struct Constraints {
    #[serde(rename = "timeConstraint_")]
    time_constraint: TimeConstraint,
    #[serde(rename = "elevationConstraint_")]
    elevation_constraint: ElevationConstraint,
    #[serde(rename = "azimuthConstraint_")]
    azimuth_constraint: AzimuthConstraint,
}

#[derive(Debug, Deserialize, Serialize)]
struct TimeConstraint {
    #[serde(rename = "minObservationTimeInSec")]
    min_observation_sec: i32,
    #[serde(rename = "requestedDurationSec")]
    requested_duration_sec: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct ElevationConstraint {
    #[serde(rename = "minElevationAngleInDeg")]
    min_alt_deg: f64,
    #[serde(rename = "maxElevationAngleInDeg")]
    max_alt_deg: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct AzimuthConstraint {
    #[serde(rename = "minAzimuthAngleInDeg")]
    min_az_deg: f64,
    #[serde(rename = "maxAzimuthAngleInDeg")]
    max_az_deg: f64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Period {
    #[serde(rename = "durationInSec")]
    duration_sec: f64,
    #[serde(rename = "startTime")]
    start_time: TimeValue,
    #[serde(rename = "stopTime")]
    stop_time: TimeValue,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct TimeValue {
    format: String,
    scale: String,
    value: f64,
}

#[derive(Debug, Deserialize)]
struct PossiblePeriodsData {
    #[serde(rename = "SchedulingBlock")]
    scheduling_blocks: HashMap<String, Vec<Period>>,
}

// ========================================
// Database Helper Functions
// ========================================

async fn get_or_create_target(
    client: &mut Client<Compat<TcpStream>>,
    name: &str,
    ra_deg: f64,
    dec_deg: f64,
    ra_pm_masyr: f64,
    dec_pm_masyr: f64,
    equinox: f64,
) -> Result<i64> {
    // Try to find existing target
    let query = "SELECT target_id FROM dbo.targets 
                 WHERE ra_deg = @P1 AND dec_deg = @P2 
                 AND ra_pm_masyr = @P3 AND dec_pm_masyr = @P4 
                 AND equinox = @P5";
    
    let stream = client
        .query(query, &[&ra_deg, &dec_deg, &ra_pm_masyr, &dec_pm_masyr, &equinox])
        .await?;
    
    let rows = stream.into_first_result().await?;
    
    if let Some(row) = rows.first() {
        let target_id: i64 = row.get(0).unwrap();
        println!("Found existing target_id = {}", target_id);
        return Ok(target_id);
    }
    
    // Insert new target
    let insert_query = "INSERT INTO dbo.targets (name, ra_deg, dec_deg, ra_pm_masyr, dec_pm_masyr, equinox) 
                       OUTPUT inserted.target_id 
                       VALUES (@P1, @P2, @P3, @P4, @P5, @P6)";
    
    let stream = client
        .query(insert_query, &[&name, &ra_deg, &dec_deg, &ra_pm_masyr, &dec_pm_masyr, &equinox])
        .await?;
    
    let rows = stream.into_first_result().await?;
    let target_id: i64 = rows.first().unwrap().get(0).unwrap();
    println!("Created new target_id = {}", target_id);
    
    Ok(target_id)
}

async fn get_or_create_period(
    client: &mut Client<Compat<TcpStream>>,
    start_time_mjd: f64,
    stop_time_mjd: f64,
) -> Result<i64> {
    // Try to find existing period
    let query = "SELECT period_id FROM dbo.periods 
                 WHERE start_time_mjd = @P1 AND stop_time_mjd = @P2";
    
    let stream = client
        .query(query, &[&start_time_mjd, &stop_time_mjd])
        .await?;
    
    let rows = stream.into_first_result().await?;
    
    if let Some(row) = rows.first() {
        let period_id: i64 = row.get(0).unwrap();
        return Ok(period_id);
    }
    
    // Insert new period
    let insert_query = "INSERT INTO dbo.periods (start_time_mjd, stop_time_mjd) 
                       OUTPUT inserted.period_id 
                       VALUES (@P1, @P2)";
    
    let stream = client
        .query(insert_query, &[&start_time_mjd, &stop_time_mjd])
        .await?;
    
    let rows = stream.into_first_result().await?;
    let period_id: i64 = rows.first().unwrap().get(0).unwrap();
    
    Ok(period_id)
}

async fn get_or_create_altitude_constraint(
    client: &mut Client<Compat<TcpStream>>,
    min_alt_deg: f64,
    max_alt_deg: f64,
) -> Result<i64> {
    // Try to find existing constraint
    let query = "SELECT altitude_constraints_id FROM dbo.altitude_constraints 
                 WHERE min_alt_deg = @P1 AND max_alt_deg = @P2";
    
    let stream = client
        .query(query, &[&min_alt_deg, &max_alt_deg])
        .await?;
    
    let rows = stream.into_first_result().await?;
    
    if let Some(row) = rows.first() {
        let constraint_id: i64 = row.get(0).unwrap();
        return Ok(constraint_id);
    }
    
    // Insert new constraint
    let insert_query = "INSERT INTO dbo.altitude_constraints (min_alt_deg, max_alt_deg) 
                       OUTPUT inserted.altitude_constraints_id 
                       VALUES (@P1, @P2)";
    
    let stream = client
        .query(insert_query, &[&min_alt_deg, &max_alt_deg])
        .await?;
    
    let rows = stream.into_first_result().await?;
    let constraint_id: i64 = rows.first().unwrap().get(0).unwrap();
    
    Ok(constraint_id)
}

async fn get_or_create_azimuth_constraint(
    client: &mut Client<Compat<TcpStream>>,
    min_az_deg: f64,
    max_az_deg: f64,
) -> Result<i64> {
    // Try to find existing constraint
    let query = "SELECT azimuth_constraints_id FROM dbo.azimuth_constraints 
                 WHERE min_az_deg = @P1 AND max_az_deg = @P2";
    
    let stream = client
        .query(query, &[&min_az_deg, &max_az_deg])
        .await?;
    
    let rows = stream.into_first_result().await?;
    
    if let Some(row) = rows.first() {
        let constraint_id: i64 = row.get(0).unwrap();
        return Ok(constraint_id);
    }
    
    // Insert new constraint
    let insert_query = "INSERT INTO dbo.azimuth_constraints (min_az_deg, max_az_deg) 
                       OUTPUT inserted.azimuth_constraints_id 
                       VALUES (@P1, @P2)";
    
    let stream = client
        .query(insert_query, &[&min_az_deg, &max_az_deg])
        .await?;
    
    let rows = stream.into_first_result().await?;
    let constraint_id: i64 = rows.first().unwrap().get(0).unwrap();
    
    Ok(constraint_id)
}

async fn get_or_create_constraint(
    client: &mut Client<Compat<TcpStream>>,
    time_constraints_id: Option<i64>,
    altitude_constraints_id: Option<i64>,
    azimuth_constraints_id: Option<i64>,
) -> Result<i64> {
    // Build dynamic query based on which constraints are present
    let query = "SELECT constraints_id FROM dbo.constraints 
                 WHERE (time_constraints_id IS NULL AND @P1 IS NULL OR time_constraints_id = @P1)
                 AND (altitude_constraints_id IS NULL AND @P2 IS NULL OR altitude_constraints_id = @P2)
                 AND (azimuth_constraints_id IS NULL AND @P3 IS NULL OR azimuth_constraints_id = @P3)";
    
    let stream = client
        .query(query, &[&time_constraints_id, &altitude_constraints_id, &azimuth_constraints_id])
        .await?;
    
    let rows = stream.into_first_result().await?;
    
    if let Some(row) = rows.first() {
        let constraint_id: i64 = row.get(0).unwrap();
        return Ok(constraint_id);
    }
    
    // Insert new constraint
    let insert_query = "INSERT INTO dbo.constraints (time_constraints_id, altitude_constraints_id, azimuth_constraints_id) 
                       OUTPUT inserted.constraints_id 
                       VALUES (@P1, @P2, @P3)";
    
    let stream = client
        .query(insert_query, &[&time_constraints_id, &altitude_constraints_id, &azimuth_constraints_id])
        .await?;
    
    let rows = stream.into_first_result().await?;
    let constraint_id: i64 = rows.first().unwrap().get(0).unwrap();
    
    Ok(constraint_id)
}

async fn create_scheduling_block(
    client: &mut Client<Compat<TcpStream>>,
    target_id: i64,
    constraints_id: i64,
    priority: f64,
    min_observation_sec: i32,
    requested_duration_sec: i32,
) -> Result<i64> {
    let insert_query = "INSERT INTO dbo.scheduling_blocks 
                       (target_id, constraints_id, priority, min_observation_sec, requested_duration_sec) 
                       OUTPUT inserted.scheduling_block_id 
                       VALUES (@P1, @P2, @P3, @P4, @P5)";
    
    let priority_decimal = tiberius::numeric::Numeric::new_with_scale(
        (priority * 10.0) as i128,
        1,
    );
    
    let stream = client
        .query(
            insert_query,
            &[&target_id, &constraints_id, &priority_decimal, &min_observation_sec, &requested_duration_sec],
        )
        .await?;
    
    let rows = stream.into_first_result().await?;
    let scheduling_block_id: i64 = rows.first().unwrap().get(0).unwrap();
    
    Ok(scheduling_block_id)
}

async fn link_schedule_to_scheduling_block(
    client: &mut Client<Compat<TcpStream>>,
    schedule_id: i64,
    scheduling_block_id: i64,
    scheduled_period_id: Option<i64>,
) -> Result<()> {
    let insert_query = "INSERT INTO dbo.schedule_scheduling_blocks 
                       (schedule_id, scheduling_block_id, scheduled_period_id) 
                       VALUES (@P1, @P2, @P3)";
    
    client
        .execute(insert_query, &[&schedule_id, &scheduling_block_id, &scheduled_period_id])
        .await?;
    
    Ok(())
}

async fn add_visibility_period(
    client: &mut Client<Compat<TcpStream>>,
    schedule_id: i64,
    scheduling_block_id: i64,
    period_id: i64,
) -> Result<()> {
    let insert_query = "INSERT INTO dbo.visibility_periods 
                       (schedule_id, scheduling_block_id, period_id) 
                       VALUES (@P1, @P2, @P3)";
    
    client
        .execute(insert_query, &[&schedule_id, &scheduling_block_id, &period_id])
        .await?;
    
    Ok(())
}

// ========================================
// Main Upload Function
// ========================================

async fn upload_schedule(
    schedule_path: &str,
    possible_periods_path: &str,
    server: &str,
    database: &str,
    username: &str,
    password: &str,
) -> Result<i64> {
    // Read JSON files
    println!("Reading schedule from: {}", schedule_path);
    let schedule_json = std::fs::read_to_string(schedule_path)
        .context("Failed to read schedule.json")?;
    let schedule_data: ScheduleData = serde_json::from_str(&schedule_json)
        .context("Failed to parse schedule.json")?;
    
    println!("Reading possible periods from: {}", possible_periods_path);
    let periods_json = std::fs::read_to_string(possible_periods_path)
        .context("Failed to read possible_periods.json")?;
    let possible_periods: PossiblePeriodsData = serde_json::from_str(&periods_json)
        .context("Failed to parse possible_periods.json")?;
    
    // Configure database connection
    let mut config = Config::new();
    config.host(server);
    config.port(1433);
    config.database(database);
    config.authentication(AuthMethod::sql_server(username, password));
    config.trust_cert();
    config.encryption(tiberius::EncryptionLevel::Required);
    
    println!("Connecting to Azure SQL Database...");
    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;
    let mut client = Client::connect(config, tcp.compat_write()).await?;
    
    println!("Connected successfully!");
    
    // Create schedule entry
    let checksum = format!("schedule-{}", chrono::Utc::now().timestamp());
    let insert_schedule = "INSERT INTO dbo.schedules (checksum) 
                          OUTPUT inserted.schedule_id 
                          VALUES (@P1)";
    
    let stream = client.query(insert_schedule, &[&checksum.as_str()]).await?;
    let rows = stream.into_first_result().await?;
    let schedule_id: i64 = rows.first().unwrap().get(0).unwrap();
    println!("Created schedule_id = {}", schedule_id);
    
    // Process each scheduling block
    let total_blocks = schedule_data.scheduling_blocks.len();
    println!("Processing {} scheduling blocks...", total_blocks);
    
    for (idx, sb) in schedule_data.scheduling_blocks.iter().enumerate() {
        if idx % 100 == 0 {
            println!("Progress: {}/{}", idx, total_blocks);
        }
        
        // Get or create target
        let celestial = &sb.target.position.coord.celestial;
        let target_id = get_or_create_target(
            &mut client,
            &sb.target.name,
            celestial.ra_deg,
            celestial.dec_deg,
            celestial.ra_pm_masyr,
            celestial.dec_pm_masyr,
            celestial.equinox,
        ).await?;
        
        // Get or create altitude constraint
        let altitude_constraint_id = get_or_create_altitude_constraint(
            &mut client,
            sb.configuration.constraints.elevation_constraint.min_alt_deg,
            sb.configuration.constraints.elevation_constraint.max_alt_deg,
        ).await?;
        
        // Get or create azimuth constraint
        let azimuth_constraint_id = get_or_create_azimuth_constraint(
            &mut client,
            sb.configuration.constraints.azimuth_constraint.min_az_deg,
            sb.configuration.constraints.azimuth_constraint.max_az_deg,
        ).await?;
        
        // Get or create composite constraint
        let constraints_id = get_or_create_constraint(
            &mut client,
            None, // time_constraints_id
            Some(altitude_constraint_id),
            Some(azimuth_constraint_id),
        ).await?;
        
        // Create scheduling block
        let scheduling_block_id = create_scheduling_block(
            &mut client,
            target_id,
            constraints_id,
            sb.priority,
            sb.configuration.constraints.time_constraint.min_observation_sec,
            sb.configuration.constraints.time_constraint.requested_duration_sec,
        ).await?;
        
        // Get or create scheduled period if present
        let scheduled_period_id = if let Some(period) = &sb.scheduled_period {
            Some(get_or_create_period(
                &mut client,
                period.start_time.value,
                period.stop_time.value,
            ).await?)
        } else {
            None
        };
        
        // Link scheduling block to schedule
        link_schedule_to_scheduling_block(
            &mut client,
            schedule_id,
            scheduling_block_id,
            scheduled_period_id,
        ).await?;
        
        // Add visibility periods from possible_periods.json
        let sb_id_str = sb.scheduling_block_id.to_string();
        if let Some(periods) = possible_periods.scheduling_blocks.get(&sb_id_str) {
            for period in periods {
                let period_id = get_or_create_period(
                    &mut client,
                    period.start_time.value,
                    period.stop_time.value,
                ).await?;
                
                add_visibility_period(
                    &mut client,
                    schedule_id,
                    scheduling_block_id,
                    period_id,
                ).await?;
            }
        }
    }
    
    println!("Successfully uploaded schedule with {} scheduling blocks!", total_blocks);
    Ok(schedule_id)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Database credentials - read from environment or use defaults
    let server = std::env::var("DB_SERVER")
        .unwrap_or_else(|_| "tsi-upgrade.database.windows.net".to_string());
    let database = std::env::var("DB_DATABASE")
        .unwrap_or_else(|_| "db-schedules".to_string());
    let username = std::env::var("DB_USERNAME")
        .unwrap_or_else(|_| "ramon.valles@bootcamp-upgrade.com".to_string());
    let password = std::env::var("DB_PASSWORD")
        .expect("DB_PASSWORD environment variable must be set");
    
    // File paths - read from args or use defaults
    let args: Vec<String> = std::env::args().collect();
    let schedule_path = args.get(1)
        .map(|s| s.as_str())
        .unwrap_or("/workspace/data/schedule.json");
    let possible_periods_path = args.get(2)
        .map(|s| s.as_str())
        .unwrap_or("/workspace/data/possible_periods.json");
    
    println!("=== Schedule Upload Tool ===");
    println!("Server: {}", server);
    println!("Database: {}", database);
    println!("Username: {}", username);
    println!("Schedule file: {}", schedule_path);
    println!("Possible periods file: {}", possible_periods_path);
    println!();
    
    match upload_schedule(
        schedule_path,
        possible_periods_path,
        &server,
        &database,
        &username,
        &password,
    ).await {
        Ok(schedule_id) => {
            println!();
            println!("✓ Upload completed successfully!");
            println!("  Schedule ID: {}", schedule_id);
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ Upload failed: {}", e);
            Err(e)
        }
    }
}
