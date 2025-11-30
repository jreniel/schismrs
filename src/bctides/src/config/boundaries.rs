// schismrs-bctides/src/config/boundaries.rs

/* =============================================================================
 * WHY WE NEED TWO CONFIGURATION TYPES
 * =============================================================================
 * 
 * This file contains the INPUT configuration types that handle deserialization
 * from YAML/JSON/TOML files. These are separate from the INTERNAL configuration
 * types in mod.rs for the following important reasons:
 * 
 * 1. **Serialization Requirements vs Runtime Needs**:
 *    - Input types: Must implement Deserialize, handle various input formats,
 *      support serde attributes like #[serde(tag = "type")]
 *    - Internal types: Must implement Bctype trait for the bctides engine,
 *      optimized for runtime performance, no serde dependencies
 * 
 * 2. **Validation and Error Handling**:
 *    - Input types: Focus on parsing and basic validation during deserialization
 *    - Internal types: Focus on providing the correct ibtype() values for SCHISM
 *    - Conversion step allows comprehensive validation with proper error messages
 * 
 * 3. **API Stability**:
 *    - Input types: Can evolve to support new YAML/JSON formats without breaking
 *      existing internal logic
 *    - Internal types: Remain stable for the bctides computational engine
 * 
 * 4. **Flexibility in Input Formats**:
 *    - Input types: Support complex serde patterns like #[serde(untagged)],
 *      #[serde(flatten)], custom deserializers
 *    - Internal types: Simple, efficient enums focused on computational needs
 * 
 * 5. **Separation of Concerns**:
 *    - This file: Handles configuration parsing, deserialization, input validation
 *    - mod.rs: Handles the actual boundary forcing computation logic
 *    - types.rs: Handles the bctides engine that uses the internal types
 * 
 * While this creates some duplication, it provides clear separation between the
 * "configuration interface" and the "computational engine", making the codebase
 * more maintainable and allowing each part to evolve independently.
 * =============================================================================
 */

use crate::config::{
    ElevationBoundaryForcingConfig, SalinityBoundaryForcingConfig, 
    TemperatureBoundaryForcingConfig, VelocityBoundaryForcingConfig
};
use crate::tides::{
    ConstituentsConfig, SpaceVaryingTimeSeriesConfig, TidalDatabase, TidesConfig,
    TimeSeriesDatabase,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use thiserror::Error;

// =============================================================================
// BOUNDARY CONFIGURATION
// =============================================================================

#[derive(Debug, Deserialize, Clone, Default)]
pub struct BoundariesConfig {
    #[serde(default)]
    pub open: Option<OpenBoundaryForcings>,

    #[serde(default)]
    pub land: Option<LandBoundaryForcings>,

    #[serde(default)]
    pub interior: Option<InteriorBoundaryForcings>,
}

// =============================================================================
// OPEN BOUNDARY FORCINGS (Main focus for bctides)
// =============================================================================

/// Input configuration for open boundary forcings
/// 
/// Supports two input patterns in YAML/JSON:
/// 1. Global: Same config applied to all boundaries
/// 2. PerBoundary: Different config per boundary ID
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum OpenBoundaryForcings {
    /// Global configuration - same forcing for all open boundaries
    /// Example YAML:
    /// ```yaml
    /// boundaries:
    ///   open:
    ///     elevation:
    ///       type: tides
    ///       database: tpxo
    /// ```
    Global(OpenBoundaryForcingConfig),

    /// Per-boundary configuration - different forcing per boundary ID
    /// Example YAML:
    /// ```yaml
    /// boundaries:
    ///   open:
    ///     0:
    ///       elevation:
    ///         type: tides
    ///         database: tpxo
    ///     1:
    ///       elevation:
    ///         type: constant
    ///         value: 0.5
    /// ```
    PerBoundary(BTreeMap<u32, OpenBoundaryForcingConfig>),
}

impl OpenBoundaryForcings {
    /// Create a new OpenBoundaryForcings with global configuration
    pub fn global(config: OpenBoundaryForcingConfig) -> Self {
        OpenBoundaryForcings::Global(config)
    }

    /// Create a new OpenBoundaryForcings with per-boundary configuration
    pub fn per_boundary(configs: BTreeMap<u32, OpenBoundaryForcingConfig>) -> Self {
        OpenBoundaryForcings::PerBoundary(configs)
    }

    /// Builder method to add a boundary configuration
    pub fn add_boundary(mut self, boundary_id: u32, config: OpenBoundaryForcingConfig) -> Self {
        match self {
            OpenBoundaryForcings::PerBoundary(ref mut map) => {
                map.insert(boundary_id, config);
                self
            }
            OpenBoundaryForcings::Global(_global_config) => {
                let mut map = BTreeMap::new();
                map.insert(boundary_id, config);
                // Note: This discards the global config and switches to per-boundary mode
                OpenBoundaryForcings::PerBoundary(map)
            }
        }
    }

    /// Get configuration for a specific boundary
    pub fn get_config(&self, boundary_id: u32) -> Option<&OpenBoundaryForcingConfig> {
        match self {
            OpenBoundaryForcings::Global(config) => Some(config),
            OpenBoundaryForcings::PerBoundary(map) => map.get(&boundary_id),
        }
    }
}

/// Input configuration for a single open boundary
/// 
/// Contains the input forcing configuration types (*ForcingConfigInput) that
/// handle deserialization. These get converted to internal types during the
/// build process.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct OpenBoundaryForcingConfig {
    #[serde(default)]
    pub elevation: Option<ElevationForcingConfigInput>,

    #[serde(default)]
    pub velocity: Option<VelocityForcingConfigInput>,

    #[serde(default)]
    pub temperature: Option<TemperatureForcingConfigInput>,

    #[serde(default)]
    pub salinity: Option<SalinityForcingConfigInput>,
}

