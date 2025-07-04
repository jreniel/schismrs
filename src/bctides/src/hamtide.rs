use crate::tides::TidalBoundaryInterpolator;
use crate::tides::TidalBoundaryInterpolatorError;
use ndarray::s;
use ndarray::Array1;
use ndarray::Array2;
use ndarray::Axis;
use ndarray::Dim;
use ndarray_stats::QuantileExt;
use netcdf;
use netcdf::types::BasicType;
use netcdf::types::VariableType;
use std::path::PathBuf;
use url::Url;

static HAMTIDE_DEFAULT_URL: &'static str =
    "https://icdc.cen.uni-hamburg.de/thredds/dodsC/ftpthredds/hamtide/";

#[derive(Debug)]
pub enum HamtideSources {
    API(Url),
    Directory(PathBuf),
}

pub struct HamtideInterpolator {
    source: HamtideSources,
    lon: Array1<f64>,
    lat: Array1<f64>,
}

impl HamtideInterpolator {
    pub fn from_api() -> Self {
        #[cfg(unix)]
        {
            use std::env;
            if env::var("HTTP.SSL.CAPATH").is_err() {
                netcdf::rc::set("HTTP.SSL.CAPATH", "/etc/ssl/certs/").unwrap();
            }
        }
        let url = Url::parse(HAMTIDE_DEFAULT_URL).expect("Unreachable error parsing hamtide URL.");
        let this_url = url.join("k2.hamtide11a.nc").unwrap();
        let nc = netcdf::open(&this_url.to_string()).unwrap();
        let var = nc.variable("LON").unwrap();
        let lon: Array1<f64> = var
            .get(..)
            .unwrap()
            .into_dimensionality::<Dim<[usize; 1]>>()
            .expect("Dimensionality mismatch");
        let var = nc.variable("LAT").unwrap();
        let lat: Array1<f64> = var
            .get(..)
            .unwrap()
            .into_dimensionality::<Dim<[usize; 1]>>()
            .expect("Dimensionality mismatch");
        HamtideInterpolator {
            source: HamtideSources::API(url),
            lon,
            lat,
        }
    }
    // pub fn from_path(path: &PathBuf) -> Self {
    //     unimplemented!("HamtideI")
    //     HamtideInterpolator {
    //         source: HamtideSources::Directory(path.to_path_buf()),
    //         // lon: Mutex::new(None),
    //         // lat: Mutex::new(None),
    //     }
    // }
    // pub fn lon(&self) -> &Array1<f64> {
    //     &self.lon
    // }
    // pub fn lat(&self) -> &Array1<f64> {
    //     &self.lat
    // }

    // def _get_resource(self, variable, constituent) -> Dataset:
    //     resource = self._resource[variable][constituent]
    //     if resource is not None:
    //         return Dataset(resource)
    //     if variable == 'elevation':
    //         fname = f'{constituent.lower()}.hamtide11a.nc'
    //     if variable == 'velocity':
    //         fname = f'HAMcurrent11a_{constituent.lower()}.nc'
    //     return Dataset(base_url + fname)
    //
    fn find_nearest_index(&self, coords: f64, xin: &Array1<f64>) -> isize {
        unimplemented!()
    }

    fn get_coords_slice(&self, coords: &Array2<f64>) -> (isize, isize, isize, isize) {
        // let lat_index = self.find_nearest_index(coords[[0, 0]], &self.lat);
        // let lon_index = self.find_nearest_index(coords[[0, 1]], &self.lon);
        // let minlat = coords.index_axis(Axis(0), 0).min().unwrap();
        let lat_array = coords.index_axis(Axis(1), 1);
        let maxlat = lat_array.max().unwrap();
        let minlat = lat_array.min().unwrap();
        let local_lons =
            coords
                .index_axis(Axis(1), 0)
                .mapv(|lon| if lon < 0.0 { lon + 360.0 } else { lon });
        let min_local_lon = local_lons.min().unwrap();
        let max_local_lon = local_lons.max().unwrap();

        // (lat_idx_start, lat_idx_end, lon_idx_start, lon_idx_end)
    }

    fn get_elevation_from_url(
        &self,
        url: &Url,
        constituent: &str,
        coords: &Array2<f64>,
    ) -> Result<Array1<f64>, TidalBoundaryInterpolatorError> {
        let ncname = format!("{}.hamtide11a.nc", constituent.to_lowercase());
        let this_url = url.join(&ncname).unwrap();
        let nc = match netcdf::open(&this_url.to_string()) {
            Ok(nc) => nc,
            Err(e) => return Err(TidalBoundaryInterpolatorError::NetcdfError(e)),
        };
        let (lat_start, lat_end, lon_start, lon_end) = self.get_coords_slice(coords);
        let var = nc.variable("AMPL").unwrap();
        let data = match var.vartype() {
            VariableType::Basic(BasicType::Float) => {
                var.get::<f32, _>(s![lat_start..lat_end, lon_start..lon_end])?
            }
            _ => panic!("Unreachable!"),
        };
        dbg!(&data);
        unimplemented!()
    }
    fn get_elevation_from_directory(
        path: &PathBuf,
        constituent: &str,
        coords: &Array2<f64>,
    ) -> Result<Array1<f64>, TidalBoundaryInterpolatorError> {
        // Err(TidalBoundaryInterpolatorError)
        unimplemented!();
    }
}

impl TidalBoundaryInterpolator for HamtideInterpolator {
    fn interpolate_elevation(
        &self,
        constituent: &str,
        coords: &Array2<f64>,
    ) -> Result<Array1<f64>, TidalBoundaryInterpolatorError> {
        let _ = match &self.source {
            HamtideSources::API(url) => self.get_elevation_from_url(url, constituent, coords),
            HamtideSources::Directory(path) => {
                Self::get_elevation_from_directory(path, constituent, coords)
            }
        };
        unimplemented!("interpolate elevation constituent");
    }
    fn interpolate_velocity(
        &self,
        constituent: &str,
        coords: &Array2<f64>,
    ) -> Result<Array1<f64>, TidalBoundaryInterpolatorError> {
        unimplemented!("interpolate velocity constituent");
    }
    // fn interpolate_velocity(&self) {}
}
