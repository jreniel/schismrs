use crate::bctypes::Bctype;
use crate::tidefac;
use crate::ElevationConfig;
use crate::SalinityConfig;
use crate::TemperatureConfig;
use crate::VelocityConfig;
use chrono::{DateTime, Duration, Utc};
use linked_hash_set::LinkedHashSet;
use schismrs_hgrid::Hgrid;
use std::collections::BTreeMap;
use std::fmt;
use thiserror::Error;

#[derive(Debug)]
pub struct Bctides<'a> {
    start_date: &'a DateTime<Utc>,
    run_duration: &'a Duration,
    tidal_potential_cutoff_depth: &'a f64,
    boundary_forcing_config: &'a BoundaryForcingConfig<'a>,
}

impl<'a> Bctides<'a> {
    fn tip_dp(&self) -> &f64 {
        self.tidal_potential_cutoff_depth
    }
    // fn get_bctypes_vec(&self) -> Vec<u8> {
    fn get_bctypes_vec(&self, this_bnd_key: &u32, this_nodes: &Vec<u32>) -> Vec<i8> {
        let mut bctypes = Vec::new();
        match self.boundary_forcing_config.elevation {
            Some(conf) => {
                if conf.contains_key(&this_bnd_key) {
                    let this_bnd_config = conf.get(&this_bnd_key).unwrap();
                    bctypes.push(this_bnd_config.ibtype());
                };
            }
            None => bctypes.push(0 as i8),
        };
        match self.boundary_forcing_config.velocity {
            Some(conf) => {
                if conf.contains_key(&this_bnd_key) {
                    let this_bnd_config = conf.get(&this_bnd_key).unwrap();
                    bctypes.push(this_bnd_config.ibtype());
                };
            }
            None => bctypes.push(0 as i8),
        };
        match self.boundary_forcing_config.temperature {
            Some(conf) => {
                if conf.contains_key(&this_bnd_key) {
                    let this_bnd_config = conf.get(&this_bnd_key).unwrap();
                    bctypes.push(this_bnd_config.ibtype());
                };
            }
            None => bctypes.push(0 as i8),
        };
        match self.boundary_forcing_config.salinity {
            Some(conf) => {
                if conf.contains_key(&this_bnd_key) {
                    let this_bnd_config = conf.get(&this_bnd_key).unwrap();
                    bctypes.push(this_bnd_config.ibtype());
                };
            }
            None => bctypes.push(0 as i8),
        };
        bctypes
    }
    fn get_bctypes_line(this_nodes: &Vec<u32>, bctypes_vec: Vec<i8>) -> String {
        let mut this_line = Vec::new();
        this_line.push(format!("{}", this_nodes.len()));
        for item in bctypes_vec.iter() {
            this_line.push(format!("{}", item));
        }
        this_line.join(" ")
    }
}

impl fmt::Display for Bctides<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", self.start_date)?;
        let apc_set = self.get_active_potential_constituents_set();
        write!(
            f,
            "{} {} !# number of tidal potential and cut-off depth\n",
            apc_set.len(),
            self.tip_dp()
        )?;
        for constituent in apc_set.iter() {
            let r = tidefac(self.start_date, self.run_duration, constituent);
            write!(
                f,
                "{}\n{} {} {} {} {}\n",
                constituent.strip_prefix('_').unwrap_or(constituent),
                r.tidal_species_type(),
                r.tidal_potential_amplitude(),
                r.orbital_frequency(),
                r.nodal_factor(),
                r.greenwich_factor(),
            )?;
        }
        let afc_set = self.get_active_forcing_constituents_set();
        write!(f, "{} !# of boundary tidal frequencies\n", afc_set.len())?;
        for constituent in afc_set.iter() {
            let r = tidefac(self.start_date, self.run_duration, constituent);
            write!(
                f,
                "{}\n {} {} {}\n",
                constituent.strip_prefix('_').unwrap_or(constituent),
                r.orbital_frequency(),
                r.nodal_factor(),
                r.greenwich_factor(),
            )?;
        }
        let nodes_ids = self
            .boundary_forcing_config
            .hgrid
            .boundaries()
            .unwrap()
            .open()
            .unwrap()
            .nodes_ids();
        write!(f, "{} !# number of open bnd segs\n", nodes_ids.len(),)?;
        for (this_bnd_key, this_nodes) in nodes_ids.iter().enumerate() {
            let this_bnd_key = this_bnd_key as u32;
            let bctypes_vec = self.get_bctypes_vec(&this_bnd_key, this_nodes);
            let bctypes_line = Self::get_bctypes_line(this_nodes, bctypes_vec);
            write!(f, "{}", bctypes_line)?;
            let boundary_lines = self.get_boundary_string();
            write!(f, "{}", boundary_lines)?;
        }
        Ok(())
    }
}

