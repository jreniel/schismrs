// schismrs-bctides/src/types.rs

use crate::config::boundaries::{OpenBoundaryForcingConfig, OpenBoundaryForcings};
use crate::config::{
    ElevationBoundaryForcingConfig, SalinityBoundaryForcingConfig,
    TemperatureBoundaryForcingConfig, VelocityBoundaryForcingConfig,
};
use crate::tidefac;
use crate::traits::Bctype;
use chrono::{DateTime, Duration, Utc};

use linked_hash_set::LinkedHashSet;
use schismrs_hgrid::Hgrid;
use std::collections::BTreeMap;
use std::fmt;
use thiserror::Error;

// =============================================================================
// INTERNAL BOUNDARY FORCING CONFIGURATION
// =============================================================================

/// Type alias for boundary IDs with special sentinel value
pub type BoundaryId = u32;

// /// Special boundary ID that indicates configuration applies to all boundaries
// pub const ALL_BOUNDARIES: BoundaryId = u32::MAX;

/// Internal boundary forcing configuration used by bctides engine
///
/// This struct contains the *BoundaryForcingConfig types that implement Bctype
/// and are used for the actual boundary forcing computations. These are converted
/// from the input configuration types during the build process.
#[derive(Debug, Clone, Default)]
pub struct InternalOpenBoundaryForcingConfig {
    pub elevation: Option<BTreeMap<BoundaryId, ElevationBoundaryForcingConfig>>,
    pub velocity: Option<BTreeMap<BoundaryId, VelocityBoundaryForcingConfig>>,
    pub temperature: Option<BTreeMap<BoundaryId, TemperatureBoundaryForcingConfig>>,
    pub salinity: Option<BTreeMap<BoundaryId, SalinityBoundaryForcingConfig>>,
}

impl InternalOpenBoundaryForcingConfig {
    /// Convert from input configuration to internal configuration
    ///
    /// This is where we handle the conversion from the deserialized input types
    /// (*ForcingConfigInput) to the internal types (*BoundaryForcingConfig).
    /// We also handle the global vs per-boundary logic here.
    pub fn from_input_config(
        input: &OpenBoundaryForcings,
        num_boundaries: usize,
    ) -> Result<Self, crate::config::boundaries::OpenBoundaryForcingError> {
        let mut elevation_map = BTreeMap::new();
        let mut velocity_map = BTreeMap::new();
        let mut temperature_map = BTreeMap::new();
        let mut salinity_map = BTreeMap::new();

        match input {
            OpenBoundaryForcings::Global(config) => {
                // Apply same config to all boundaries
                for boundary_id in 0..num_boundaries {
                    let boundary_id = boundary_id as u32;
                    Self::add_forcing_configs(
                        &mut elevation_map,
                        &mut velocity_map,
                        &mut temperature_map,
                        &mut salinity_map,
                        boundary_id,
                        config,
                    )?;
                }
            }
            OpenBoundaryForcings::PerBoundary(boundary_configs) => {
                // Apply specific config to each boundary
                for (&boundary_id, config) in boundary_configs.iter() {
                    if (boundary_id as usize) >= num_boundaries {
                        return Err(crate::config::boundaries::OpenBoundaryForcingError::InvalidParameterValue(
                            format!("Boundary ID {} does not exist (only {} open boundaries)",
                                   boundary_id, num_boundaries)
                        ));
                    }

                    Self::add_forcing_configs(
                        &mut elevation_map,
                        &mut velocity_map,
                        &mut temperature_map,
                        &mut salinity_map,
                        boundary_id,
                        config,
                    )?;
                }
            }
        }

        Ok(InternalOpenBoundaryForcingConfig {
            elevation: if elevation_map.is_empty() {
                None
            } else {
                Some(elevation_map)
            },
            velocity: if velocity_map.is_empty() {
                None
            } else {
                Some(velocity_map)
            },
            temperature: if temperature_map.is_empty() {
                None
            } else {
                Some(temperature_map)
            },
            salinity: if salinity_map.is_empty() {
                None
            } else {
                Some(salinity_map)
            },
        })
    }

