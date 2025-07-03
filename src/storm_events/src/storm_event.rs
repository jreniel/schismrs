use crate::atcf::ATCFFileDeck;
use chrono::{Datelike, Utc};
use datetime::Year;
use flate2::read::GzDecoder;
use polars::frame::DataFrame;
use polars::prelude::*;
use polars_lazy::prelude::*;
use regex::Regex;
use smartstring::alias::String as SmartString;
use std::io::Cursor;
use std::io::{BufRead, BufReader, Read};
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug)]
pub struct StormEvent {
    // name: String,
    // year: Year,
    track: DataFrame,
}

#[derive(Default)]
pub struct StormEventBuilder<'a> {
    file_deck: Option<&'a ATCFFileDeck>,
    storm_id: Option<&'a str>,
}

impl<'a> StormEventBuilder<'a> {
    pub fn build(&self) -> Result<StormEvent, StormEventBuilderError> {
        let storm_id = self.storm_id.ok_or_else(|| {
            StormEventBuilderError::UninitializedFieldError("storm_id".to_string())
        })?;
        let track = self.get_track_from_storm_id(storm_id)?;
        Ok(StormEvent { track })
    }

    pub fn storm_id(&mut self, storm_id: &'a str) -> &mut Self {
        self.storm_id = Some(storm_id);
        self
    }

    pub fn file_deck(&mut self, file_deck: &'a ATCFFileDeck) -> &mut Self {
        self.file_deck = Some(file_deck);
        self
    }
    fn get_track_from_storm_id(&self, storm_id: &str) -> Result<DataFrame, StormEventBuilderError> {
        let nhc_code = match (
            Regex::new(r"^[a-zA-Z]{2}\d{6}$")
                .unwrap()
                .is_match(&storm_id),
            Regex::new(r"^[a-zA-Z]*\d{4}$").unwrap().is_match(&storm_id),
        ) {
            (true, _) => {
                // this branch handles storm_id IS nhc_code already
                Ok(storm_id.to_string())
            }
            (_, true) => {
                // this branch handles NameYear format
                let storm_name = &storm_id[..storm_id.len() - 4];
                let year = Year(storm_id[storm_id.len() - 4..].parse::<i64>().map_err(|_| {
                    StormEventBuilderError::NoMatchingPatternForStormID(storm_id.to_string())
                })?);
                let inventory = Self::get_nhc_storm_inventory()?;
                Self::get_nhc_code_from_storm_name_and_year(&inventory, storm_name, &year)
            }
            (_, _) => Err(StormEventBuilderError::NoMatchingPatternForStormID(
                storm_id.to_string(),
            )),
        }?;
        Ok(self.get_track_from_nhc_code(&nhc_code)?)
    }

