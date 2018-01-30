//! Type for holding gem experience.

use std::fmt;
use std::ops::{Add, AddAssign};

use separator::Separatable;


/// Experience gained by a gem.
#[derive(Clone, Copy)]
pub struct Experience {
    /// Experience for current level earned so far.
    /// This is always lower than total.
    current: u64,
    /// Total experience before the next level.
    total: u64,
}

impl Experience {
    /// Create an `Experience` object with given current & total values.
    #[inline]
    pub fn new(current: u64, total: u64) -> Self {
        assert!(total > 0, "Total experience cannot be zero");
        assert!(current <= total, "Current experience must not be greater than total");
        Experience { current, total }
    }

    /// Create an `Experience` object which represents a 0/total state.
    #[inline]
    pub fn zero_out_of(total: u64) -> Self {
        assert!(total > 0, "Total experience cannot be zero");
        Experience { current: 0, total }
    }

    /// Create an `Experience` object which represents
    /// a filled experience bar with given total.
    ///
    /// Such state correspond to a skill gem that is deliberately kept at a lower level.
    #[inline]
    pub fn full(total: u64) -> Self {
        assert!(total > 0, "Total experience cannot be zero");
        Experience { current: total, total }
    }
}

impl Experience {
    /// Experience for current level earned so far.
    /// This is always lower than total.
    #[inline]
    pub fn current(&self) -> u64 {
        self.current
    }

    /// Total experience before the next level.
    #[inline]
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Whether the experience "bar" is completely empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.current == 0
    }

    /// Whether the experience "bar" is fully filled.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.current == self.total
    }

    /// Experience as a 0..=1 fraction.
    pub fn fraction(&self) -> f64 {
        self.current as f64 / self.total as f64
    }

    /// Experience as a percentage value.
    pub fn percentage(&self) -> f64 {
        self.fraction() * 100.0
    }
}

impl Add<u64> for Experience {
    type Output = Experience;
    fn add(mut self, rhs: u64) -> Self {
        self += rhs; self
    }
}
impl AddAssign<u64> for Experience {
    fn add_assign(&mut self, rhs: u64) {
        let new = self.total.min(self.current + rhs);
        self.current = new;
    }
}

impl fmt::Display for Experience {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}/{}", self.current.separated_string(), self.total.separated_string())
    }
}

impl fmt::Debug for Experience {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_tuple("Experience")
            .field(&self.current)
            .field(&self.total)
            .finish()
    }
}
