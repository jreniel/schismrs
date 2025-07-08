use chrono::{DateTime, Duration, Utc};
use clap::builder::PossibleValuesParser;
use clap::command;
use clap::Arg;
use clap::ArgMatches;
use clap::Args;
use clap::Command;
use clap::FromArgMatches;
use clap::Parser;
use humantime;
use linked_hash_map::LinkedHashMap;
use regex::Regex;
use schismrs_bctides::tides;
use schismrs_bctides::BctidesBuilder;
// Updated imports to match the new API
use schismrs_bctides::config::{
    ConstituentPreset, ConstituentSelection, ConstituentsConfigInput, ElevationForcingConfigInput,
    OpenBoundaryForcingConfig, OpenBoundaryForcings, SalinityForcingConfigInput,
    TemperatureForcingConfigInput, TidesConfigInput, VelocityForcingConfigInput,
};
use schismrs_hgrid::Hgrid;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;
use strum::IntoEnumIterator;
use strum_macros::{AsRefStr, EnumIter};

// static HGRID: Lazy<Mutex<Option<Hgrid>>> = Lazy::new(|| Mutex::new(None));
thread_local! {
    static HGRID: RefCell<Option<Hgrid>> = RefCell::new(None);
}

// | Variable     | Type -4, -5 (`uv3D.th`); Nudging                                  | Nudging/Sponge layer near bnd |
// |--------------|-------------------------------------------------------------------|--------------------------------------------|--------------------------------|
// | η            | N/A                                                               | `inu_elev=1`                     |
// | S&T, Tracers | N/A                                                               | `inu_[MOD]=1 or 2`               |
// | u,v          | Relax to `uv3D.th.nc` (2 separate relaxations for in and outflow) | `inu_uv=1`                        |

#[derive(Parser, Debug)]
#[command(
    about,
    version,
    // ignore_errors = true,
    after_help = "\x1b[1m\x1b[4mDynamic Arguments:\x1b[0m \
    This program uses the input hgrid to generate additional dynamic arguments which depend on the input hgrid \
    and can be used to assign different forcing types to each boundary.\n  \
    \x1b[1m--<VARIABLE_NAME>-<BOUNDARY_ID>=<IBTYPE>\x1b[0m\n\
    \tFor example: --elevation-1=4 applies elevation type 4 to boundary id 0.\n  \
    \x1b[1m--<VARIABLE_NAME>-tidal-db-<BOUNDARY_ID>=<TIDAL_DATABASE_NAME>\x1b[0m\n\
    \tRequired if using elevation or velocity of types 3 or 5\n  \
    \x1b[1m--<VARIABLE_NAME>-baroclinic-db-<BOUNDARY_ID>=<BAROCLINIC_DATABASE_NAME>\x1b[0m\n  \
    \tRequired if using elevation, velocity of types 4 or 5, if using velocity type -4 or -5, or if using salinity or temperature of type 5.\n  \
    \nFor this emtrypoint, boundary ID's begin at 1 as per the convention of hgrid files (although internally they are represented as 0-indexed vectors). \n\n\
    The following tables describe each variable and associated types:\n\n\
| Variable     | Type 1 (`*.th`)                                                       |
|--------------|-----------------------------------------------------------------------|
| η            | `elev.th`; Time history; uniform along bnd                            |
| S&T, Tracers | `[MOD]_[ID].th`: relax to time history (uniform along bnd for inflow) |
| u,v          | `flux.th`: via discharge ( <0 for inflow!)                            |

| Variable     | Type 2                              |
|--------------|-------------------------------------|
| η            | constant                            |
| S&T, Tracers | Relax to specified value for inflow |
| u,v          | Via discharge (<0 for inflow)       |

| Variable     | Type 3                                  |
|--------------|-----------------------------------------|
| η            | Tidal amp/phases                        |
| S&T, Tracers | Relax to i.c. for inflow                |
| u,v          | Tidal amp/phases for u and v components |


| Variable     | Type 4 (`*[23]D.th`)                                                               |
|--------------|------------------------------------------------------------------------------------|
| η            | `elev2D.th.nc`: time- and space- varying along bnd                                 |
| S&T, Tracers | `[MOD]_3D.th.nc`: relax to time- and space- varying values along bnd during inflow |
| u,v          | `uv3D.th.nc`: time- and space- varying along bnd (in lon/lat for `ics=2`)          |

| Variable     | Type 5                         |
|--------------|--------------------------------|
| η            | `elev2D.th.nc`: sum of 3 and 4 |
| S&T, Tracers | N/A                            |
| u,v          | `uv3D.th.nc`: sum of 3 and 4   |

| Variable     | Type -1           |
|--------------|-------------------|
| η            | Must = 0          |
| S&T, Tracers | N/A               |
| u,v          | Flather (0 for η) |

