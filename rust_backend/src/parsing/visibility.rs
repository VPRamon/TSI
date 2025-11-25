use crate::core::domain::Period;
use siderust::astro::ModifiedJulianDate;

/// Parses a Python-style list of visibility periods into structured `Period` objects.
///
/// Accepts string representations of period tuples in the format:
/// `"[(start_mjd, stop_mjd), (start_mjd, stop_mjd), ...]"` where each MJD is a floating-point number.
///
/// # Arguments
///
/// * `visibility_str` - String representation of visibility periods
///
/// # Returns
///
/// * `Ok(Vec<Period>)` - Successfully parsed visibility periods
/// * `Err(String)` - Parse error description (currently not used; malformed tuples are silently ignored)
///
/// # Examples
///
/// ```
/// use tsi_rust::parsing::visibility::parse_visibility_string;
///
/// let periods = parse_visibility_string("[(59000.0, 59000.5), (59001.0, 59002.0)]")
///     .expect("Parse failed");
/// assert_eq!(periods.len(), 2);
/// assert_eq!(periods[0].duration_hours(), 12.0);
/// ```
///
/// # Edge Cases
///
/// - Empty strings and `"[]"` return an empty vector
/// - Malformed tuples (unparseable floats, wrong number of elements) are silently skipped
/// - Handles nested parentheses correctly
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

/// High-performance visibility period parser optimized for batch processing.
///
/// `VisibilityParser` provides both single and batch parsing capabilities for
/// visibility period strings, enabling efficient processing of large datasets.
///
/// # Examples
///
/// ```
/// use tsi_rust::parsing::VisibilityParser;
///
/// // Single parse
/// let periods = VisibilityParser::parse("[(59000.0, 59000.5)]")
///     .expect("Parse failed");
/// assert_eq!(periods.len(), 1);
///
/// // Batch parse
/// let inputs = vec!["[(59000.0, 59001.0)]", "[(59002.0, 59003.0)]"];
/// let results = VisibilityParser::parse_batch(&inputs);
/// assert_eq!(results.len(), 2);
/// ```
pub struct VisibilityParser;

impl VisibilityParser {
    /// Parses a single visibility string.
    ///
    /// This is a convenience wrapper around [`parse_visibility_string`].
    ///
    /// # Arguments
    ///
    /// * `visibility_str` - String representation of visibility periods
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Period>)` - Successfully parsed periods
    /// * `Err(String)` - Parse error description
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::parsing::VisibilityParser;
    ///
    /// let result = VisibilityParser::parse("[(59000.0, 59000.25)]")
    ///     .expect("Parse failed");
    /// assert_eq!(result[0].duration_hours(), 6.0);
    /// ```
    pub fn parse(visibility_str: &str) -> Result<Vec<Period>, String> {
        parse_visibility_string(visibility_str)
    }

    /// Parses multiple visibility strings in parallel for batch processing.
    ///
    /// Processes an array of visibility strings and returns results for each,
    /// allowing some parses to fail without affecting others.
    ///
    /// # Arguments
    ///
    /// * `visibility_strings` - Slice of string references to parse
    ///
    /// # Returns
    ///
    /// A vector of `Result<Vec<Period>, String>` with one entry per input string.
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::parsing::VisibilityParser;
    ///
    /// let inputs = vec![
    ///     "[(59000.0, 59001.0)]",
    ///     "[(59002.0, 59003.0)]",
    ///     "[]"  // Empty is valid
    /// ];
    /// let results = VisibilityParser::parse_batch(&inputs);
    ///
    /// assert_eq!(results.len(), 3);
    /// assert!(results[0].is_ok());
    /// assert!(results[2].as_ref().unwrap().is_empty());
    /// ```
    ///
    /// # Performance
    ///
    /// This method uses iterator mapping which can be parallelized by the compiler.
    /// For very large batches, consider using `rayon` for explicit parallelization.
    pub fn parse_batch(visibility_strings: &[&str]) -> Vec<Result<Vec<Period>, String>> {
        visibility_strings
            .iter()
            .map(|s| parse_visibility_string(s))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_visibility_string_variants() {
        assert!(parse_visibility_string("").unwrap().is_empty());

        let parsed = parse_visibility_string("[(1.0, 2.0), (3.5,4.5)]").unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].duration_hours(), 24.0);

        // Malformed tuple should be ignored gracefully
        let parsed_mixed = parse_visibility_string("[(1.0, 2.0), (bad, data)]").unwrap();
        assert_eq!(parsed_mixed.len(), 1);
    }

    #[test]
    fn visibility_parser_batch() {
        let inputs = vec!["[]", "[(0.0,1.0)]"];
        let results = VisibilityParser::parse_batch(&inputs);

        assert_eq!(results.len(), 2);
        assert!(results[0].as_ref().unwrap().is_empty());
        assert_eq!(results[1].as_ref().unwrap()[0].duration_hours(), 24.0);
    }
}