impl OpenBoundaryForcingConfig {
    /// Create a new builder for OpenBoundaryForcingConfig
    pub fn builder() -> OpenBoundaryForcingConfigBuilder {
        OpenBoundaryForcingConfigBuilder::new()
    }

    /// Set elevation forcing
    pub fn with_elevation(mut self, elevation: ElevationForcingConfigInput) -> Self {
        self.elevation = Some(elevation);
        self
    }

    /// Set velocity forcing
    pub fn with_velocity(mut self, velocity: VelocityForcingConfigInput) -> Self {
        self.velocity = Some(velocity);
        self
    }

    /// Set temperature forcing
    pub fn with_temperature(mut self, temperature: TemperatureForcingConfigInput) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set salinity forcing
    pub fn with_salinity(mut self, salinity: SalinityForcingConfigInput) -> Self {
        self.salinity = Some(salinity);
        self
    }
}

/// Builder for OpenBoundaryForcingConfig
#[derive(Debug, Default)]
pub struct OpenBoundaryForcingConfigBuilder {
    elevation: Option<ElevationForcingConfigInput>,
    velocity: Option<VelocityForcingConfigInput>,
    temperature: Option<TemperatureForcingConfigInput>,
    salinity: Option<SalinityForcingConfigInput>,
}

impl OpenBoundaryForcingConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn elevation(mut self, elevation: ElevationForcingConfigInput) -> Self {
        self.elevation = Some(elevation);
        self
    }

    pub fn velocity(mut self, velocity: VelocityForcingConfigInput) -> Self {
        self.velocity = Some(velocity);
        self
    }

    pub fn temperature(mut self, temperature: TemperatureForcingConfigInput) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn salinity(mut self, salinity: SalinityForcingConfigInput) -> Self {
        self.salinity = Some(salinity);
        self
    }

    pub fn build(self) -> OpenBoundaryForcingConfig {
        OpenBoundaryForcingConfig {
            elevation: self.elevation,
            velocity: self.velocity,
            temperature: self.temperature,
            salinity: self.salinity,
        }
    }
}