impl<'a> Bctides<'a> {
    pub fn get_active_potential_constituents_set(&self) -> LinkedHashSet<String> {
        self.boundary_forcing_config
            .get_active_potential_constituents_set()
    }
    pub fn get_active_forcing_constituents_set(&self) -> LinkedHashSet<String> {
        self.boundary_forcing_config
            .get_active_forcing_constituents_set()
    }
}

#[derive(Default)]
pub struct BctidesBuilder<'a> {
    start_date: Option<&'a DateTime<Utc>>,
    run_duration: Option<&'a Duration>,
    tidal_potential_cutoff_depth: Option<&'a f64>,
    boundary_forcing_config: Option<&'a BoundaryForcingConfig<'a>>,
}

impl<'a> BctidesBuilder<'a> {
    pub fn build(&self) -> Result<Bctides, BctidesBuilderError> {
        let start_date = self.start_date.ok_or_else(|| {
            BctidesBuilderError::UninitializedFieldError("start_date".to_string())
        })?;
        let run_duration = self.run_duration.ok_or_else(|| {
            BctidesBuilderError::UninitializedFieldError("run_duration".to_string())
        })?;
        let tidal_potential_cutoff_depth = self.tidal_potential_cutoff_depth.ok_or_else(|| {
            BctidesBuilderError::UninitializedFieldError("tidal_potential_cutoff_depth".to_string())
        })?;
        let boundary_forcing_config = self.boundary_forcing_config.ok_or_else(|| {
            BctidesBuilderError::UninitializedFieldError("boundary_forcing_config".to_string())
        })?;
        Self::validate(tidal_potential_cutoff_depth)?;
        Ok(Bctides {
            // hgrid,
            start_date,
            run_duration,
            tidal_potential_cutoff_depth,
            boundary_forcing_config,
        })
    }
    pub fn start_date(&mut self, start_date: &'a DateTime<Utc>) -> &mut Self {
        self.start_date = Some(start_date);
        self
    }
    pub fn run_duration(&mut self, run_duration: &'a Duration) -> &mut Self {
        self.run_duration = Some(run_duration);
        self
    }
    pub fn tidal_potential_cutoff_depth(
        &mut self,
        tidal_potential_cutoff_depth: &'a f64,
    ) -> &mut Self {
        self.tidal_potential_cutoff_depth = Some(tidal_potential_cutoff_depth);
        self
    }
    pub fn boundary_forcing_config(
        &mut self,
        boundary_forcing_config: &'a BoundaryForcingConfig,
    ) -> &mut Self {
        self.boundary_forcing_config = Some(boundary_forcing_config);
        self
    }
    fn validate(tidal_potential_cutoff_depth: &'a f64) -> Result<(), BctidesBuilderError> {
        Self::validate_tidal_potential_cutoff_depth(tidal_potential_cutoff_depth)?;
        Ok(())
    }
    fn validate_tidal_potential_cutoff_depth(
        tidal_potential_cutoff_depth: &f64,
    ) -> Result<(), BctidesBuilderError> {
        if tidal_potential_cutoff_depth < &0. {
            return Err(BctidesBuilderError::InvalidTidalPotentialCutoffDepth);
        }
        Ok(())
    }
}
#[derive(Error, Debug)]
pub enum BctidesBuilderError {
    #[error("Unitialized field on BctidesBuilder: {0}")]
    UninitializedFieldError(String),
    #[error("tidal_potential_cutoff_depth must be >= 0.")]
    InvalidTidalPotentialCutoffDepth,
}

#[derive(Debug)]
pub struct BoundaryForcingConfig<'a> {
    hgrid: &'a Hgrid,
    elevation: Option<&'a BTreeMap<u32, ElevationConfig>>,
    velocity: Option<&'a BTreeMap<u32, VelocityConfig>>,
    temperature: Option<&'a BTreeMap<u32, TemperatureConfig>>,
    salinity: Option<&'a BTreeMap<u32, SalinityConfig>>,
}

