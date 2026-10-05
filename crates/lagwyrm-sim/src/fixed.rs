//! Q16.16 fixed-point numbers for authoritative state.
//!
//! Integer arithmetic gives the same bits on every CPU, compiler, and
//! optimisation level. That is the whole point: see QUESTIONS.md for why
//! `f32` is a risk here.

use std::fmt;
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

/// A signed Q16.16 fixed-point number: 16 integer bits, 16 fractional bits.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fx(i32);

impl Fx {
    pub const FRAC_BITS: u32 = 16;
    pub const ONE: Fx = Fx(1 << Self::FRAC_BITS);
    pub const ZERO: Fx = Fx(0);

    /// Builds a value from its raw bit pattern.
    pub const fn from_raw(raw: i32) -> Fx {
        Fx(raw)
    }

    /// The raw bit pattern. Stable across platforms, so safe to hash or send.
    pub const fn raw(self) -> i32 {
        self.0
    }

    pub const fn from_int(n: i32) -> Fx {
        Fx(n << Self::FRAC_BITS)
    }

    /// `num / den`, rounded toward zero.
    pub const fn from_ratio(num: i32, den: i32) -> Fx {
        Fx((((num as i64) << Self::FRAC_BITS) / den as i64) as i32)
    }

    /// Integer part, rounded toward negative infinity.
    pub const fn floor_int(self) -> i32 {
        self.0 >> Self::FRAC_BITS
    }

    pub fn clamp(self, lo: Fx, hi: Fx) -> Fx {
        Fx(self.0.clamp(lo.0, hi.0))
    }

    pub fn abs(self) -> Fx {
        Fx(self.0.abs())
    }

    /// Lossy conversion for display and metrics only. Never feed it back in.
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / (1u64 << Self::FRAC_BITS) as f64
    }
}

impl Add for Fx {
    type Output = Fx;
    fn add(self, rhs: Fx) -> Fx {
        Fx(self.0 + rhs.0)
    }
}

impl AddAssign for Fx {
    fn add_assign(&mut self, rhs: Fx) {
        self.0 += rhs.0;
    }
}

impl Sub for Fx {
    type Output = Fx;
    fn sub(self, rhs: Fx) -> Fx {
        Fx(self.0 - rhs.0)
    }
}

impl SubAssign for Fx {
    fn sub_assign(&mut self, rhs: Fx) {
        self.0 -= rhs.0;
    }
}

impl Neg for Fx {
    type Output = Fx;
    fn neg(self) -> Fx {
        Fx(-self.0)
    }
}

impl Mul for Fx {
    type Output = Fx;
    fn mul(self, rhs: Fx) -> Fx {
        Fx(((self.0 as i64 * rhs.0 as i64) >> Self::FRAC_BITS) as i32)
    }
}

/// Scale by a small integer, e.g. a direction in `-1..=1`.
impl Mul<i32> for Fx {
    type Output = Fx;
    fn mul(self, rhs: i32) -> Fx {
        Fx(self.0 * rhs)
    }
}

impl fmt::Debug for Fx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fx({})", self.to_f64())
    }
}

impl fmt::Display for Fx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.to_f64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_round_trips() {
        let a = Fx::from_int(3);
        let b = Fx::from_ratio(1, 2);
        assert_eq!((a + b).to_f64(), 3.5);
        assert_eq!((a - b).to_f64(), 2.5);
        assert_eq!((a * b).to_f64(), 1.5);
        assert_eq!((b * -1).to_f64(), -0.5);
        assert_eq!(Fx::from_ratio(-3, 2).floor_int(), -2);
    }
}