    fn get_track_from_nhc_code(&self, nhc_code: &str) -> Result<DataFrame, StormEventBuilderError> {
        let storm_year = Year(nhc_code[nhc_code.len() - 4..].parse::<i64>().map_err(|_| {
            StormEventBuilderError::NoMatchingPatternForNhcCode(nhc_code.to_string())
        })?);
        let current_year = Year(Utc::now().year() as i64);
        let url = "https://ftp.nhc.noaa.gov/atcf";
        let file_deck = self.file_deck.ok_or_else(|| {
            StormEventBuilderError::UninitializedFieldError("file_deck".to_string())
        })?;
        let suffix = match storm_year == current_year {
            true => match file_deck {
                ATCFFileDeck::ADVISORY => {
                    format!("aid_public/a{}.dat.gz", nhc_code.to_lowercase()).to_string()
                }
                ATCFFileDeck::BEST => format!("btk/b{}.dat", nhc_code.to_lowercase()).to_string(),
                ATCFFileDeck::FIXED => format!("fix/f{}.dat", nhc_code.to_lowercase()).to_string(),
            },
            false => match file_deck {
                ATCFFileDeck::ADVISORY => format!(
                    "archive/{}/a{}.dat.gz",
                    storm_year.0,
                    nhc_code.to_lowercase()
                )
                .to_string(),
                ATCFFileDeck::BEST => format!(
                    "archive/{}/b{}.dat.gz",
                    storm_year.0,
                    nhc_code.to_lowercase()
                )
                .to_string(),
                ATCFFileDeck::FIXED => format!(
                    "archive/{}/f{}.dat.gz",
                    storm_year.0,
                    nhc_code.to_lowercase()
                )
                .to_string(),
            },
        };
        let url = format!("{}/{}", url, suffix);
        let response = reqwest::blocking::get(&url)?.bytes()?;
        let mut basin = Vec::new();
        let mut cy = Vec::new();
        let mut yyyymmddhh = Vec::new();
        let mut technum_min = Vec::new();
        let mut tech = Vec::new();
        let mut tau = Vec::new();
        let mut latn_s = Vec::new();
        let mut lone_w = Vec::new();
        let mut vmax = Vec::new();
        let mut mslp = Vec::new();
        let mut ty = Vec::new();
        let mut rad = Vec::new();
        let mut windcode = Vec::new();
        let mut rad1 = Vec::new();
        let mut rad2 = Vec::new();
        let mut rad3 = Vec::new();
        let mut rad4 = Vec::new();
        let mut pouter = Vec::new();
        let mut router = Vec::new();
        let mut rmw = Vec::new();
        let mut gusts = Vec::new();
        let mut eye = Vec::new();
        let mut subregion = Vec::new();
        let mut maxseas = Vec::new();
        let mut initials = Vec::new();
        let mut dir = Vec::new();
        let mut speed = Vec::new();
        let mut stormname = Vec::new();
        let mut depth = Vec::new();
        let mut seas = Vec::new();
        let mut seascode = Vec::new();
        let mut seas1 = Vec::new();
        let mut seas2 = Vec::new();
        let mut seas3 = Vec::new();
        let mut seas4 = Vec::new();
        let reader: Box<dyn Read> = if url.ends_with("gz") {
            Box::new(GzDecoder::new(response.as_ref()))
        } else {
            Box::new(response.as_ref())
        };

        let buf_reader = BufReader::new(reader);
        for line in buf_reader.lines() {
            let line = line.unwrap();
            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            basin.push(parts.get(0).unwrap_or(&"").to_string());
            cy.push(parts.get(1).unwrap_or(&"").to_string());
            yyyymmddhh.push(parts.get(2).unwrap_or(&"").to_string());
            technum_min.push(parts.get(3).unwrap_or(&"").to_string());
            tech.push(parts.get(4).unwrap_or(&"").to_string());
            tau.push(parts.get(5).unwrap_or(&"").to_string());
            latn_s.push(parts.get(6).unwrap_or(&"").to_string());
            lone_w.push(parts.get(7).unwrap_or(&"").to_string());
            vmax.push(parts.get(8).unwrap_or(&"").to_string());
            mslp.push(parts.get(9).unwrap_or(&"").to_string());
            ty.push(parts.get(10).unwrap_or(&"").to_string());
            rad.push(parts.get(11).unwrap_or(&"").to_string());
            windcode.push(parts.get(12).unwrap_or(&"").to_string());
            rad1.push(parts.get(13).unwrap_or(&"").to_string());
            rad2.push(parts.get(14).unwrap_or(&"").to_string());
            rad3.push(parts.get(15).unwrap_or(&"").to_string());
            rad4.push(parts.get(16).unwrap_or(&"").to_string());
            pouter.push(parts.get(17).unwrap_or(&"").to_string());
            router.push(parts.get(18).unwrap_or(&"").to_string());
            rmw.push(parts.get(19).unwrap_or(&"").to_string());
            gusts.push(parts.get(20).unwrap_or(&"").to_string());
            eye.push(parts.get(21).unwrap_or(&"").to_string());
            subregion.push(parts.get(22).unwrap_or(&"").to_string());
            maxseas.push(parts.get(23).unwrap_or(&"").to_string());
            initials.push(parts.get(24).unwrap_or(&"").to_string());
            dir.push(parts.get(25).unwrap_or(&"").to_string());
            speed.push(parts.get(26).unwrap_or(&"").to_string());
            stormname.push(parts.get(27).unwrap_or(&"").to_string());
            depth.push(parts.get(28).unwrap_or(&"").to_string());
            seas.push(parts.get(29).unwrap_or(&"").to_string());
            seascode.push(parts.get(30).unwrap_or(&"").to_string());
            seas1.push(parts.get(31).unwrap_or(&"").to_string());
            seas2.push(parts.get(32).unwrap_or(&"").to_string());
            seas3.push(parts.get(33).unwrap_or(&"").to_string());
            seas4.push(parts.get(34).unwrap_or(&"").to_string());
        }
        let df = DataFrame::new(vec![
            Series::new("BASIN", basin),
            Series::new("CY", cy),
            Series::new("YYYYMMDDHH", yyyymmddhh),
            Series::new("TECHNUM/MIN", technum_min),
            Series::new("TECH", tech),
            Series::new("TAU", tau),
            Series::new("LatN/S", latn_s),
            Series::new("LonE/W", lone_w),
            Series::new("VMAX", vmax),
            Series::new("MSLP", mslp),
            Series::new("TY", ty),
            Series::new("RAD", rad),
            Series::new("WINDCODE", windcode),
            Series::new("RAD1", rad1),
            Series::new("RAD2", rad2),
            Series::new("RAD3", rad3),
            Series::new("RAD4", rad4),
            Series::new("POUTER", pouter),
            Series::new("ROUTER", router),
            Series::new("RMW", rmw),
            Series::new("GUSTS", gusts),
            Series::new("EYE", eye),
            Series::new("SUBREGION", subregion),
            Series::new("MAXSEAS", maxseas),
            Series::new("INITIALS", initials),
            Series::new("DIR", dir),
            Series::new("SPEED", speed),
            Series::new("STORMNAME", stormname),
            Series::new("DEPTH", depth),
            Series::new("SEAS", seas),
            Series::new("SEASCODE", seascode),
            Series::new("SEAS1", seas1),
            Series::new("SEAS2", seas2),
            Series::new("SEAS3", seas3),
            Series::new("SEAS4", seas4),
        ])?;
        Ok(df)
    }

