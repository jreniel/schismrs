use crate::tides::TidalBoundaryInterpolator;
use crate::tides::TidalBoundaryInterpolatorError;
use ndarray::Array1;
use ndarray::Array2;
pub(crate) struct FESInterpolator {}

impl TidalBoundaryInterpolator for FESInterpolator {
    fn interpolate_elevation(
        &self,
        constituent: &str,
        coords: &Array2<f64>,
    ) -> Result<Array1<f64>, TidalBoundaryInterpolatorError> {
        unimplemented!("interpolate elevation constituent");
    }
    fn interpolate_velocity(
        &self,
        constituent: &str,
        coords: &Array2<f64>,
    ) -> Result<Array1<f64>, TidalBoundaryInterpolatorError> {
        unimplemented!("interpolate elevation constituent");
    }
    // fn interpolate_velocity(&self) {}
}
