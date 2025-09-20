// f90nmlrs/src/findex.rs

//! Column-major Fortran iterator of indices across multiple dimensions.
//!
//! This module provides utilities for handling Fortran-style array indexing,
//! including support for custom start indices, strides, and multi-dimensional arrays.

use crate::error::{F90nmlError, Result};

/// Represents a single dimension's indexing information.
#[derive(Debug, Clone, PartialEq)]
pub struct IndexBound {
    /// Starting index (None means implicit)
    pub start: Option<i32>,
    /// Ending index (None means implicit)
    pub end: Option<i32>,
    /// Stride (None means 1)
    pub stride: Option<i32>,
}

impl IndexBound {
    /// Create a new index bound with explicit start and end.
    pub fn new(start: Option<i32>, end: Option<i32>, stride: Option<i32>) -> Self {
        Self { start, end, stride }
    }

    /// Create a simple range from start to end.
    pub fn range(start: i32, end: i32) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
            stride: None,
        }
    }

    /// Create a single index.
    pub fn single(index: i32) -> Self {
        Self {
            start: Some(index),
            end: Some(index),
            stride: None,
        }
    }

    /// Create an implicit range (:).
    pub fn implicit() -> Self {
        Self {
            start: None,
            end: None,
            stride: None,
        }
    }

    /// Get the effective start index.
    pub fn effective_start(&self, default: i32) -> i32 {
        self.start.unwrap_or(default)
    }

    /// Get the effective stride.
    pub fn effective_stride(&self) -> i32 {
        self.stride.unwrap_or(1)
    }

    /// Calculate the number of elements in this dimension.
    pub fn size(&self, default_start: i32, default_end: Option<i32>) -> Option<usize> {
        let start = self.effective_start(default_start);
        let stride = self.effective_stride();

        if stride == 0 {
            return None; // Invalid stride
        }

        let end = match (self.end, default_end) {
            (Some(e), _) => e,
            (None, Some(e)) => e,
            (None, None) => return None, // Cannot determine size
        };

        if stride > 0 && end >= start {
            Some(((end - start) / stride + 1) as usize)
        } else if stride < 0 && end <= start {
            Some(((start - end) / (-stride) + 1) as usize)
        } else {
            Some(0) // Empty range
        }
    }
}

/// Column-major multidimensional index iterator for Fortran-style arrays.
#[derive(Debug, Clone)]
pub struct FIndex {
    /// The bounds for each dimension
    _bounds: Vec<IndexBound>,
    /// Current position in each dimension
    current: Vec<i32>,
    /// Starting position for each dimension
    start: Vec<i32>,
    /// Ending position for each dimension
    end: Vec<i32>,
    /// Stride for each dimension
    step: Vec<i32>,
    /// Global starting index override
    first: Vec<i32>,
    /// Whether the iterator is exhausted
    exhausted: bool,
}

impl FIndex {
    /// Create a new FIndex iterator.
    pub fn new(bounds: Vec<IndexBound>, global_start: Option<i32>) -> Self {
        let len = bounds.len();
        let mut start = Vec::with_capacity(len);
        let mut end = Vec::with_capacity(len);
        let mut step = Vec::with_capacity(len);
        let mut first = Vec::with_capacity(len);

        for bound in &bounds {
            let default_start = global_start.unwrap_or(1);
            start.push(bound.effective_start(default_start));
            end.push(bound.end.unwrap_or(default_start)); // Will be adjusted later
            step.push(bound.effective_stride());
            first.push(bound.effective_start(default_start));
        }

        // Adjust first indices if global_start is provided
        if let Some(gs) = global_start {
            for f in first.iter_mut() {
                *f = (*f).min(gs);
            }
        }

        let current = start.clone();

        Self {
            _bounds: bounds,
            current,
            start,
            end,
            step,
            first,
            exhausted: false,
        }
    }

    /// Create an iterator for a simple 1D array.
    pub fn simple_1d(start: i32, end: i32) -> Self {
        let bounds = vec![IndexBound::range(start, end)];
        Self::new(bounds, None)
    }

    /// Create an iterator with implicit bounds.
    pub fn implicit(dimensions: usize) -> Self {
        let bounds = vec![IndexBound::implicit(); dimensions];
        Self::new(bounds, Some(1))
    }

    /// Get the current index tuple.
    pub fn current(&self) -> &[i32] {
        &self.current
    }

    /// Get the starting indices.
    pub fn start_indices(&self) -> &[i32] {
        &self.first
    }