| Variable     | Type -4, -5 (`uv3D.th`); Nudging                                  |
|--------------|-------------------------------------------------------------------|
| η            | N/A                                                               |
| S&T, Tracers | N/A                                                               |
| u,v          | Relax to `uv3D.th.nc` (2 separate relaxations for in and outflow) |

Variable legend:
 η: elevation
 S: salinity
 T: temperature
 u,v: velocity
"
)]
struct Cli {
    hgrid_path: PathBuf,
    #[clap(value_parser = dateparser::parse, help="See https://docs.rs/dateparser/latest/dateparser/#accepted-date-formats for a list of accepted input formats.")]
    start_date: DateTime<Utc>,
    #[clap(long, required = true)]
    run_duration: humantime::Duration,
    /// aliases: tip-dp, tip_dp, cutoff_depth, cutoff-depth, tpcd
    #[clap(short, long, aliases = &["tip-dp", "tip_dp", "cutoff_depth", "cutoff-depth", "tpcd"])]
    tidal_potential_cutoff_depth: f64,
    #[command(flatten)]
    boundary_config: BoundaryConfigArgs,
}

#[derive(Debug)]
struct BoundaryConfigArgs {
    elevation: Option<BTreeMap<u32, ElevationForcingConfigInput>>,
    velocity: Option<BTreeMap<u32, VelocityForcingConfigInput>>,
    temperature: Option<BTreeMap<u32, TemperatureForcingConfigInput>>,
    salinity: Option<BTreeMap<u32, SalinityForcingConfigInput>>,
}

impl BoundaryConfigArgs {
    fn get_elevation_config(
        matches: &ArgMatches,
        key_str: &str,
        fort_id: &str,
        bnd_key: &u32,
    ) -> ElevationForcingConfigInput {
        let user_value = matches.get_one::<String>(&key_str).unwrap();
        let elev_map = get_elevation_bctypes_map();
        let the_requested_type = elev_map.get(user_value).unwrap();
        match the_requested_type {
            ElevationConfigType::TidesAndSpaceVaryingTimeSeries => {
                let constituents = match (
                    matches
                        .get_one::<bool>(&format!("elevation-{}-all", fort_id))
                        .unwrap(),
                    matches
                        .get_one::<bool>(&format!("elevation-{}-major", fort_id))
                        .unwrap(),
                    matches
                        .get_one::<bool>(&format!("elevation-{}-minor", fort_id))
                        .unwrap(),
                ) {
                    (true, false, false) => ConstituentSelection::Preset(ConstituentPreset::All),
                    (false, true, false) => ConstituentSelection::Preset(ConstituentPreset::Major),
                    (false, false, true) => ConstituentSelection::Preset(ConstituentPreset::Minor),
                    (false, false, false) => {
                        let mut ec = ConstituentsConfigInput::default();
                        for constituent_name in tides::ConstituentsConfig::field_names().iter() {
                            let constituent_flag_base_name = get_constituent_flag_base_name(
                                bnd_key,
                                "elevation",
                                &constituent_name,
                            );
                            match matches.get_one::<bool>(constituent_flag_base_name).unwrap() {
                                true => {
                                    // Set the constituent field to true using reflection or match
                                    match constituent_name.as_ref() {
                                        "Q1" => ec.Q1 = true,
                                        "O1" => ec.O1 = true,
                                        "P1" => ec.P1 = true,
                                        "K1" => ec.K1 = true,
                                        "N2" => ec.N2 = true,
                                        "M2" => ec.M2 = true,
                                        "S2" => ec.S2 = true,
                                        "K2" => ec.K2 = true,
                                        "Mm" => ec.Mm = true,
                                        "Mf" => ec.Mf = true,
                                        "M4" => ec.M4 = true,
                                        "MN4" => ec.MN4 = true,
                                        "MS4" => ec.MS4 = true,
                                        "_2N2" => ec._2N2 = true,
                                        "S1" => ec.S1 = true,
                                        _ => {}
                                    }
                                }
                                false => {}
                            }
                        }
                        ConstituentSelection::Custom { constituents: ec }
                    }
                    (_, _, _) => panic!("Unreachable!"),
                };
                let database =
                    match matches.get_one::<String>(&format!("elevation-{}-tidal-db", fort_id)) {
                        Some(tidal_db) => match get_tidal_db_possible_values_map().get(tidal_db) {
                            Some(TidalDbConfigType::TPXO) => tides::TidalDatabase::TPXO,
                            Some(TidalDbConfigType::FES) => tides::TidalDatabase::FES,
                            Some(TidalDbConfigType::HAMTIDE) => tides::TidalDatabase::HAMTIDE,
                            None => panic!("Unreachable!"),
                        },
                        None => panic!("Unreachable"),
                    };
                let tides_config = TidesConfigInput {
                    database,
                    constituents,
                };
                let time_series_database = match matches
                    .get_one::<String>(&format!("elevation-{}-baroclinic-db", fort_id))
                {
                    Some(tidal_db) => match get_baroclinic_db_possible_values_map().get(tidal_db) {
                        Some(BaroclinicDbConfigType::HYCOM) => tides::TimeSeriesDatabase::HYCOM,
                        None => panic!("Unreachable!"),
                    },
                    None => panic!("Unreachable"),
                };
                ElevationForcingConfigInput::TidesAndSpaceVaryingTimeSeries {
                    tides: tides_config,
                    time_series: time_series_database,
                }
            }
            _ => panic!("Unhandled type: {:?}", the_requested_type),
        }
    }

