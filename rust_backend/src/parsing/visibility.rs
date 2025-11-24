use crate::core::domain::Period;
use crate::time::mjd::parse_visibility_string;

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

#[cfg(all(test, not(feature = "extension-module")))]
mod tests {
    use super::*;
    
    #[test]
    fn test_visibility_parser() {
        let input = "[(59580.0, 59581.0)]";
        let periods = VisibilityParser::parse(input).unwrap();
        assert_eq!(periods.len(), 1);
    }
    
    #[test]
    fn test_batch_parsing() {
        let inputs = vec![
            "[(59580.0, 59581.0)]",
            "[(59582.0, 59583.0), (59584.0, 59585.0)]",
            "[]",
        ];
        
        let results = VisibilityParser::parse_batch(&inputs);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].as_ref().unwrap().len(), 1);
        assert_eq!(results[1].as_ref().unwrap().len(), 2);
        assert_eq!(results[2].as_ref().unwrap().len(), 0);
    }
}