impl<'a> BoundaryForcingConfig<'a> {
    fn get_active_potential_constituents_set(&self) -> LinkedHashSet<String> {
        let mut hash_set = LinkedHashSet::<String>::new();
        match self.elevation {
            Some(config) => {
                for cfg_value in config.values() {
                    match cfg_value {
                        ElevationConfig::TidesAndSpaceVaryingTimeSeries {
                            tides,
                            time_series: _,
                        } => {
                            for apc in tides.constituents.get_active_potential_constituents() {
                                hash_set.insert_if_absent(apc);
                            }
                        }
                        ElevationConfig::Tides(config) => {
                            for apc in config.constituents.get_active_potential_constituents() {
                                hash_set.insert_if_absent(apc);
                            }
                        }
                        _ => {}
                    }
                }
            }
            None => {}
        };
        match self.velocity {
            Some(config) => {
                for (_bnd_key, cfg_value) in config.iter() {
                    match cfg_value {
                        VelocityConfig::TidesAndSpaceVaryingTimeSeries {
                            tides,
                            time_series: _,
                        } => {
                            for apc in tides.constituents.get_active_potential_constituents() {
                                hash_set.insert_if_absent(apc);
                            }
                        }
                        VelocityConfig::Tides(config) => {
                            for apc in config.constituents.get_active_potential_constituents() {
                                hash_set.insert_if_absent(apc);
                            }
                        }
                        _ => {}
                    }
                }
            }
            None => {}
        };
        hash_set
    }
    fn get_active_forcing_constituents_set(&self) -> LinkedHashSet<String> {
        let mut hash_set = LinkedHashSet::<String>::new();
        match self.elevation {
            Some(config) => {
                for cfg_value in config.values() {
                    match cfg_value {
                        ElevationConfig::TidesAndSpaceVaryingTimeSeries {
                            tides,
                            time_series: _,
                        } => {
                            for apc in tides.constituents.get_active_forcing_constituents() {
                                hash_set.insert_if_absent(apc);
                            }
                        }
                        ElevationConfig::Tides(config) => {
                            for apc in config.constituents.get_active_forcing_constituents() {
                                hash_set.insert_if_absent(apc);
                            }
                        }
                        _ => {}
                    }
                }
            }
            None => {}
        };
        match self.velocity {
            Some(config) => {
                for (_bnd_key, cfg_value) in config.iter() {
                    match cfg_value {
                        VelocityConfig::TidesAndSpaceVaryingTimeSeries {
                            tides,
                            time_series: _,
                        } => {
                            for apc in tides.constituents.get_active_forcing_constituents() {
                                hash_set.insert_if_absent(apc);
                            }
                        }
                        VelocityConfig::Tides(config) => {
                            for apc in config.constituents.get_active_forcing_constituents() {
                                hash_set.insert_if_absent(apc);
                            }
                        }
                        _ => {}
                    }
                }
            }
            None => {}
        };
        hash_set
    }
}

#[derive(Default)]
pub struct BoundaryForcingConfigBuilder<'a> {
    hgrid: Option<&'a Hgrid>,
    elevation: Option<&'a BTreeMap<u32, ElevationConfig>>,
    velocity: Option<&'a BTreeMap<u32, VelocityConfig>>,
    temperature: Option<&'a BTreeMap<u32, TemperatureConfig>>,
    salinity: Option<&'a BTreeMap<u32, SalinityConfig>>,
}

impl<'a> BoundaryForcingConfigBuilder<'a> {
    pub fn build(&self) -> Result<BoundaryForcingConfig, BoundaryForcingConfigBuilderError> {
        let hgrid = self.hgrid.ok_or_else(|| {
            BoundaryForcingConfigBuilderError::UninitializedFieldError("hgrid".to_string())
        })?;
        // Self::validate(hgrid, self.elevation)?;
        Ok(BoundaryForcingConfig {
            hgrid,
            elevation: self.elevation,
            velocity: self.velocity,
            temperature: self.temperature,
            salinity: self.salinity,
        })
    }
    pub fn hgrid(&mut self, hgrid: &'a Hgrid) -> &mut Self {
        self.hgrid = Some(hgrid);
        self
    }
    pub fn elevation(&mut self, elevation: &'a BTreeMap<u32, ElevationConfig>) -> &mut Self {
        self.elevation = Some(elevation);
        self
    }
    pub fn velocity(&mut self, velocity: &'a BTreeMap<u32, VelocityConfig>) -> &mut Self {
        self.velocity = Some(velocity);
        self
    }
    pub fn temperature(&mut self, temperature: &'a BTreeMap<u32, TemperatureConfig>) -> &mut Self {
        self.temperature = Some(temperature);
        self
    }
    pub fn salinity(&mut self, salinity: &'a BTreeMap<u32, SalinityConfig>) -> &mut Self {
        self.salinity = Some(salinity);
        self
    }
}
#[derive(Error, Debug)]
pub enum BoundaryForcingConfigBuilderError {
    #[error("Unitialized field on BoundaryForcingConfigBuilder: {0}")]
    UninitializedFieldError(String),
    // #[error("tidal_potential_cutoff_depth must be >= 0.")]
    // InvalidTidalPotentialCutoffDepth,
}