    fn get_velocity_config(
        matches: &ArgMatches,
        key_str: &str,
        fort_id: &str,
        bnd_key: &u32,
    ) -> VelocityForcingConfigInput {
        let user_value = matches.get_one::<String>(&key_str).unwrap();
        let velo_map = get_velocity_bctypes_map();
        let the_requested_type = velo_map.get(user_value).unwrap();
        match the_requested_type {
            VelocityConfigType::TidesAndSpaceVaryingTimeSeries => {
                let constituents = match (
                    matches
                        .get_one::<bool>(&format!("velocity-{}-all", fort_id))
                        .unwrap(),
                    matches
                        .get_one::<bool>(&format!("velocity-{}-major", fort_id))
                        .unwrap(),
                    matches
                        .get_one::<bool>(&format!("velocity-{}-minor", fort_id))
                        .unwrap(),
                ) {
                    (true, false, false) => ConstituentSelection::Preset(ConstituentPreset::All),
                    (false, true, false) => ConstituentSelection::Preset(ConstituentPreset::Major),
                    (false, false, true) => ConstituentSelection::Preset(ConstituentPreset::Minor),
                    (false, false, false) => {
                        let mut ec = ConstituentsConfigInput::default();
                        for constituent_name in tides::ConstituentsConfig::field_names().iter() {
                            let constituent_flag_base_name = get_constituent_flag_base_name(
                                bnd_key,
                                "velocity",
                                &constituent_name,
                            );
                            match matches.get_one::<bool>(constituent_flag_base_name).unwrap() {
                                true => {
                                    // Set the constituent field to true using match
                                    match constituent_name.as_ref() {
                                        "Q1" => ec.Q1 = true,
                                        "O1" => ec.O1 = true,
                                        "P1" => ec.P1 = true,
                                        "K1" => ec.K1 = true,
                                        "N2" => ec.N2 = true,
                                        "M2" => ec.M2 = true,
                                        "S2" => ec.S2 = true,
                                        "K2" => ec.K2 = true,
                                        "Mm" => ec.Mm = true,
                                        "Mf" => ec.Mf = true,
                                        "M4" => ec.M4 = true,
                                        "MN4" => ec.MN4 = true,
                                        "MS4" => ec.MS4 = true,
                                        "_2N2" => ec._2N2 = true,
                                        "S1" => ec.S1 = true,
                                        _ => {}
                                    }
                                }
                                false => {}
                            }
                        }
                        ConstituentSelection::Custom { constituents: ec }
                    }
                    (_, _, _) => panic!("Unreachable!"),
                };
                let database =
                    match matches.get_one::<String>(&format!("velocity-{}-tidal-db", fort_id)) {
                        Some(tidal_db) => match get_tidal_db_possible_values_map().get(tidal_db) {
                            Some(TidalDbConfigType::TPXO) => tides::TidalDatabase::TPXO,
                            Some(TidalDbConfigType::FES) => tides::TidalDatabase::FES,
                            Some(TidalDbConfigType::HAMTIDE) => tides::TidalDatabase::HAMTIDE,
                            None => panic!("Unreachable!"),
                        },
                        None => panic!("Unreachable"),
                    };
                let tides_config = TidesConfigInput {
                    database,
                    constituents,
                };
                let time_series_database = match matches
                    .get_one::<String>(&format!("velocity-{}-baroclinic-db", fort_id))
                {
                    Some(tidal_db) => match get_baroclinic_db_possible_values_map().get(tidal_db) {
                        Some(BaroclinicDbConfigType::HYCOM) => tides::TimeSeriesDatabase::HYCOM,
                        None => panic!("Unreachable!"),
                    },
                    None => panic!("Unreachable"),
                };
                VelocityForcingConfigInput::TidesAndSpaceVaryingTimeSeries {
                    tides: tides_config,
                    time_series: time_series_database,
                }
            }
            _ => panic!("Unhandled type: {:?}", the_requested_type),
        }
    }

