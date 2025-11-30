// schismrs/src/config/timestep.rs

use anyhow::Context;
use chrono::Duration;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Represents a SCHISM model timestep
///
/// The timestep must be positive and is internally stored as a chrono::Duration.
/// Can be deserialized from either:
/// - A float (interpreted as seconds): `timestep: 100.0`
/// - A string with units (parsed via humantime): `timestep: "2.5m"`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimestepConfig {
    duration: Duration,
}

impl TimestepConfig {
    /// Create a new TimestepConfig from a Duration
    ///
    /// Returns an error if the duration is negative or zero.
    pub fn new(duration: Duration) -> anyhow::Result<Self> {
        if duration <= Duration::zero() {
            anyhow::bail!(format!(
                "Timestep must be > 0., but got duration of {}",
                duration
            ));
        }

        Ok(Self { duration })
    }

    /// Create a Timestep from seconds (f64)
    pub fn from_seconds(seconds: f64) -> anyhow::Result<Self> {
        if seconds <= 0.0 {
            anyhow::bail!(format!("Timestep must be > 0., but got f64: {}", seconds));
        }

        // Convert float seconds to chrono::Duration
        let duration = Duration::milliseconds((seconds * 1000.0) as i64);
        Self::new(duration)
    }

    /// Create a TimestepConfig from a humantime-compatible string
    ///
    /// Examples: "100s", "2.5m", "1h"
    pub fn from_humantime_str(s: &str) -> anyhow::Result<Self> {
        // Parse using humantime (returns std::time::Duration)
        let std_duration = humantime::parse_duration(s)
            .context(format!("Error parsing duration from string: {:?}", s))?;

        // Convert std::time::Duration to chrono::Duration
        let chrono_duration = Duration::from_std(std_duration).context(format!(
            "Error converting std duration to chrono duration: {}",
            humantime::format_duration(std_duration)
        ))?;

        Self::new(chrono_duration)
    }

    /// Get the timestep as a chrono::Duration
    pub fn as_duration(&self) -> Duration {
        self.duration
    }

    /// Get the timestep as seconds (f64)
    pub fn as_secs_f64(&self) -> f64 {
        self.duration.num_milliseconds() as f64 / 1000.0
    }

    /// Get the timestep as seconds (i64)
    pub fn as_secs(&self) -> i64 {
        self.duration.num_seconds()
    }

    /// Validate timestep against typical SCHISM constraints
    ///
    /// This is a soft validation - returns warnings but doesn't fail.
    /// Typical SCHISM timesteps are 100-300 seconds for realistic models.
    pub fn validate_schism_range(&self) -> Vec<String> {
        let mut warnings = Vec::new();
        let secs = self.as_secs_f64();

        if secs < 75.0 {
            warnings.push(format!(
                "Timestep {} is very small (< 75s). This may cause slow model execution.",
                self
            ));
        }

        if secs > 300.0 {
            warnings.push(format!(
                "Timestep {} is large (> 300s). This may cause numerical instability.",
                self
            ));
        }

        warnings
    }
}

// Display implementation for user-friendly output
impl fmt::Display for TimestepConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}s", self.as_secs_f64())
    }
}

// Serde deserialization: accept both float and string
impl<'de> Deserialize<'de> for TimestepConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        // Try to deserialize as either f64 or String
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum TimestepValue {
            Float(f64),
            String(String),
        }

        let value = TimestepValue::deserialize(deserializer)?;

        match value {
            TimestepValue::Float(seconds) => {
                TimestepConfig::from_seconds(seconds).map_err(D::Error::custom)
            }
            TimestepValue::String(s) => {
                TimestepConfig::from_humantime_str(&s).map_err(D::Error::custom)
            }
        }
    }
}

// Serde serialization: always output as float (seconds)
impl Serialize for TimestepConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.as_secs_f64())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_from_seconds_positive() {
//         let ts = TimestepConfig::from_seconds(100.0).unwrap();
//         assert_eq!(ts.as_secs_f64(), 100.0);
//     }

//     #[test]
//     fn test_from_seconds_fractional() {
//         let ts = TimestepConfig::from_seconds(2.5).unwrap();
//         assert_eq!(ts.as_secs_f64(), 2.5);
//     }

//     #[test]
//     fn test_from_seconds_negative() {
//         let result = TimestepConfig::from_seconds(-10.0);
//         assert!(result.is_err());
//         assert!(matches!(result, Err(TimestepError::NonPositive)));
//     }

//     #[test]
//     fn test_from_seconds_zero() {
//         let result = TimestepConfig::from_seconds(0.0);
//         assert!(result.is_err());
//     }

//     #[test]
//     fn test_from_humantime_seconds() {
//         let ts = TimestepConfig::from_humantime_str("100s").unwrap();
//         assert_eq!(ts.as_secs(), 100);
//     }

//     #[test]
//     fn test_from_humantime_minutes() {
//         let ts = TimestepConfig::from_humantime_str("2.5m").unwrap();
//         assert_eq!(ts.as_secs(), 150);
//     }

//     #[test]
//     fn test_from_humantime_hours() {
//         let ts = TimestepConfig::from_humantime_str("1h").unwrap();
//         assert_eq!(ts.as_secs(), 3600);
//     }

//     #[test]
//     fn test_from_humantime_invalid() {
//         let result = TimestepConfig::from_humantime_str("invalid");
//         assert!(result.is_err());
//     }

//     #[test]
//     fn test_deserialize_float() {
//         let yaml = "100.0";
//         let ts: TimestepConfig = serde_yaml::from_str(yaml).unwrap();
//         assert_eq!(ts.as_secs_f64(), 100.0);
//     }

//     #[test]
//     fn test_deserialize_string() {
//         let yaml = "\"2.5m\"";
//         let ts: TimestepConfig = serde_yaml::from_str(yaml).unwrap();
//         assert_eq!(ts.as_secs(), 150);
//     }

//     #[test]
//     fn test_deserialize_negative_fails() {
//         let yaml = "-10.0";
//         let result: Result<TimestepConfig, _> = serde_yaml::from_str(yaml);
//         assert!(result.is_err());
//     }

//     #[test]
//     fn test_serialize() {
//         let ts = TimestepConfig::from_seconds(100.5).unwrap();
//         let yaml = serde_yaml::to_string(&ts).unwrap();
//         assert!(yaml.contains("100.5"));
//     }

//     #[test]
//     fn test_validate_schism_range_normal() {
//         let ts = TimestepConfig::from_seconds(150.0).unwrap();
//         let warnings = ts.validate_schism_range();
//         assert!(warnings.is_empty());
//     }

//     #[test]
//     fn test_validate_schism_range_too_small() {
//         let ts = TimestepConfig::from_seconds(0.5).unwrap();
//         let warnings = ts.validate_schism_range();
//         assert!(!warnings.is_empty());
//         assert!(warnings[0].contains("very small"));
//     }

//     #[test]
//     fn test_validate_schism_range_too_large() {
//         let ts = TimestepConfig::from_seconds(500.0).unwrap();
//         let warnings = ts.validate_schism_range();
//         assert!(!warnings.is_empty());
//         assert!(warnings[0].contains("large"));
//     }

//     #[test]
//     fn test_display() {
//         let ts = TimestepConfig::from_seconds(100.5).unwrap();
//         assert_eq!(format!("{}", ts), "100.5s");
//     }
// }