    /// Helper to add forcing configurations to the respective maps
    fn add_forcing_configs(
        elevation_map: &mut BTreeMap<u32, ElevationBoundaryForcingConfig>,
        velocity_map: &mut BTreeMap<u32, VelocityBoundaryForcingConfig>,
        temperature_map: &mut BTreeMap<u32, TemperatureBoundaryForcingConfig>,
        salinity_map: &mut BTreeMap<u32, SalinityBoundaryForcingConfig>,
        boundary_id: u32,
        config: &OpenBoundaryForcingConfig,
    ) -> Result<(), crate::config::boundaries::OpenBoundaryForcingError> {
        if let Some(elev_config) = &config.elevation {
            let elev = ElevationBoundaryForcingConfig::try_from(elev_config)?;
            elevation_map.insert(boundary_id, elev);
        }

        if let Some(vel_config) = &config.velocity {
            let vel = VelocityBoundaryForcingConfig::try_from(vel_config)?;
            velocity_map.insert(boundary_id, vel);
        }

        if let Some(temp_config) = &config.temperature {
            let temp = TemperatureBoundaryForcingConfig::try_from(temp_config)?;
            temperature_map.insert(boundary_id, temp);
        }

        if let Some(sal_config) = &config.salinity {
            let sal = SalinityBoundaryForcingConfig::try_from(sal_config)?;
            salinity_map.insert(boundary_id, sal);
        }

        Ok(())
    }

    /// Get active potential constituents from all tidal configurations
    pub fn get_active_potential_constituents_set(&self) -> LinkedHashSet<String> {
        let mut constituents = LinkedHashSet::new();

        // Collect from elevation configs
        if let Some(elev_map) = &self.elevation {
            for config in elev_map.values() {
                if let ElevationBoundaryForcingConfig::Tides(tides_config) = config {
                    constituents.extend(tides_config.get_active_potential_constituents());
                } else if let ElevationBoundaryForcingConfig::TidesAndSpaceVaryingTimeSeries {
                    tides,
                    ..
                } = config
                {
                    constituents.extend(tides.get_active_potential_constituents());
                }
            }
        }

        // Collect from velocity configs
        if let Some(vel_map) = &self.velocity {
            for config in vel_map.values() {
                if let VelocityBoundaryForcingConfig::Tides(tides_config) = config {
                    constituents.extend(tides_config.get_active_potential_constituents());
                } else if let VelocityBoundaryForcingConfig::TidesAndSpaceVaryingTimeSeries {
                    tides,
                    ..
                } = config
                {
                    constituents.extend(tides.get_active_potential_constituents());
                }
            }
        }

        constituents
    }

    /// Get active forcing constituents from all tidal configurations
    pub fn get_active_forcing_constituents_set(&self) -> LinkedHashSet<String> {
        let mut constituents = LinkedHashSet::new();

        // Collect from elevation configs
        if let Some(elev_map) = &self.elevation {
            for config in elev_map.values() {
                if let ElevationBoundaryForcingConfig::Tides(tides_config) = config {
                    constituents.extend(tides_config.get_active_forcing_constituents());
                } else if let ElevationBoundaryForcingConfig::TidesAndSpaceVaryingTimeSeries {
                    tides,
                    ..
                } = config
                {
                    constituents.extend(tides.get_active_forcing_constituents());
                }
            }
        }

        // Collect from velocity configs
        if let Some(vel_map) = &self.velocity {
            for config in vel_map.values() {
                if let VelocityBoundaryForcingConfig::Tides(tides_config) = config {
                    constituents.extend(tides_config.get_active_forcing_constituents());
                } else if let VelocityBoundaryForcingConfig::TidesAndSpaceVaryingTimeSeries {
                    tides,
                    ..
                } = config
                {
                    constituents.extend(tides.get_active_forcing_constituents());
                }
            }
        }

        constituents
    }
}

// =============================================================================
// BCTIDES TYPES
// =============================================================================

#[derive(Debug)]
pub struct Bctides<'a> {
    hgrid: &'a Hgrid,
    start_date: DateTime<Utc>,
    run_duration: Duration,
    tidal_potential_cutoff_depth: f64,
    /// This is the internal boundary forcing config that contains the converted types
    open_boundary_forcing_config: InternalOpenBoundaryForcingConfig,
}

impl<'a> Bctides<'a> {
    fn tip_dp(&self) -> f64 {
        self.tidal_potential_cutoff_depth
    }