// =============================================================================
// FORCING INPUT CONFIGURATIONS (For Deserialization)
// 
// These are the INPUT types that handle deserialization from YAML/JSON.
// They get converted to the INTERNAL *BoundaryForcingConfig types during build.
// 
// Key differences from internal types:
// - Implement Deserialize with complex serde attributes
// - Handle various input formats and validation
// - Focus on configuration parsing, not runtime performance
// =============================================================================

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum ElevationForcingConfigInput {
    #[serde(rename = "uniform_time_series")]
    UniformTimeSeries {
        #[serde(deserialize_with = "deserialize_time_series")]
        data: BTreeMap<DateTime<Utc>, f64>,
    },

    #[serde(rename = "constant")]
    ConstantValue { value: f64 },

    #[serde(rename = "tides")]
    Tides {
        #[serde(flatten)]
        config: TidesConfigInput,
    },

    #[serde(rename = "space_varying_time_series")]
    SpaceVaryingTimeSeries { database: TimeSeriesDatabase },

    #[serde(rename = "tides_and_space_varying")]
    TidesAndSpaceVaryingTimeSeries {
        #[serde(flatten)]
        tides: TidesConfigInput,
        time_series: TimeSeriesDatabase,
    },

    #[serde(rename = "zero")]
    EqualToZero,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum VelocityForcingConfigInput {
    #[serde(rename = "uniform_time_series")]
    UniformTimeSeries {
        #[serde(deserialize_with = "deserialize_time_series")]
        data: BTreeMap<DateTime<Utc>, f64>,
    },

    #[serde(rename = "constant")]
    ConstantValue { value: f64 },

    #[serde(rename = "tides")]
    Tides {
        #[serde(flatten)]
        config: TidesConfigInput,
    },

    #[serde(rename = "space_varying_time_series")]
    SpaceVaryingTimeSeries { database: TimeSeriesDatabase },

    #[serde(rename = "tides_and_space_varying")]
    TidesAndSpaceVaryingTimeSeries {
        #[serde(flatten)]
        tides: TidesConfigInput,
        time_series: TimeSeriesDatabase,
    },

    #[serde(rename = "flather")]
    Flather,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum TemperatureForcingConfigInput {
    #[serde(rename = "relax_uniform_time_series")]
    RelaxToUniformTimeSeries {
        #[serde(deserialize_with = "deserialize_time_series")]
        data: BTreeMap<DateTime<Utc>, f64>,
    },

    #[serde(rename = "relax_constant")]
    RelaxToConstantValue { value: f64 },

    #[serde(rename = "relax_initial_conditions")]
    RelaxToInitialConditions,

    #[serde(rename = "relax_space_varying")]
    RelaxToSpaceVaryingTimeSeries { database: TimeSeriesDatabase },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum SalinityForcingConfigInput {
    #[serde(rename = "relax_uniform_time_series")]
    RelaxToUniformTimeSeries {
        #[serde(deserialize_with = "deserialize_time_series")]
        data: BTreeMap<DateTime<Utc>, f64>,
    },

    #[serde(rename = "relax_constant")]
    RelaxToConstantValue { value: f64 },

    #[serde(rename = "relax_initial_conditions")]
    RelaxToInitialConditions,

    #[serde(rename = "relax_space_varying")]
    RelaxToSpaceVaryingTimeSeries { database: TimeSeriesDatabase },
}

// =============================================================================
// TIDAL CONFIGURATION INPUT
// =============================================================================

#[derive(Debug, Deserialize, Clone)]
pub struct TidesConfigInput {
    pub database: TidalDatabase,

    #[serde(default)]
    pub constituents: ConstituentSelection,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ConstituentSelection {
    /// Predefined sets
    Preset(ConstituentPreset),

    /// Custom selection
    Custom {
        #[serde(flatten)]
        constituents: ConstituentsConfigInput,
    },
}

impl Default for ConstituentSelection {
    fn default() -> Self {
        ConstituentSelection::Preset(ConstituentPreset::Major)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ConstituentPreset {
    All,
    Major,
    Minor,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[allow(non_snake_case)]
pub struct ConstituentsConfigInput {
    #[serde(default)]
    pub Q1: bool,
    #[serde(default)]
    pub O1: bool,
    #[serde(default)]
    pub P1: bool,
    #[serde(default)]
    pub K1: bool,
    #[serde(default)]
    pub N2: bool,
    #[serde(default)]
    pub M2: bool,
    #[serde(default)]
    pub S2: bool,
    #[serde(default)]
    pub K2: bool,
    #[serde(default)]
    pub Mm: bool,
    #[serde(default)]
    pub Mf: bool,
    #[serde(default)]
    pub M4: bool,
    #[serde(default)]
    pub MN4: bool,
    #[serde(default)]
    pub MS4: bool,
    #[serde(default)]
    pub _2N2: bool,
    #[serde(default)]
    pub S1: bool,
}

// =============================================================================
// PLACEHOLDER FOR OTHER BOUNDARY TYPES
// =============================================================================

#[derive(Debug, Deserialize, Clone)]
pub struct LandBoundaryForcings {
    // Land boundaries typically don't have forcings, but keeping for completeness
    // Could be used for nudging zones or other purposes
}

#[derive(Debug, Deserialize, Clone)]
pub struct InteriorBoundaryForcings {
    // Interior boundaries might have special treatments
    // Could be used for sponge layers or relaxation zones
}

// =============================================================================
// CONVERSION IMPLEMENTATIONS
// Convert from INPUT types to INTERNAL BoundaryForcingConfig types
// 
// This is where the magic happens - we convert from the deserialization-focused
// input types to the computation-focused internal types. This allows each type
// system to be optimized for its specific purpose.
// =============================================================================

impl TryFrom<&ElevationForcingConfigInput> for ElevationBoundaryForcingConfig {
    type Error = OpenBoundaryForcingError;

    fn try_from(config: &ElevationForcingConfigInput) -> Result<Self, Self::Error> {
        match config {
            ElevationForcingConfigInput::UniformTimeSeries { data } => {
                Ok(ElevationBoundaryForcingConfig::UniformTimeSeries(data.clone()))
            }
            ElevationForcingConfigInput::ConstantValue { value } => {
                Ok(ElevationBoundaryForcingConfig::ConstantValue(*value))
            }
            ElevationForcingConfigInput::Tides { config } => {
                let tides_config = TidesConfig::try_from(config)?;
                Ok(ElevationBoundaryForcingConfig::Tides(tides_config))
            }
            ElevationForcingConfigInput::SpaceVaryingTimeSeries { database } => {
                let ts_config = SpaceVaryingTimeSeriesConfig {
                    database: database.clone(),
                };
                Ok(ElevationBoundaryForcingConfig::SpaceVaryingTimeSeries(ts_config))
            }
            ElevationForcingConfigInput::TidesAndSpaceVaryingTimeSeries { tides, time_series } => {
                let tides_config = TidesConfig::try_from(tides)?;
                let ts_config = SpaceVaryingTimeSeriesConfig {
                    database: time_series.clone(),
                };
                Ok(ElevationBoundaryForcingConfig::TidesAndSpaceVaryingTimeSeries {
                    tides: tides_config,
                    time_series: ts_config,
                })
            }
            ElevationForcingConfigInput::EqualToZero => Ok(ElevationBoundaryForcingConfig::EqualToZero),
        }
    }
}

impl TryFrom<&VelocityForcingConfigInput> for VelocityBoundaryForcingConfig {
    type Error = OpenBoundaryForcingError;

    fn try_from(config: &VelocityForcingConfigInput) -> Result<Self, Self::Error> {
        match config {
            VelocityForcingConfigInput::UniformTimeSeries { data } => {
                Ok(VelocityBoundaryForcingConfig::UniformTimeSeries(data.clone()))
            }
            VelocityForcingConfigInput::ConstantValue { value } => {
                Ok(VelocityBoundaryForcingConfig::ConstantValue(*value))
            }
            VelocityForcingConfigInput::Tides { config } => {
                let tides_config = TidesConfig::try_from(config)?;
                Ok(VelocityBoundaryForcingConfig::Tides(tides_config))
            }
            VelocityForcingConfigInput::SpaceVaryingTimeSeries { database } => {
                let ts_config = SpaceVaryingTimeSeriesConfig {
                    database: database.clone(),
                };
                Ok(VelocityBoundaryForcingConfig::SpaceVaryingTimeSeries(ts_config))
            }
            VelocityForcingConfigInput::TidesAndSpaceVaryingTimeSeries { tides, time_series } => {
                let tides_config = TidesConfig::try_from(tides)?;
                let ts_config = SpaceVaryingTimeSeriesConfig {
                    database: time_series.clone(),
                };
                Ok(VelocityBoundaryForcingConfig::TidesAndSpaceVaryingTimeSeries {
                    tides: tides_config,
                    time_series: ts_config,
                })
            }
            VelocityForcingConfigInput::Flather => Ok(VelocityBoundaryForcingConfig::Flather),
        }
    }
}

impl TryFrom<&TemperatureForcingConfigInput> for TemperatureBoundaryForcingConfig {
    type Error = OpenBoundaryForcingError;

    fn try_from(config: &TemperatureForcingConfigInput) -> Result<Self, Self::Error> {
        match config {
            TemperatureForcingConfigInput::RelaxToUniformTimeSeries { data } => {
                Ok(TemperatureBoundaryForcingConfig::RelaxToUniformTimeSeries(data.clone()))
            }
            TemperatureForcingConfigInput::RelaxToConstantValue { value } => {
                Ok(TemperatureBoundaryForcingConfig::RelaxToConstantValue(*value))
            }
            TemperatureForcingConfigInput::RelaxToInitialConditions => {
                Ok(TemperatureBoundaryForcingConfig::RelaxToInitialConditions)
            }
            TemperatureForcingConfigInput::RelaxToSpaceVaryingTimeSeries { database } => {
                let ts_config = SpaceVaryingTimeSeriesConfig {
                    database: database.clone(),
                };
                Ok(TemperatureBoundaryForcingConfig::RelaxToSpaceVaryingTimeSeries(ts_config))
            }
        }
    }
}

impl TryFrom<&SalinityForcingConfigInput> for SalinityBoundaryForcingConfig {
    type Error = OpenBoundaryForcingError;

    fn try_from(config: &SalinityForcingConfigInput) -> Result<Self, Self::Error> {
        match config {
            SalinityForcingConfigInput::RelaxToUniformTimeSeries { data } => {
                Ok(SalinityBoundaryForcingConfig::RelaxToUniformTimeSeries(data.clone()))
            }
            SalinityForcingConfigInput::RelaxToConstantValue { value } => {
                Ok(SalinityBoundaryForcingConfig::RelaxToConstantValue(*value))
            }
            SalinityForcingConfigInput::RelaxToInitialConditions => {
                Ok(SalinityBoundaryForcingConfig::RelaxToInitialConditions)
            }
            SalinityForcingConfigInput::RelaxToSpaceVaryingTimeSeries { database } => {
                let ts_config = SpaceVaryingTimeSeriesConfig {
                    database: database.clone(),
                };
                Ok(SalinityBoundaryForcingConfig::RelaxToSpaceVaryingTimeSeries(ts_config))
            }
        }
    }
}

impl TryFrom<&TidesConfigInput> for TidesConfig {
    type Error = OpenBoundaryForcingError;

    fn try_from(input: &TidesConfigInput) -> Result<Self, Self::Error> {
        let constituents = match &input.constituents {
            ConstituentSelection::Preset(preset) => match preset {
                ConstituentPreset::All => ConstituentsConfig::all(),
                ConstituentPreset::Major => ConstituentsConfig::major(),
                ConstituentPreset::Minor => ConstituentsConfig::minor(),
            },
            ConstituentSelection::Custom { constituents } => {
                ConstituentsConfig::try_from(constituents)?
            }
        };

        Ok(TidesConfig {
            constituents,
            database: input.database.clone(),
        })
    }
}

impl TryFrom<&ConstituentsConfigInput> for ConstituentsConfig {
    type Error = OpenBoundaryForcingError;

    fn try_from(input: &ConstituentsConfigInput) -> Result<Self, Self::Error> {
        let mut config = ConstituentsConfig::default();

        // Set each constituent based on input
        config.Q1 = input.Q1;
        config.O1 = input.O1;
        config.P1 = input.P1;
        config.K1 = input.K1;
        config.N2 = input.N2;
        config.M2 = input.M2;
        config.S2 = input.S2;
        config.K2 = input.K2;
        config.Mm = input.Mm;
        config.Mf = input.Mf;
        config.M4 = input.M4;
        config.MN4 = input.MN4;
        config.MS4 = input.MS4;
        config._2N2 = input._2N2;
        config.S1 = input.S1;

        Ok(config)
    }
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

fn deserialize_time_series<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<DateTime<Utc>, f64>, D::Error>
where
    D: Deserializer<'de>,
{
    use humantime::parse_rfc3339_weak;
    use serde::de::Error;

    // Deserialize as a map of string keys to f64 values
    let string_map: BTreeMap<String, f64> = BTreeMap::deserialize(deserializer)?;

    // Convert string keys to DateTime<Utc>
    let mut datetime_map = BTreeMap::new();
    for (date_str, value) in string_map {
        let datetime = parse_rfc3339_weak(&date_str)
            .map(DateTime::<Utc>::from)
            .map_err(|e| D::Error::custom(format!("Invalid datetime '{}': {}", date_str, e)))?;
        datetime_map.insert(datetime, value);
    }

    Ok(datetime_map)
}

// =============================================================================
// ERROR TYPES
// =============================================================================

#[derive(Error, Debug)]
pub enum OpenBoundaryForcingError {
    #[error("Invalid tidal configuration: {0}")]
    InvalidTidalConfig(String),

    #[error("Invalid constituent configuration: {0}")]
    InvalidConstituentConfig(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Invalid parameter value: {0}")]
    InvalidParameterValue(String),
}

// =============================================================================
// EXTENSIONS FOR EXISTING TYPES
// =============================================================================

// Add Deserialize to existing enum types
impl<'de> Deserialize<'de> for TidalDatabase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "tpxo" => Ok(TidalDatabase::TPXO),
            "hamtide" => Ok(TidalDatabase::HAMTIDE),
            "fes" => Ok(TidalDatabase::FES),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["tpxo", "hamtide", "fes"],
            )),
        }
    }
}

impl<'de> Deserialize<'de> for TimeSeriesDatabase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "hycom" => Ok(TimeSeriesDatabase::HYCOM),
            _ => Err(serde::de::Error::unknown_variant(&s, &["hycom"])),
        }
    }
}