    fn get_temperature_config(
        matches: &ArgMatches,
        key_str: &str,
        _fort_id: &str,
        _bnd_key: &u32,
    ) -> TemperatureForcingConfigInput {
        let user_value = matches.get_one::<String>(&key_str).unwrap();
        let tem_map = get_temperature_bctypes_map();
        let the_requested_type = tem_map.get(user_value).unwrap();
        match the_requested_type {
            _ => panic!("Unhandled type: {:?}", the_requested_type),
        }
    }

    fn get_salinity_config(
        matches: &ArgMatches,
        key_str: &str,
        _fort_id: &str,
        _bnd_key: &u32,
    ) -> SalinityForcingConfigInput {
        let user_value = matches.get_one::<String>(&key_str).unwrap();
        let salt_map = get_salinity_bctypes_map();
        let the_requested_type = salt_map.get(user_value).unwrap();
        match the_requested_type {
            _ => panic!("Unhandled type: {:?}", the_requested_type),
        }
    }
}

impl FromArgMatches for BoundaryConfigArgs {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::error::Error> {
        let mut matches = matches.clone();
        Self::from_arg_matches_mut(&mut matches)
    }

    fn from_arg_matches_mut(matches: &mut ArgMatches) -> Result<Self, clap::error::Error> {
        let mut elevation_map = BTreeMap::<u32, ElevationForcingConfigInput>::new();
        let mut velocity_map = BTreeMap::<u32, VelocityForcingConfigInput>::new();
        let mut temperature_map = BTreeMap::<u32, TemperatureForcingConfigInput>::new();
        let mut salinity_map = BTreeMap::<u32, SalinityForcingConfigInput>::new();
        let ele_re = Regex::new(r"elevation-(\d+)$").unwrap();
        let vel_re = Regex::new(r"velocity-(\d+)$").unwrap();
        let tem_re = Regex::new(r"temperature-(\d+)$").unwrap();
        let sal_re = Regex::new(r"salinity-(\d+)$").unwrap();
        for key in matches.ids() {
            let key_str = key.to_string();
            match (
                ele_re.captures(&key_str),
                vel_re.captures(&key_str),
                tem_re.captures(&key_str),
                sal_re.captures(&key_str),
            ) {
                // Here we transform from fort_id representation to 0-index
                (Some(caps), None, None, None) => {
                    let fort_id = caps.get(1).unwrap().as_str();
                    let bnd_key = fort_id.parse::<u32>().unwrap() - 1;
                    elevation_map.insert(
                        bnd_key,
                        Self::get_elevation_config(matches, &key_str, fort_id, &bnd_key),
                    );
                }
                (None, Some(caps), None, None) => {
                    let fort_id = caps.get(1).unwrap().as_str();
                    let bnd_key = fort_id.parse::<u32>().unwrap() - 1;
                    velocity_map.insert(
                        bnd_key,
                        Self::get_velocity_config(matches, &key_str, fort_id, &bnd_key),
                    );
                }
                (None, None, Some(caps), None) => {
                    let fort_id = caps.get(1).unwrap().as_str();
                    let bnd_key = fort_id.parse::<u32>().unwrap() - 1;
                    temperature_map.insert(
                        bnd_key,
                        Self::get_temperature_config(matches, &key_str, fort_id, &bnd_key),
                    );
                }
                (None, None, None, Some(caps)) => {
                    let fort_id = caps.get(1).unwrap().as_str();
                    let bnd_key = fort_id.parse::<u32>().unwrap() - 1;
                    salinity_map.insert(
                        bnd_key,
                        Self::get_salinity_config(matches, &key_str, fort_id, &bnd_key),
                    );
                }
                (None, None, None, None) => {}
                (_, _, _, _) => {
                    panic!("Unreachable: {}!", key_str);
                }
            }
        }
        let elevation = if elevation_map.is_empty() {
            None
        } else {
            Some(elevation_map)
        };
        let velocity = if velocity_map.is_empty() {
            None
        } else {
            Some(velocity_map)
        };
        let temperature = if temperature_map.is_empty() {
            None
        } else {
            Some(temperature_map)
        };
        let salinity = if salinity_map.is_empty() {
            None
        } else {
            Some(salinity_map)
        };
        Ok(Self {
            elevation,
            velocity,
            temperature,
            salinity,
        })
    }
    fn update_from_arg_matches(&mut self, _matches: &ArgMatches) -> Result<(), clap::error::Error> {
        panic!("You should not directly call BoundaryConfigArgs::update_from_arg_matches");
    }
    fn update_from_arg_matches_mut(
        &mut self,
        _matches: &mut ArgMatches,
    ) -> Result<(), clap::error::Error> {
        panic!("You should not directly call BoundaryConfigArgs::update_from_arg_matches_mut")
    }
}