    fn get_nhc_code_from_storm_name_and_year(
        inventory: &DataFrame,
        storm_name: &str,
        year: &Year,
    ) -> Result<String, StormEventBuilderError> {
        let some_coll = inventory
            .clone()
            .lazy()
            .filter(
                col("name")
                    .eq(lit(format!("{:>10}", storm_name.to_uppercase())))
                    .and(col("year").eq(lit(year.0))),
            )
            .collect()?;
        if some_coll.height() > 1 {
            return Err(StormEventBuilderError::MultipleMatchingData {
                storm_name: storm_name.to_owned(),
                year: year.0,
            });
        } else if some_coll.height() < 1 {
            return Err(StormEventBuilderError::NoMatchingData {
                storm_name: storm_name.to_owned(),
                year: year.0,
            });
        }

        let nhc_code_column = some_coll.column("nhc_code")?;

        let nhc_code_value = nhc_code_column.get(0);

        let nhc_code = nhc_code_value?.to_string();
        let nhc_code = nhc_code.trim_matches('\"').trim().to_string();
        Ok(nhc_code)
    }
    fn get_nhc_storm_inventory() -> Result<DataFrame, StormEventBuilderError> {
        let url = "https://ftp.nhc.noaa.gov/atcf/index/storm_list.txt";
        let response = reqwest::blocking::get(url)?.text()?;
        let cursor = Cursor::new(response);
        let mut schema = Schema::new();
        schema.with_column(SmartString::from("name"), DataType::String);
        schema.with_column(SmartString::from("basin"), DataType::String);
        schema.with_column(SmartString::from("2"), DataType::String);
        schema.with_column(SmartString::from("3"), DataType::String);
        schema.with_column(SmartString::from("4"), DataType::String);
        schema.with_column(SmartString::from("5"), DataType::String);
        schema.with_column(SmartString::from("6"), DataType::String);
        schema.with_column(SmartString::from("number"), DataType::String);
        schema.with_column(SmartString::from("year"), DataType::Int32);
        schema.with_column(SmartString::from("class"), DataType::String);
        schema.with_column(SmartString::from("10"), DataType::String);
        schema.with_column(SmartString::from("start_date"), DataType::String);
        schema.with_column(SmartString::from("end_date"), DataType::String);
        schema.with_column(SmartString::from("13"), DataType::String);
        schema.with_column(SmartString::from("14"), DataType::String);
        schema.with_column(SmartString::from("15"), DataType::String);
        schema.with_column(SmartString::from("16"), DataType::String);
        schema.with_column(SmartString::from("17"), DataType::String);
        schema.with_column(SmartString::from("source"), DataType::String);
        schema.with_column(SmartString::from("19"), DataType::String);
        schema.with_column(SmartString::from("nhc_code"), DataType::String);
        let schema = Arc::new(schema);
        let df = CsvReader::new(cursor)
            .with_schema(Some(schema))
            .has_header(true)
            .finish()
            .expect(&format!("Unreachable: polars should've been be able to parse this. Maybe something changed at the url {}", url));
        Ok(df)
    }
}

#[derive(Error, Debug)]
pub enum StormEventBuilderError {
    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error(
        "storm_id '{0}' does not match any known patterns for initialization \"Sandy2012\" or "
    )]
    NoMatchingPatternForStormID(String),

    #[error(
        "storm_id '{0}' does not match any known patterns for NHC code: r\"^[a-zA-Z]{{2}}\\d{{6}}$\"  "
    )]
    NoMatchingPatternForNhcCode(String),

    #[error("Polars error: {0}")]
    PolarsError(#[from] polars::prelude::PolarsError),

    #[error("No matching data found for storm: {storm_name}, year: {year}")]
    NoMatchingData { storm_name: String, year: i64 },

    #[error("Multiple matching data found for storm: {storm_name}, year: {year}")]
    MultipleMatchingData { storm_name: String, year: i64 },

    #[error("Unreachable: Unexpected multiple matching entries found for NHC code: {0}")]
    MultipleMatchingNhcCode(String),

    #[error("No matching entries found for NHC code: {0}")]
    NoMatchingNhcCode(String),

    #[error("Either storm_id or nhc_code must be set")]
    MissingArguments,

    #[error("Unitialized field on StormEventBuilder: {0}")]
    UninitializedFieldError(String),

    #[error("{0}")]
    MutuallyExclusiveArguments(String),
}
