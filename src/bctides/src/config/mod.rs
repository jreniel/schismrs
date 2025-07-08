// schismrs-bctides/src/config/mod.rs

pub mod boundaries;

use crate::tides::SpaceVaryingTimeSeriesConfig;
use crate::tides::TidesConfig;
use crate::traits::*;
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

// =============================================================================
// INTERNAL BOUNDARY FORCING CONFIGURATION TYPES
// These are the types used internally by bctides - they implement Bctype
// =============================================================================

#[derive(Debug, Clone)]
pub enum ElevationBoundaryForcingConfig {
    UniformTimeSeries(BTreeMap<DateTime<Utc>, f64>),
    ConstantValue(f64),
    Tides(TidesConfig),
    SpaceVaryingTimeSeries(SpaceVaryingTimeSeriesConfig),
    TidesAndSpaceVaryingTimeSeries {
        tides: TidesConfig,
        time_series: SpaceVaryingTimeSeriesConfig,
    },
    EqualToZero,
}

impl Bctype for ElevationBoundaryForcingConfig {
    fn ibtype(&self) -> i8 {
        match *self {
            ElevationBoundaryForcingConfig::UniformTimeSeries(_) => 1,
            ElevationBoundaryForcingConfig::ConstantValue(_) => 2,
            ElevationBoundaryForcingConfig::Tides(_) => 3,
            ElevationBoundaryForcingConfig::SpaceVaryingTimeSeries(_) => 4,
            ElevationBoundaryForcingConfig::TidesAndSpaceVaryingTimeSeries { .. } => 5,
            ElevationBoundaryForcingConfig::EqualToZero => -1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VelocityBoundaryForcingConfig {
    UniformTimeSeries(BTreeMap<DateTime<Utc>, f64>),
    ConstantValue(f64),
    Tides(TidesConfig),
    SpaceVaryingTimeSeries(SpaceVaryingTimeSeriesConfig),
    TidesAndSpaceVaryingTimeSeries {
        tides: TidesConfig,
        time_series: SpaceVaryingTimeSeriesConfig,
    },
    Flather,
}

impl Bctype for VelocityBoundaryForcingConfig {
    fn ibtype(&self) -> i8 {
        match *self {
            VelocityBoundaryForcingConfig::UniformTimeSeries(_) => 1,
            VelocityBoundaryForcingConfig::ConstantValue(_) => 2,
            VelocityBoundaryForcingConfig::Tides(_) => 3,
            VelocityBoundaryForcingConfig::SpaceVaryingTimeSeries(_) => 4,
            VelocityBoundaryForcingConfig::TidesAndSpaceVaryingTimeSeries { .. } => 5,
            VelocityBoundaryForcingConfig::Flather => -1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TemperatureBoundaryForcingConfig {
    RelaxToUniformTimeSeries(BTreeMap<DateTime<Utc>, f64>),
    RelaxToConstantValue(f64),
    RelaxToInitialConditions,
    RelaxToSpaceVaryingTimeSeries(SpaceVaryingTimeSeriesConfig),
}

impl Bctype for TemperatureBoundaryForcingConfig {
    fn ibtype(&self) -> i8 {
        match *self {
            TemperatureBoundaryForcingConfig::RelaxToUniformTimeSeries(_) => 1,
            TemperatureBoundaryForcingConfig::RelaxToConstantValue(_) => 2,
            TemperatureBoundaryForcingConfig::RelaxToInitialConditions => 3,
            TemperatureBoundaryForcingConfig::RelaxToSpaceVaryingTimeSeries(_) => 4,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SalinityBoundaryForcingConfig {
    RelaxToUniformTimeSeries(BTreeMap<DateTime<Utc>, f64>),
    RelaxToConstantValue(f64),
    RelaxToInitialConditions,
    RelaxToSpaceVaryingTimeSeries(SpaceVaryingTimeSeriesConfig),
}

impl Bctype for SalinityBoundaryForcingConfig {
    fn ibtype(&self) -> i8 {
        match *self {
            SalinityBoundaryForcingConfig::RelaxToUniformTimeSeries(_) => 1,
            SalinityBoundaryForcingConfig::RelaxToConstantValue(_) => 2,
            SalinityBoundaryForcingConfig::RelaxToInitialConditions => 3,
            SalinityBoundaryForcingConfig::RelaxToSpaceVaryingTimeSeries(_) => 4,
        }
    }
}

// Re-export key types from boundaries module for easier access
pub use boundaries::{
    BoundariesConfig,
    OpenBoundaryForcings,
    OpenBoundaryForcingConfig,
    OpenBoundaryForcingConfigBuilder,
    ElevationForcingConfigInput,
    VelocityForcingConfigInput,
    TemperatureForcingConfigInput,
    SalinityForcingConfigInput,
    TidesConfigInput,
    ConstituentSelection,
    ConstituentPreset,
    ConstituentsConfigInput,
    LandBoundaryForcings,
    InteriorBoundaryForcings,
    OpenBoundaryForcingError,
};