fn get_base_name(i: &usize, variable: &str) -> &'static str {
    let base_name = format!("{}-{}", variable.to_lowercase(), i + 1);
    Box::leak(base_name.into_boxed_str())
}

fn get_tidal_db_base_name(i: &usize, variable: &str) -> &'static str {
    let base_name = format!("{}-{}-tidal-db", variable.to_lowercase(), i + 1);
    Box::leak(base_name.into_boxed_str())
}

fn get_all_tidal_constituents_base_name(i: &usize, variable: &str) -> &'static str {
    let base_name = format!("{}-{}-all", variable.to_lowercase(), i + 1);
    Box::leak(base_name.into_boxed_str())
}
fn get_major_tidal_constituents_base_name(i: &usize, variable: &str) -> &'static str {
    let base_name = format!("{}-{}-major", variable.to_lowercase(), i + 1);
    Box::leak(base_name.into_boxed_str())
}
fn get_minor_tidal_constituents_base_name(i: &usize, variable: &str) -> &'static str {
    let base_name = format!("{}-{}-minor", variable.to_lowercase(), i + 1);
    Box::leak(base_name.into_boxed_str())
}
fn get_constituent_flag_base_name(i: &u32, variable: &str, constituent: &str) -> &'static str {
    let base_name = format!("{}-{}-{}", variable.to_lowercase(), i + 1, constituent);
    Box::leak(base_name.into_boxed_str())
}

fn get_elev_th_base_name(i: &usize) -> &'static str {
    let base_name = format!("elev-th-{}", i + 1);
    Box::leak(base_name.into_boxed_str())
}
fn get_baroclinic_db_base_name(i: &usize, variable: &str) -> &'static str {
    let base_name = format!("{}-{}-baroclinic-db", variable.to_lowercase(), i + 1);
    Box::leak(base_name.into_boxed_str())
}
fn get_variable_help(i: &usize, variable: &str) -> &'static str {
    let base_name = format!(
        "Sets the forcing type for {} on boundary with id = {}",
        variable.to_lowercase(),
        i + 1
    );
    Box::leak(base_name.into_boxed_str())
}

fn get_tidal_db_help(i: &usize, variable: &str) -> &'static str {
    let base_name = format!(
        "Sets the tidal database for {} on boundary with id = {}",
        variable.to_lowercase(),
        i + 1
    );
    Box::leak(base_name.into_boxed_str())
}

fn get_baroclinic_db_help(i: &usize, variable: &str) -> &'static str {
    let base_name = format!(
        "Sets the baroclinic database for {} on boundary with id = {}",
        variable.to_lowercase(),
        i + 1
    );
    Box::leak(base_name.into_boxed_str())
}
fn get_elev_th_help(i: &usize) -> &'static str {
    let base_name = format!(
        "Path to elev.th file. Required if using elevation of type 1 on boundary id {} ",
        i + 1
    );
    Box::leak(base_name.into_boxed_str())
}

#[derive(EnumIter, AsRefStr, Debug)]
enum PossibleBoundaryVariables {
    Elevation,
    Velocity,
    Temperature,
    Salinity,
}

#[derive(Debug)]
enum ElevationConfigType {
    UniformTimeSeries,
    ConstantValue,
    Tides,
    SpaceVaryingTimeSeries,
    TidesAndSpaceVaryingTimeSeries,
    EqualToZero,
}

fn get_elevation_bctypes_map() -> LinkedHashMap<String, ElevationConfigType> {
    let tuples = [
        ("1".to_string(), ElevationConfigType::UniformTimeSeries),
        ("2".to_string(), ElevationConfigType::ConstantValue),
        ("3".to_string(), ElevationConfigType::Tides),
        ("4".to_string(), ElevationConfigType::SpaceVaryingTimeSeries),
        (
            "5".to_string(),
            ElevationConfigType::TidesAndSpaceVaryingTimeSeries,
        ),
        ("-1".to_string(), ElevationConfigType::EqualToZero),
    ];
    let map: LinkedHashMap<String, ElevationConfigType> = tuples.into_iter().collect();
    map
}

