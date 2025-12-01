//! Database operations for schedule validation results.
//!
//! This module handles inserting and querying validation results from the
//! `analytics.schedule_validation_results` table.

use log::{debug, info};
use tiberius::Query;

use super::pool;
use crate::services::validation::ValidationResult;

/// Insert validation results into the database
///
/// This function bulk-inserts validation results for a schedule into the
/// `analytics.schedule_validation_results` table.
///
/// # Arguments
/// * `results` - Vector of validation results to insert
///
/// # Returns
/// * `Ok(usize)` - Number of rows inserted
/// * `Err(String)` - Error description if the operation fails
pub async fn insert_validation_results(results: &[ValidationResult]) -> Result<usize, String> {
    if results.is_empty() {
        return Ok(0);
    }

    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let schedule_id = results[0].schedule_id;
    info!(
        "Inserting {} validation results for schedule_id={}",
        results.len(),
        schedule_id
    );

    // First, delete existing validation results for this schedule
    let delete_sql = "DELETE FROM analytics.schedule_validation_results WHERE schedule_id = @P1";
    let mut delete_query = Query::new(delete_sql);
    delete_query.bind(schedule_id);

    let delete_result = delete_query
        .execute(&mut *conn)
        .await
        .map_err(|e| format!("Failed to delete existing validation results: {e}"))?;

    let deleted_count = delete_result.rows_affected().iter().sum::<u64>();
    if deleted_count > 0 {
        debug!(
            "Deleted {} existing validation results for schedule_id={}",
            deleted_count, schedule_id
        );
    }

    // Bulk insert validation results
    let insert_sql = r#"
        INSERT INTO analytics.schedule_validation_results (
            schedule_id,
            scheduling_block_id,
            validation_status,
            issue_type,
            issue_category,
            criticality,
            field_name,
            current_value,
            expected_value,
            description
        ) VALUES (
            @P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10
        )
    "#;

    let mut inserted_count = 0;
    for result in results {
        let mut query = Query::new(insert_sql);
        query.bind(result.schedule_id);
        query.bind(result.scheduling_block_id);
        query.bind(result.status.as_str());
        query.bind(result.issue_type.as_deref());
        query.bind(result.issue_category.as_ref().map(|c| c.as_str()));
        query.bind(result.criticality.as_ref().map(|c| c.as_str()));
        query.bind(result.field_name.as_deref());
        query.bind(result.current_value.as_deref());
        query.bind(result.expected_value.as_deref());
        query.bind(result.description.as_deref());

        let exec_result = query
            .execute(&mut *conn)
            .await
            .map_err(|e| format!("Failed to insert validation result: {e}"))?;

        inserted_count += exec_result.rows_affected().iter().sum::<u64>();
    }

    info!(
        "Inserted {} validation results for schedule_id={}",
        inserted_count, schedule_id
    );

    Ok(inserted_count as usize)
}

/// Update validation_impossible flags in analytics.schedule_blocks_analytics
///
/// This function updates the validation_impossible column for blocks that were
/// marked as impossible by the validation rules. This allows analytics queries
/// to filter out impossible blocks without re-running validation.
///
/// # Arguments
/// * `schedule_id` - The schedule ID to update
///
/// # Returns
/// * `Ok(usize)` - Number of rows updated
/// * `Err(String)` - Error description if the operation fails
pub async fn update_validation_impossible_flags(schedule_id: i64) -> Result<usize, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    info!(
        "Updating validation_impossible flags for schedule_id={}",
        schedule_id
    );

    // Update analytics rows to mark blocks as impossible based on validation results
    let update_sql = r#"
        UPDATE analytics.schedule_blocks_analytics
        SET validation_impossible = 1
        WHERE schedule_id = @P1
          AND scheduling_block_id IN (
            SELECT scheduling_block_id
            FROM analytics.schedule_validation_results
            WHERE schedule_id = @P1 AND validation_status = 'impossible'
          )
    "#;

    let mut query = Query::new(update_sql);
    query.bind(schedule_id);
    query.bind(schedule_id);

    let result = query
        .execute(&mut *conn)
        .await
        .map_err(|e| format!("Failed to update validation_impossible flags: {e}"))?;

    let updated_count = result.rows_affected().iter().sum::<u64>() as usize;

    info!(
        "Updated {} blocks with validation_impossible=1 for schedule_id={}",
        updated_count, schedule_id
    );

    Ok(updated_count)
}

