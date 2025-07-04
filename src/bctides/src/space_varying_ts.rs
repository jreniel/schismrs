use anyhow::Result as AnyResult;
use chrono::{DateTime, Utc};
use schismrs_hgrid::Hgrid;
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum TimeSeriesDatabase {
    HYCOM,
}

#[derive(Debug)]
pub struct SpaceVaryingTimeSeriesConfig {
    data: BTreeMap<DateTime<Utc>, Vec<f64>>,
    database: TimeSeriesDatabase,
}

impl SpaceVaryingTimeSeriesConfig {
    pub fn from_hycom(hgrid: &Hgrid) -> AnyResult<Self> {
        todo!()
    }
}
