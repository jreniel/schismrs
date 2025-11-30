// src/config/fingerprint.rs

// use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use twox_hash::XxHash64;

/// Compute a deterministic fingerprint string for any hashable type
///
/// Uses Rust's standard Hash trait for consistency across runs.
/// Returns a hex string representation of the hash.
pub fn config_fingerprint<T: Hash>(data: &T) -> String {
    let mut hasher = XxHash64::with_seed(0);
    data.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Macro to combine fingerprints from multiple config sections
///
/// Usage:
/// ```
/// let fingerprint = config_deps!(config, [timestep, forcings, outputs]);
/// ```
///
/// This creates a combined fingerprint by:
/// 1. Computing individual fingerprints for each field
/// 2. Sorting them (for deterministic order)
/// 3. Joining and hashing the combination
#[macro_export]
macro_rules! config_deps {
    ($config:expr, [$($field:ident),+ $(,)?]) => {{
        let mut parts: Vec<String> = vec![
            $($crate::config::fingerprint::config_fingerprint(&$config.$field)),+
        ];
        parts.sort();
        $crate::config::fingerprint::config_fingerprint(&parts.join("-"))
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_fingerprint_deterministic() {
        let data1 = "test data";
        let data2 = "test data";

        assert_eq!(
            config_fingerprint(&data1),
            config_fingerprint(&data2),
            "Same input should produce same fingerprint"
        );
    }

    #[test]
    fn test_config_fingerprint_different() {
        let data1 = "test data 1";
        let data2 = "test data 2";

        assert_ne!(
            config_fingerprint(&data1),
            config_fingerprint(&data2),
            "Different input should produce different fingerprints"
        );
    }

    #[test]
    fn test_config_fingerprint_hex_format() {
        let data = "test";
        let fp = config_fingerprint(&data);

        // Should be a valid hex string
        assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(!fp.is_empty());
    }
}
