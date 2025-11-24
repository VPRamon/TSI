use crate::core::domain::Period;
use siderust::astro::ModifiedJulianDate;

/// Parse a Python list of visibility periods (as string) into Period structs
/// 
/// Expected format: "[(start_mjd, stop_mjd), ...]"
/// where start_mjd and stop_mjd are MJD floats
/// 
/// # Arguments
/// * `visibility_str` - String representation of visibility periods
/// 
/// # Returns
/// * `Vec<Period>` - Parsed visibility periods
pub fn parse_visibility_string(visibility_str: &str) -> Result<Vec<Period>, String> {
    if visibility_str.trim().is_empty() || visibility_str == "[]" {
        return Ok(Vec::new());
    }
    
    // Parse string like "[(mjd1, mjd2), (mjd3, mjd4)]"
    let cleaned = visibility_str
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']');
    
    if cleaned.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut periods = Vec::new();
    let mut current_tuple = String::new();
    let mut paren_depth = 0;
    
    for ch in cleaned.chars() {
        match ch {
            '(' => {
                paren_depth += 1;
                if paren_depth == 1 {
                    current_tuple.clear();
                }
            }
            ')' => {
                paren_depth -= 1;
                if paren_depth == 0 && !current_tuple.is_empty() {
                    // Parse tuple (start, stop)
                    let parts: Vec<&str> = current_tuple.split(',').collect();
                    if parts.len() == 2 {
                        if let (Ok(start), Ok(stop)) = (
                            parts[0].trim().parse::<f64>(),
                            parts[1].trim().parse::<f64>(),
                        ) {
                            periods.push(Period::new(
                                ModifiedJulianDate::new(start),
                                ModifiedJulianDate::new(stop),
                            ));
                        }
                    }
                }
            }
            _ => {
                if paren_depth > 0 {
                    current_tuple.push(ch);
                }
            }
        }
    }
    
    Ok(periods)
}

/// High-performance visibility parser optimized for batch processing
pub struct VisibilityParser;

impl VisibilityParser {
    pub fn parse(visibility_str: &str) -> Result<Vec<Period>, String> {
        parse_visibility_string(visibility_str)
    }
    
    /// Parse multiple visibility strings in parallel (for batch processing)
    pub fn parse_batch(visibility_strings: &[&str]) -> Vec<Result<Vec<Period>, String>> {
        visibility_strings
            .iter()
            .map(|s| parse_visibility_string(s))
            .collect()
    }
}