    /// Check if the iterator is exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.exhausted
    }

    /// Reset the iterator to the beginning.
    pub fn reset(&mut self) {
        self.current = self.start.clone();
        self.exhausted = false;
    }

    /// Advance to the next index combination.
    pub fn advance(&mut self) -> Option<Vec<i32>> {
        if self.exhausted {
            return None;
        }

        let result = self.current.clone();

        // Advance using column-major (Fortran) ordering
        // Start from the first (leftmost) dimension, not the last
        let mut carry = true;
        for rank in 0..self.current.len() {
            if carry {
                let next_val = self.current[rank] + self.step[rank];

                // Check if we're still within bounds for this dimension
                if (self.step[rank] > 0 && next_val <= self.end[rank])
                    || (self.step[rank] < 0 && next_val >= self.end[rank])
                {
                    self.current[rank] = next_val;
                    carry = false;
                } else {
                    // Reset this dimension and carry to the next
                    self.current[rank] = self.start[rank];
                    // carry remains true
                }
            }
        }

        if carry {
            // We've exhausted all dimensions
            self.exhausted = true;
        }

        Some(result)
    }

    /// Convert a multi-dimensional index to a linear index.
    pub fn to_linear_index(&self, indices: &[i32], dimensions: &[usize]) -> Result<usize> {
        if indices.len() != dimensions.len() {
            return Err(F90nmlError::InvalidIndex {
                variable: "array".to_string(),
                index: format!("{:?}", indices),
                message: format!(
                    "Index has {} dimensions but array has {}",
                    indices.len(),
                    dimensions.len()
                ),
            });
        }

        let mut linear = 0;
        let mut multiplier = 1;

        // Column-major ordering (Fortran style)
        for (i, (&idx, &dim)) in indices.iter().zip(dimensions.iter()).enumerate() {
            let zero_based = idx - self.first[i];
            if zero_based < 0 || zero_based >= dim as i32 {
                return Err(F90nmlError::InvalidIndex {
                    variable: "array".to_string(),
                    index: format!("{:?}", indices),
                    message: format!(
                        "Index {} out of bounds for dimension {} (size {})",
                        idx, i, dim
                    ),
                });
            }
            linear += zero_based as usize * multiplier;
            multiplier *= dim;
        }

        Ok(linear)
    }

    /// Convert a linear index to multi-dimensional indices.
    pub fn from_linear_index(&self, linear: usize, dimensions: &[usize]) -> Vec<i32> {
        let mut indices = Vec::with_capacity(dimensions.len());
        let mut remaining = linear;

        // Column-major ordering (Fortran style)
        for (i, &dim) in dimensions.iter().enumerate() {
            let idx = remaining % dim;
            indices.push(idx as i32 + self.first[i]);
            remaining /= dim;
        }

        indices
    }
}

impl Iterator for FIndex {
    type Item = Vec<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance()
    }
}

