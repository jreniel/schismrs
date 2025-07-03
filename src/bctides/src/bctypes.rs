use crate::tides::SpaceVaryingTimeSeriesConfig;
use crate::tides::TidesConfig;
use chrono::{DateTime, Utc};
use linked_hash_map::LinkedHashMap;
use linked_hash_set::LinkedHashSet;
use ndarray::Array2;
use std::collections::BTreeMap;

pub trait Bctype {
    fn ibtype(&self) -> i8;
    fn get_boundary_string(
        &self,
        coords: &Array2<f64>,
        afc: &LinkedHashSet<String>,
    ) -> Option<String>;
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
}

impl ElevationConfig {
    fn get_tidal_boundary_string(
        tides_config: &TidesConfig,
        coords: &Array2<f64>,
        afc: &LinkedHashSet<String>,
    ) -> String {
        let mut output = Vec::<String>::new();
        for constituent in afc {
            output.push(constituent.to_string());
            match tides_config.constituents.get_value_by_name(constituent) {
                //
                Some(_) => {
                    for value in tides_config
                        .database
                        .interpolate("elevation", constituent, coords)
                    {
                        output.push(format!("{}", value));
                    }
                }
                None => {}
            }
            // let bnd_elev = tides_config.get_constituent_string(constituent, coords);
            // for row in bnd_elev {
            //     output.push("{}", row);
            // }
        }
        output.join("\n")
    }
}

impl Bctype for ElevationConfig {
    fn ibtype(&self) -> i8 {
        match self {
            ElevationConfig::UniformTimeSeries(_) => 1,
            ElevationConfig::ConstantValue(_) => 2,
            ElevationConfig::Tides(_) => 3,
            ElevationConfig::SpaceVaryingTimeSeries(_) => 4,
            ElevationConfig::TidesAndSpaceVaryingTimeSeries { .. } => 5,
        }
    }
    fn get_boundary_string(
        &self,
        coords: &Array2<f64>,
        afc: &LinkedHashSet<String>,
    ) -> Option<String> {
        match self {
            ElevationConfig::UniformTimeSeries(_) => None,
            ElevationConfig::ConstantValue(value) => Some(format!("{}", value)),
            ElevationConfig::Tides(tides_config) => {
                Some(Self::get_tidal_boundary_string(tides_config, coords, afc))
            }
            ElevationConfig::SpaceVaryingTimeSeries(_) => None,
            ElevationConfig::TidesAndSpaceVaryingTimeSeries {
                tides,
                time_series: _,
            } => Some(Self::get_tidal_boundary_string(tides, coords, afc)),
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
impl VelocityConfig {
    fn get_tidal_boundary_string(
        tides_config: &TidesConfig,
        coords: &Array2<f64>,
        afc: &LinkedHashSet<String>,
    ) -> String {
        unimplemented!();
    }
}
impl Bctype for VelocityConfig {
    fn ibtype(&self) -> i8 {
        match self {
            VelocityConfig::UniformTimeSeries(_) => 1,
            VelocityConfig::ConstantValue(_) => 2,
            VelocityConfig::Tides(_) => 3,
            VelocityConfig::SpaceVaryingTimeSeries(_) => 4,
            VelocityConfig::TidesAndSpaceVaryingTimeSeries { .. } => 5,
            VelocityConfig::Flather => -1,
        }
    }
    fn get_boundary_string(
        &self,
        coords: &Array2<f64>,
        afc: &LinkedHashSet<String>,
    ) -> Option<String> {
        match self {
            VelocityConfig::UniformTimeSeries(_) => None,
            VelocityConfig::ConstantValue(value) => Some(format!("{}", value)),
            VelocityConfig::Tides(tides_config) => {
                Some(Self::get_tidal_boundary_string(tides_config, coords, afc))
            }
            VelocityConfig::SpaceVaryingTimeSeries(_) => None,
            VelocityConfig::TidesAndSpaceVaryingTimeSeries {
                tides,
                time_series: _,
            } => Some(Self::get_tidal_boundary_string(tides, coords, afc)),
            VelocityConfig::Flather => {
                unimplemented!("flather velocity is unimplemented")
                // 'eta_mean' !comment only - mean elevation below
                //         for i=1,nond(j) !loop over all nodes
                //             eta_m0(i) !mean elev at each node
                //         end for i
                //         'vn_mean'!comment only - mean normal velocity below
                //         for i=1,nond(j)
                //             qthcon(1:Nz,i,j) !mean normal velocity at the node (at all levels)
                //         end for i
            }
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
        match self {
            TemperatureConfig::RelaxToUniformTimeSeries(_) => 1,
            TemperatureConfig::RelaxToConstantValue(_) => 2,
            TemperatureConfig::RelaxToInitialConditions => 3,
            TemperatureConfig::RelaxToSpaceVaryingTimeSeries(_) => 4,
        }
    }
    fn get_boundary_string(
        &self,
        coords: &Array2<f64>,
        afc: &LinkedHashSet<String>,
    ) -> Option<String> {
        match self {
            TemperatureConfig::RelaxToUniformTimeSeries(_) => None,
            TemperatureConfig::RelaxToConstantValue(value) => Some(format!("{}", value)),
            TemperatureConfig::RelaxToInitialConditions => {
                unimplemented!("TemperatureConfig.get_boundary_string()->RelaxToInitialConditions");
            }
            TemperatureConfig::RelaxToSpaceVaryingTimeSeries(_) => None,
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
        match self {
            SalinityConfig::RelaxToUniformTimeSeries(_) => 1,
            SalinityConfig::RelaxToConstantValue(_) => 2,
            SalinityConfig::RelaxToInitialConditions => 3,
            SalinityConfig::RelaxToSpaceVaryingTimeSeries(_) => 4,
        }
    }
    fn get_boundary_string(
        &self,
        coords: &Array2<f64>,
        afc: &LinkedHashSet<String>,
    ) -> Option<String> {
        match self {
            SalinityConfig::RelaxToUniformTimeSeries(_) => None,
            SalinityConfig::RelaxToConstantValue(value) => Some(format!("{}", value)),
            SalinityConfig::RelaxToInitialConditions => {
                unimplemented!("SalinityConfig.get_boundary_string()->RelaxToInitialConditions");
            }
            SalinityConfig::RelaxToSpaceVaryingTimeSeries(_) => None,
        }
    }
}