/// Fetch validation results for a schedule
///
/// Returns all validation results grouped by status and criticality.
///
/// # Arguments
/// * `schedule_id` - Schedule ID to fetch validation results for
///
/// # Returns
/// * `Ok(ValidationReportData)` - Validation report data
/// * `Err(String)` - Error description if the operation fails
pub async fn fetch_validation_results(schedule_id: i64) -> Result<ValidationReportData, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    debug!("Fetching validation results for schedule_id={}", schedule_id);

    // Fetch all validation results for this schedule, joining with analytics to get original_block_id
    let query_sql = r#"
        SELECT 
            vr.scheduling_block_id,
            vr.validation_status,
            vr.issue_type,
            vr.issue_category,
            vr.criticality,
            vr.field_name,
            vr.current_value,
            vr.expected_value,
            vr.description,
            a.original_block_id
        FROM analytics.schedule_validation_results vr
        LEFT JOIN analytics.schedule_blocks_analytics a 
            ON vr.schedule_id = a.schedule_id 
            AND vr.scheduling_block_id = a.scheduling_block_id
        WHERE vr.schedule_id = @P1
        ORDER BY vr.criticality DESC, vr.validation_status, vr.scheduling_block_id
    "#;

    let mut query = Query::new(query_sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to query validation results: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to fetch validation results: {e}"))?;

    let mut impossible_blocks = Vec::new();
    let mut validation_errors = Vec::new();
    let mut validation_warnings = Vec::new();
    let mut valid_count = 0;

    for row in rows {
        let scheduling_block_id: i64 = row.get(0).unwrap_or(0);
        let validation_status: &str = row.get(1).unwrap_or("valid");
        let issue_type: Option<&str> = row.get(2);
        let issue_category: Option<&str> = row.get(3);
        let criticality: Option<&str> = row.get(4);
        let field_name: Option<&str> = row.get(5);
        let current_value: Option<&str> = row.get(6);
        let expected_value: Option<&str> = row.get(7);
        let description: Option<&str> = row.get(8);
        let original_block_id: Option<&str> = row.get(9);

        match validation_status {
            "impossible" => {
                impossible_blocks.push(ValidationIssue {
                    block_id: scheduling_block_id,
                    original_block_id: original_block_id.map(|s| s.to_string()),
                    issue_type: issue_type.unwrap_or("Unknown").to_string(),
                    category: issue_category.unwrap_or("Unknown").to_string(),
                    criticality: criticality.unwrap_or("Critical").to_string(),
                    field_name: field_name.map(|s| s.to_string()),
                    current_value: current_value.map(|s| s.to_string()),
                    expected_value: expected_value.map(|s| s.to_string()),
                    description: description.unwrap_or("").to_string(),
                });
            }
            "error" => {
                validation_errors.push(ValidationIssue {
                    block_id: scheduling_block_id,
                    original_block_id: original_block_id.map(|s| s.to_string()),
                    issue_type: issue_type.unwrap_or("Unknown").to_string(),
                    category: issue_category.unwrap_or("Unknown").to_string(),
                    criticality: criticality.unwrap_or("High").to_string(),
                    field_name: field_name.map(|s| s.to_string()),
                    current_value: current_value.map(|s| s.to_string()),
                    expected_value: expected_value.map(|s| s.to_string()),
                    description: description.unwrap_or("").to_string(),
                });
            }
            "warning" => {
                validation_warnings.push(ValidationIssue {
                    block_id: scheduling_block_id,
                    original_block_id: original_block_id.map(|s| s.to_string()),
                    issue_type: issue_type.unwrap_or("Unknown").to_string(),
                    category: issue_category.unwrap_or("Unknown").to_string(),
                    criticality: criticality.unwrap_or("Medium").to_string(),
                    field_name: field_name.map(|s| s.to_string()),
                    current_value: current_value.map(|s| s.to_string()),
                    expected_value: expected_value.map(|s| s.to_string()),
                    description: description.unwrap_or("").to_string(),
                });
            }
            "valid" => {
                valid_count += 1;
            }
            _ => {}
        }
    }

    let total_blocks = valid_count
        + impossible_blocks.len()
        + validation_errors.len()
        + validation_warnings.len();

    Ok(ValidationReportData {
        schedule_id,
        total_blocks,
        valid_blocks: valid_count,
        impossible_blocks,
        validation_errors,
        validation_warnings,
    })
}

/// Check if validation results exist for a schedule
pub async fn has_validation_results(schedule_id: i64) -> Result<bool, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let query_sql =
        "SELECT COUNT(*) FROM analytics.schedule_validation_results WHERE schedule_id = @P1";
    let mut query = Query::new(query_sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to check validation results: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read validation count: {e}"))?;

    match row {
        Some(row) => {
            let count: i32 = row.get(0).unwrap_or(0);
            Ok(count > 0)
        }
        None => Ok(false),
    }
}

/// Delete validation results for a schedule
pub async fn delete_validation_results(schedule_id: i64) -> Result<u64, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let delete_sql = "DELETE FROM analytics.schedule_validation_results WHERE schedule_id = @P1";
    let mut query = Query::new(delete_sql);
    query.bind(schedule_id);

    let result = query
        .execute(&mut *conn)
        .await
        .map_err(|e| format!("Failed to delete validation results: {e}"))?;

    let deleted_count = result.rows_affected().iter().sum::<u64>();
    info!(
        "Deleted {} validation results for schedule_id={}",
        deleted_count, schedule_id
    );

    Ok(deleted_count)
}

// ============================================================================
// Data structures for validation report
// ============================================================================

/// A single validation issue (impossible block, error, or warning)
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub block_id: i64,
    pub original_block_id: Option<String>,
    pub issue_type: String,
    pub category: String,
    pub criticality: String,
    pub field_name: Option<String>,
    pub current_value: Option<String>,
    pub expected_value: Option<String>,
    pub description: String,
}

/// Validation report data for a schedule
#[derive(Debug, Clone)]
pub struct ValidationReportData {
    pub schedule_id: i64,
    pub total_blocks: usize,
    pub valid_blocks: usize,
    pub impossible_blocks: Vec<ValidationIssue>,
    pub validation_errors: Vec<ValidationIssue>,
    pub validation_warnings: Vec<ValidationIssue>,
}