    fn get_bctypes_vec(&self, this_bnd_key: &BoundaryId, _this_nodes: &Vec<u32>) -> Vec<i8> {
        let mut bctypes = Vec::new();

        // Elevation boundary type
        match &self.open_boundary_forcing_config.elevation {
            Some(conf) => {
                if let Some(this_bnd_config) = conf.get(this_bnd_key) {
                    bctypes.push(this_bnd_config.ibtype());
                } else {
                    bctypes.push(0 as i8);
                }
            }
            None => bctypes.push(0 as i8),
        };

        // Velocity boundary type
        match &self.open_boundary_forcing_config.velocity {
            Some(conf) => {
                if let Some(this_bnd_config) = conf.get(this_bnd_key) {
                    bctypes.push(this_bnd_config.ibtype());
                } else {
                    bctypes.push(0 as i8);
                }
            }
            None => bctypes.push(0 as i8),
        };

        // Temperature boundary type
        match &self.open_boundary_forcing_config.temperature {
            Some(conf) => {
                if let Some(this_bnd_config) = conf.get(this_bnd_key) {
                    bctypes.push(this_bnd_config.ibtype());
                } else {
                    bctypes.push(0 as i8);
                }
            }
            None => bctypes.push(0 as i8),
        };

        // Salinity boundary type
        match &self.open_boundary_forcing_config.salinity {
            Some(conf) => {
                if let Some(this_bnd_config) = conf.get(this_bnd_key) {
                    bctypes.push(this_bnd_config.ibtype());
                } else {
                    bctypes.push(0 as i8);
                }
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
            let r = tidefac(&self.start_date, &self.run_duration, constituent);
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
            let r = tidefac(&self.start_date, &self.run_duration, constituent);
            write!(
                f,
                "{}\n {} {} {}\n",
                constituent.strip_prefix('_').unwrap_or(constituent),
                r.orbital_frequency(),
                r.nodal_factor(),
                r.greenwich_factor(),
            )?;
        }
        let nodes_ids = self.hgrid.boundaries().unwrap().open().unwrap().nodes_ids();
        write!(f, "{} !# number of open bnd segs\n", nodes_ids.len(),)?;
        for (this_bnd_key, this_nodes) in nodes_ids.iter().enumerate() {
            let this_bnd_key = this_bnd_key as BoundaryId;
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
        self.open_boundary_forcing_config
            .get_active_potential_constituents_set()
    }

    pub fn get_active_forcing_constituents_set(&self) -> LinkedHashSet<String> {
        self.open_boundary_forcing_config
            .get_active_forcing_constituents_set()
    }

    fn get_boundary_string(&self) -> String {
        unimplemented!("Bctides.get_boundary_string() is not implemented.")
    }
}

#[derive(Default)]
pub struct BctidesBuilder<'a> {
    hgrid: Option<&'a Hgrid>,
    start_date: Option<&'a DateTime<Utc>>,
    run_duration: Option<&'a Duration>,
    tidal_potential_cutoff_depth: Option<f64>,
    /// Builder accepts the input configuration type and converts during build()
    open_boundary_forcing_config: Option<&'a OpenBoundaryForcings>,
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
        let input_config = self.open_boundary_forcing_config.ok_or_else(|| {
            BctidesBuilderError::UninitializedFieldError("open_boundary_forcing_config".to_string())
        })?;
        let hgrid = self
            .hgrid
            .ok_or_else(|| BctidesBuilderError::UninitializedFieldError("hgrid".to_string()))?;

        Self::validate(tidal_potential_cutoff_depth)?;

        // Get number of open boundaries from hgrid
        let num_open_boundaries = hgrid
            .boundaries()
            .and_then(|b| b.open())
            .map(|ob| ob.nodes_ids().len())
            .unwrap_or(0);

        // Convert input config to internal config
        let internal_config =
            InternalOpenBoundaryForcingConfig::from_input_config(input_config, num_open_boundaries)
                .map_err(|e| BctidesBuilderError::ConfigurationError(e.to_string()))?;

        Ok(Bctides {
            hgrid: hgrid,
            start_date: start_date.clone(),
            run_duration: run_duration.clone(),
            tidal_potential_cutoff_depth,
            open_boundary_forcing_config: internal_config,
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

    pub fn tidal_potential_cutoff_depth(&mut self, tidal_potential_cutoff_depth: f64) -> &mut Self {
        self.tidal_potential_cutoff_depth = Some(tidal_potential_cutoff_depth);
        self
    }

    pub fn hgrid(&mut self, hgrid: &'a Hgrid) -> &mut Self {
        self.hgrid = Some(hgrid);
        self
    }

    /// Accept the input configuration type (OpenBoundaryForcings)
    /// The conversion to internal types happens during build()
    pub fn open_boundary_forcing_config(
        &mut self,
        open_boundary_forcing_config: &'a OpenBoundaryForcings,
    ) -> &mut Self {
        self.open_boundary_forcing_config = Some(open_boundary_forcing_config);
        self
    }

    fn validate(tidal_potential_cutoff_depth: f64) -> Result<(), BctidesBuilderError> {
        Self::validate_tidal_potential_cutoff_depth(tidal_potential_cutoff_depth)?;
        Ok(())
    }

    fn validate_tidal_potential_cutoff_depth(
        tidal_potential_cutoff_depth: f64,
    ) -> Result<(), BctidesBuilderError> {
        if tidal_potential_cutoff_depth < 0. {
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
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
}

