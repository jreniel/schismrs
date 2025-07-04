use crate::tides::SpaceVaryingTimeSeriesConfig;
use crate::tides::TidesConfig;
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

pub trait Bctype {
    fn ibtype(&self) -> i8;
}

#[derive(Debug)]
pub enum ElevationConfig {
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

impl Bctype for ElevationConfig {
    fn ibtype(&self) -> i8 {
        match *self {
            ElevationConfig::UniformTimeSeries(_) => 1,
            ElevationConfig::ConstantValue(_) => 2,
            ElevationConfig::Tides(_) => 3,
            ElevationConfig::SpaceVaryingTimeSeries(_) => 4,
            ElevationConfig::TidesAndSpaceVaryingTimeSeries { .. } => 5,
            ElevationConfig::EqualToZero => -1,
        }
    }
}
#[derive(Debug)]
pub enum VelocityConfig {
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
impl Bctype for VelocityConfig {
    fn ibtype(&self) -> i8 {
        match *self {
            VelocityConfig::UniformTimeSeries(_) => 1,
            VelocityConfig::ConstantValue(_) => 2,
            VelocityConfig::Tides(_) => 3,
            VelocityConfig::SpaceVaryingTimeSeries(_) => 4,
            VelocityConfig::TidesAndSpaceVaryingTimeSeries { .. } => 5,
            VelocityConfig::Flather => -1,
        }
    }
}
#[derive(Debug)]
pub enum TemperatureConfig {
    RelaxToUniformTimeSeries(BTreeMap<DateTime<Utc>, f64>),
    RelaxToConstantValue(f64),
    RelaxToInitialConditions,
    RelaxToSpaceVaryingTimeSeries(SpaceVaryingTimeSeriesConfig),
}
impl Bctype for TemperatureConfig {
    fn ibtype(&self) -> i8 {
        match *self {
            TemperatureConfig::RelaxToUniformTimeSeries(_) => 1,
            TemperatureConfig::RelaxToConstantValue(_) => 2,
            TemperatureConfig::RelaxToInitialConditions => 3,
            TemperatureConfig::RelaxToSpaceVaryingTimeSeries(_) => 4,
        }
    }
}
#[derive(Debug)]
pub enum SalinityConfig {
    RelaxToUniformTimeSeries(BTreeMap<DateTime<Utc>, f64>),
    RelaxToConstantValue(f64),
    RelaxToInitialConditions,
    RelaxToSpaceVaryingTimeSeries(SpaceVaryingTimeSeriesConfig),
}
impl Bctype for SalinityConfig {
    fn ibtype(&self) -> i8 {
        match *self {
            SalinityConfig::RelaxToUniformTimeSeries(_) => 1,
            SalinityConfig::RelaxToConstantValue(_) => 2,
            SalinityConfig::RelaxToInitialConditions => 3,
            SalinityConfig::RelaxToSpaceVaryingTimeSeries(_) => 4,
        }
    }
}