/// Parse Fortran array indices from a string.
///
/// Examples:
/// - "1:10" -> range from 1 to 10
/// - "5" -> single index 5
/// - "1:10:2" -> range from 1 to 10 with stride 2
/// - ":" -> implicit range
pub fn parse_index_string(index_str: &str) -> Result<IndexBound> {
    let trimmed = index_str.trim();

    if trimmed == ":" {
        return Ok(IndexBound::implicit());
    }

    let parts: Vec<&str> = trimmed.split(':').collect();

    match parts.len() {
        1 => {
            // Single index
            let idx = parts[0]
                .parse::<i32>()
                .map_err(|_| F90nmlError::InvalidIndex {
                    variable: "array".to_string(),
                    index: index_str.to_string(),
                    message: "Invalid integer index".to_string(),
                })?;
            Ok(IndexBound::single(idx))
        }
        2 => {
            // Range start:end
            let start = if parts[0].is_empty() {
                None
            } else {
                Some(
                    parts[0]
                        .parse::<i32>()
                        .map_err(|_| F90nmlError::InvalidIndex {
                            variable: "array".to_string(),
                            index: index_str.to_string(),
                            message: "Invalid start index".to_string(),
                        })?,
                )
            };

            let end = if parts[1].is_empty() {
                None
            } else {
                Some(
                    parts[1]
                        .parse::<i32>()
                        .map_err(|_| F90nmlError::InvalidIndex {
                            variable: "array".to_string(),
                            index: index_str.to_string(),
                            message: "Invalid end index".to_string(),
                        })?,
                )
            };

            Ok(IndexBound::new(start, end, None))
        }
        3 => {
            // Range start:end:stride
            let start = if parts[0].is_empty() {
                None
            } else {
                Some(
                    parts[0]
                        .parse::<i32>()
                        .map_err(|_| F90nmlError::InvalidIndex {
                            variable: "array".to_string(),
                            index: index_str.to_string(),
                            message: "Invalid start index".to_string(),
                        })?,
                )
            };

            let end = if parts[1].is_empty() {
                None
            } else {
                Some(
                    parts[1]
                        .parse::<i32>()
                        .map_err(|_| F90nmlError::InvalidIndex {
                            variable: "array".to_string(),
                            index: index_str.to_string(),
                            message: "Invalid end index".to_string(),
                        })?,
                )
            };

            let stride = if parts[2].is_empty() {
                None
            } else {
                let s = parts[2]
                    .parse::<i32>()
                    .map_err(|_| F90nmlError::InvalidIndex {
                        variable: "array".to_string(),
                        index: index_str.to_string(),
                        message: "Invalid stride".to_string(),
                    })?;

                if s == 0 {
                    return Err(F90nmlError::InvalidIndex {
                        variable: "array".to_string(),
                        index: index_str.to_string(),
                        message: "Stride cannot be zero".to_string(),
                    });
                }

                Some(s)
            };

            Ok(IndexBound::new(start, end, stride))
        }
        _ => Err(F90nmlError::InvalidIndex {
            variable: "array".to_string(),
            index: index_str.to_string(),
            message: "Too many colons in index specification".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_bound_creation() {
        let bound = IndexBound::range(1, 10);
        assert_eq!(bound.start, Some(1));
        assert_eq!(bound.end, Some(10));
        assert_eq!(bound.stride, None);

        let bound = IndexBound::single(5);
        assert_eq!(bound.start, Some(5));
        assert_eq!(bound.end, Some(5));

        let bound = IndexBound::implicit();
        assert_eq!(bound.start, None);
        assert_eq!(bound.end, None);
    }

    #[test]
    fn test_index_bound_size() {
        let bound = IndexBound::range(1, 10);
        assert_eq!(bound.size(1, Some(10)), Some(10));

        let bound = IndexBound::new(Some(2), Some(10), Some(2));
        assert_eq!(bound.size(1, Some(10)), Some(5)); // 2, 4, 6, 8, 10

        let bound = IndexBound::single(5);
        assert_eq!(bound.size(1, Some(10)), Some(1));
    }

    #[test]
    fn test_findex_simple() {
        let bounds = vec![IndexBound::range(1, 3)];
        let findex = FIndex::new(bounds, None);

        let indices: Vec<Vec<i32>> = findex.collect();
        assert_eq!(indices, vec![vec![1], vec![2], vec![3]]);
    }

    #[test]
    fn test_findex_2d() {
        let bounds = vec![IndexBound::range(1, 2), IndexBound::range(1, 3)];
        let findex = FIndex::new(bounds, None);

        let indices: Vec<Vec<i32>> = findex.collect();
        // Column-major ordering: (1,1), (2,1), (1,2), (2,2), (1,3), (2,3)
        assert_eq!(
            indices,
            vec![
                vec![1, 1],
                vec![2, 1],
                vec![1, 2],
                vec![2, 2],
                vec![1, 3],
                vec![2, 3],
            ]
        );
    }

    #[test]
    fn test_parse_index_string() {
        assert_eq!(parse_index_string("5").unwrap(), IndexBound::single(5));
        assert_eq!(
            parse_index_string("1:10").unwrap(),
            IndexBound::range(1, 10)
        );
        assert_eq!(parse_index_string(":").unwrap(), IndexBound::implicit());

        let bound = parse_index_string("1:10:2").unwrap();
        assert_eq!(bound.start, Some(1));
        assert_eq!(bound.end, Some(10));
        assert_eq!(bound.stride, Some(2));

        assert!(parse_index_string("1:10:0").is_err()); // Zero stride
        assert!(parse_index_string("abc").is_err()); // Invalid integer
    }

    #[test]
    fn test_linear_index_conversion() {
        let bounds = vec![IndexBound::range(1, 2), IndexBound::range(1, 3)];
        let findex = FIndex::new(bounds, None);
        let dimensions = vec![2, 3];

        // Test forward conversion
        assert_eq!(findex.to_linear_index(&[1, 1], &dimensions).unwrap(), 0);
        assert_eq!(findex.to_linear_index(&[2, 1], &dimensions).unwrap(), 1);
        assert_eq!(findex.to_linear_index(&[1, 2], &dimensions).unwrap(), 2);
        assert_eq!(findex.to_linear_index(&[2, 3], &dimensions).unwrap(), 5);

        // Test reverse conversion
        assert_eq!(findex.from_linear_index(0, &dimensions), vec![1, 1]);
        assert_eq!(findex.from_linear_index(1, &dimensions), vec![2, 1]);
        assert_eq!(findex.from_linear_index(2, &dimensions), vec![1, 2]);
        assert_eq!(findex.from_linear_index(5, &dimensions), vec![2, 3]);
    }
}

