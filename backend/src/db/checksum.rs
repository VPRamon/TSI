//! Checksum calculation for schedule deduplication.

use sha2::{Digest, Sha256};

/// Calculate SHA-256 checksum of schedule JSON content.
///
/// # Arguments
/// * `content` - JSON string content of the schedule
///
/// # Returns
/// Hexadecimal string representation of the SHA-256 hash.
pub fn calculate_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_consistency() {
        let content = r#"{"test": "data"}"#;
        let checksum1 = calculate_checksum(content);
        let checksum2 = calculate_checksum(content);
        assert_eq!(checksum1, checksum2);
    }

    #[test]
    fn test_different_content_different_checksum() {
        let content1 = r#"{"test": "data1"}"#;
        let content2 = r#"{"test": "data2"}"#;
        let checksum1 = calculate_checksum(content1);
        let checksum2 = calculate_checksum(content2);
        assert_ne!(checksum1, checksum2);
    }
}