#[derive(Debug)]
enum VelocityConfigType {
    UniformTimeSeries,
    ConstantValue,
    Tides,
    SpaceVaryingTimeSeries,
    TidesAndSpaceVaryingTimeSeries,
    Flather,
}
fn get_velocity_bctypes_map() -> LinkedHashMap<String, VelocityConfigType> {
    let tuples = [
        ("1".to_string(), VelocityConfigType::UniformTimeSeries),
        ("2".to_string(), VelocityConfigType::ConstantValue),
        ("3".to_string(), VelocityConfigType::Tides),
        ("4".to_string(), VelocityConfigType::SpaceVaryingTimeSeries),
        (
            "5".to_string(),
            VelocityConfigType::TidesAndSpaceVaryingTimeSeries,
        ),
        ("-1".to_string(), VelocityConfigType::Flather),
    ];
    let map: LinkedHashMap<String, VelocityConfigType> = tuples.into_iter().collect();
    map
}
#[derive(Debug)]
enum TemperatureConfigType {
    RelaxToUniformTimeSeries,
    RelaxToConstantValue,
    RelaxToInitialConditions,
    RelaxToSpaceVaryingTimeSeries,
}
fn get_temperature_bctypes_map() -> LinkedHashMap<String, TemperatureConfigType> {
    let tuples = [
        (
            "1".to_string(),
            TemperatureConfigType::RelaxToUniformTimeSeries,
        ),
        ("2".to_string(), TemperatureConfigType::RelaxToConstantValue),
        (
            "3".to_string(),
            TemperatureConfigType::RelaxToInitialConditions,
        ),
        (
            "4".to_string(),
            TemperatureConfigType::RelaxToSpaceVaryingTimeSeries,
        ),
    ];
    let map: LinkedHashMap<String, TemperatureConfigType> = tuples.into_iter().collect();
    map
}
#[derive(Debug)]
enum SalinityConfigType {
    RelaxToUniformTimeSeries,
    RelaxToConstantValue,
    RelaxToInitialConditions,
    RelaxToSpaceVaryingTimeSeries,
}
fn get_salinity_bctypes_map() -> LinkedHashMap<String, SalinityConfigType> {
    let tuples = [
        (
            "1".to_string(),
            SalinityConfigType::RelaxToUniformTimeSeries,
        ),
        ("2".to_string(), SalinityConfigType::RelaxToConstantValue),
        (
            "3".to_string(),
            SalinityConfigType::RelaxToInitialConditions,
        ),
        (
            "4".to_string(),
            SalinityConfigType::RelaxToSpaceVaryingTimeSeries,
        ),
    ];
    let map: LinkedHashMap<String, SalinityConfigType> = tuples.into_iter().collect();
    map
}
fn get_leaked_string(k: String) -> &'static str {
    Box::leak(k.into_boxed_str())
}
enum TidalDbConfigType {
    TPXO,
    FES,
    HAMTIDE,
}

fn get_tidal_db_possible_values_map() -> LinkedHashMap<String, TidalDbConfigType> {
    LinkedHashMap::from_iter([
        ("tpxo".to_string(), TidalDbConfigType::TPXO),
        ("fes".to_string(), TidalDbConfigType::FES),
        ("hamtide".to_string(), TidalDbConfigType::HAMTIDE),
    ])
}

fn get_tidal_db_possible_values() -> PossibleValuesParser {
    let keys_vec: Vec<&str> = get_tidal_db_possible_values_map()
        .keys()
        .map(|k| get_leaked_string(k.to_string()))
        .collect();
    PossibleValuesParser::new(keys_vec)
}

enum BaroclinicDbConfigType {
    HYCOM,
}
fn get_baroclinic_db_possible_values_map() -> LinkedHashMap<String, BaroclinicDbConfigType> {
    LinkedHashMap::from_iter([("hycom".to_string(), BaroclinicDbConfigType::HYCOM)])
}

fn get_baroclinic_db_possible_values() -> PossibleValuesParser {
    let keys_vec: Vec<&str> = get_baroclinic_db_possible_values_map()
        .keys()
        .map(|k| get_leaked_string(k.to_string()))
        .collect();
    PossibleValuesParser::new(keys_vec)
}

fn get_variable_possible_values(variable: &PossibleBoundaryVariables) -> PossibleValuesParser {
    match variable {
        &PossibleBoundaryVariables::Elevation => {
            let elev_map = get_elevation_bctypes_map();
            let keys_vec: Vec<&str> = elev_map
                .keys()
                .map(|k| get_leaked_string(k.to_string()))
                .collect();
            PossibleValuesParser::new(keys_vec)
        }
        &PossibleBoundaryVariables::Velocity => {
            let vel_map = get_velocity_bctypes_map();
            let keys_vec: Vec<&str> = vel_map
                .keys()
                .map(|k| get_leaked_string(k.to_string()))
                .collect();
            PossibleValuesParser::new(keys_vec)
        }
        &PossibleBoundaryVariables::Temperature => {
            let keys_vec: Vec<&str> = get_temperature_bctypes_map()
                .keys()
                .map(|k| get_leaked_string(k.to_string()))
                .collect();
            PossibleValuesParser::new(keys_vec)
        }
        &PossibleBoundaryVariables::Salinity => {
            let keys_vec: Vec<&str> = get_salinity_bctypes_map()
                .keys()
                .map(|k| get_leaked_string(k.to_string()))
                .collect();
            PossibleValuesParser::new(keys_vec)
        }
    }
}

impl Args for BoundaryConfigArgs {
    fn augment_args(mut cmd: Command) -> Command {
        #[derive(Parser, Debug)]
        #[clap(disable_help_flag = true, ignore_errors = true)]
        struct TempCli {
            hgrid_path: PathBuf,
        }
        let cli = TempCli::parse();
        let hgrid = Hgrid::try_from(&cli.hgrid_path)
            .expect(format!("Unable to open hgrid file from path: {:?}", cli.hgrid_path).as_str());
        let boundaries = &hgrid
            .boundaries()
            .expect("The mesh file has no defined boundaries.");
        let open_boundaries = boundaries
            .open()
            .expect("The mesh has no defined open boundaries.");
        for (i, _bnd) in open_boundaries.iter().enumerate() {
            for var in PossibleBoundaryVariables::iter() {
                let base_name = get_base_name(&i, var.as_ref());
                let varhelp = get_variable_help(&i, var.as_ref());
                let possible_values = get_variable_possible_values(&var);
                cmd = cmd.arg(
                    Arg::new(base_name)
                        .long(base_name)
                        .help(varhelp)
                        .value_parser(possible_values),
                );
                let mut tmpcmd = cmd.clone();
                tmpcmd = tmpcmd.disable_help_flag(true);
                tmpcmd = tmpcmd.ignore_errors(true);
                let matches = tmpcmd.get_matches();
                match var {
                    PossibleBoundaryVariables::Elevation | PossibleBoundaryVariables::Velocity => {
                        let tidal_db_base_name = get_tidal_db_base_name(&i, var.as_ref());
                        let mut is_required_flag = false;
                        if let Some(argument_value) = matches.get_one::<String>(base_name) {
                            if argument_value == "3" || argument_value == "5" {
                                is_required_flag = true;
                            }
                        }
                        let tidal_db_possible_values = get_tidal_db_possible_values();
                        let tidal_help = get_tidal_db_help(&i, var.as_ref());
                        cmd = cmd.arg(
                            Arg::new(tidal_db_base_name)
                                .long(tidal_db_base_name)
                                .value_parser(tidal_db_possible_values)
                                .help(tidal_help)
                                .required(is_required_flag),
                        );
                        let all_tidal_constituents_base_name =
                            get_all_tidal_constituents_base_name(&i, var.as_ref());
                        let major_tidal_constituents_base_name =
                            get_major_tidal_constituents_base_name(&i, var.as_ref());
                        let minor_tidal_constituents_base_name =
                            get_minor_tidal_constituents_base_name(&i, var.as_ref());
                        cmd = cmd.arg(
                            Arg::new(all_tidal_constituents_base_name)
                                .long(all_tidal_constituents_base_name)
                                .action(clap::ArgAction::SetTrue)
                                .conflicts_with(major_tidal_constituents_base_name)
                                .conflicts_with(minor_tidal_constituents_base_name),
                        );
                        cmd = cmd.arg(
                            Arg::new(major_tidal_constituents_base_name)
                                .long(major_tidal_constituents_base_name)
                                .action(clap::ArgAction::SetTrue)
                                .conflicts_with(all_tidal_constituents_base_name)
                                .conflicts_with(minor_tidal_constituents_base_name),
                        );
                        cmd = cmd.arg(
                            Arg::new(minor_tidal_constituents_base_name)
                                .long(minor_tidal_constituents_base_name)
                                .action(clap::ArgAction::SetTrue)
                                .conflicts_with(all_tidal_constituents_base_name)
                                .conflicts_with(major_tidal_constituents_base_name),
                        );
                        let constituents_name_iter = match var {
                            PossibleBoundaryVariables::Elevation => {
                                tides::ConstituentsConfig::field_names()
                            }
                            PossibleBoundaryVariables::Velocity => {
                                tides::ConstituentsConfig::field_names()
                            }
                            _ => panic!("Unreachable!"),
                        };
                        for constituent_name in constituents_name_iter.iter() {
                            let constituent_flag_base_name = get_constituent_flag_base_name(
                                &(i as u32),
                                var.as_ref(),
                                &constituent_name,
                            );
                            cmd = cmd.arg(
                                Arg::new(constituent_flag_base_name)
                                    .long(constituent_flag_base_name)
                                    .action(clap::ArgAction::SetTrue)
                                    .conflicts_with(all_tidal_constituents_base_name)
                                    .conflicts_with(major_tidal_constituents_base_name)
                                    .conflicts_with(minor_tidal_constituents_base_name),
                            );
                        }
                    }
                    PossibleBoundaryVariables::Temperature
                    | PossibleBoundaryVariables::Salinity => {}
                }
                let baroclinic_db_base_name = get_baroclinic_db_base_name(&i, var.as_ref());
                let mut is_required_flag = false;
                if let Some(argument_value) = matches.get_one::<String>(base_name) {
                    if argument_value == "4" || argument_value == "5" {
                        is_required_flag = true;
                    }
                }
                let baroclinic_db_help = get_baroclinic_db_help(&i, var.as_ref());
                let baroclinic_db_possible_values = get_baroclinic_db_possible_values();
                cmd = cmd.arg(
                    Arg::new(baroclinic_db_base_name)
                        .long(baroclinic_db_base_name)
                        .help(baroclinic_db_help)
                        .value_parser(baroclinic_db_possible_values)
                        .required(is_required_flag),
                );
                match var {
                    PossibleBoundaryVariables::Elevation => {
                        let elev_th_base_name = get_elev_th_base_name(&i);
                        let elev_th_help = get_elev_th_help(&i);
                        let mut is_required_flag = false;
                        if let Some(argument_value) = matches.get_one::<String>(base_name) {
                            if argument_value == "1" {
                                is_required_flag = true;
                            }
                        }
                        cmd = cmd.arg(
                            Arg::new(elev_th_base_name)
                                .long(elev_th_base_name)
                                .value_parser(clap::value_parser!(PathBuf))
                                .help(elev_th_help)
                                .required(is_required_flag),
                        );
                    }
                    _ => {}
                }
            }
        }
        HGRID.with(|h| {
            *h.borrow_mut() = Some(hgrid);
        });
        cmd
    }
    fn augment_args_for_update(_cmd: Command) -> Command {
        unimplemented!()
    }
}

fn entrypoint() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let hgrid = HGRID.with(|h| h.borrow().clone()).unwrap();

    // Build the boundary forcing configuration using the new API
    let mut boundary_configs = BTreeMap::new();

    // Create per-boundary configurations
    let num_boundaries = hgrid
        .boundaries()
        .and_then(|b| b.open())
        .map(|ob| ob.nodes_ids().len())
        .unwrap_or(0);

    for boundary_id in 0..num_boundaries {
        let boundary_id = boundary_id as u32;
        let mut config = OpenBoundaryForcingConfig::default();

        // Set elevation forcing if present
        if let Some(ref elevation_map) = cli.boundary_config.elevation {
            if let Some(elevation_config) = elevation_map.get(&boundary_id) {
                config.elevation = Some(elevation_config.clone());
            }
        }

        // Set velocity forcing if present
        if let Some(ref velocity_map) = cli.boundary_config.velocity {
            if let Some(velocity_config) = velocity_map.get(&boundary_id) {
                config.velocity = Some(velocity_config.clone());
            }
        }

        // Set temperature forcing if present
        if let Some(ref temperature_map) = cli.boundary_config.temperature {
            if let Some(temperature_config) = temperature_map.get(&boundary_id) {
                config.temperature = Some(temperature_config.clone());
            }
        }

        // Set salinity forcing if present
        if let Some(ref salinity_map) = cli.boundary_config.salinity {
            if let Some(salinity_config) = salinity_map.get(&boundary_id) {
                config.salinity = Some(salinity_config.clone());
            }
        }

        // Only add to map if any forcing is configured
        if config.elevation.is_some()
            || config.velocity.is_some()
            || config.temperature.is_some()
            || config.salinity.is_some()
        {
            boundary_configs.insert(boundary_id, config);
        }
    }

    // Create the boundary forcing configuration
    let boundary_forcing_config = if boundary_configs.is_empty() {
        // No boundary configurations specified
        OpenBoundaryForcings::PerBoundary(BTreeMap::new())
    } else {
        OpenBoundaryForcings::PerBoundary(boundary_configs)
    };

    let mut builder = BctidesBuilder::default();
    let run_duration =
        Duration::try_seconds(cli.run_duration.as_secs().try_into().unwrap()).unwrap();
    let bctides = builder
        .start_date(&cli.start_date)
        .hgrid(&hgrid)
        .run_duration(&run_duration)
        .tidal_potential_cutoff_depth(cli.tidal_potential_cutoff_depth)
        .open_boundary_forcing_config(&boundary_forcing_config)
        .build()?;
    println!("{}", &bctides);
    Ok(())
}

fn main() -> ExitCode {
    match entrypoint() {
        Err(e) => {
            eprintln!("Error: {}", e);
            // Print the error chain for better debugging
            let mut source = e.source();
            while let Some(err) = source {
                eprintln!("  Caused by: {}", err);
                source = err.source();
            }
            ExitCode::FAILURE
        }
        Ok(_) => ExitCode::SUCCESS,
    }
}

