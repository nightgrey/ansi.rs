use std::num::{FpCategory, ParseIntError};
use std::ops::Neg;
use crate::number::Number;

pub trait Float: Number + Neg<Output = Self>
{
    /// Archimedes' constant (π)
    const PI: Self;

    /// The full circle constant (τ)
    ///
    /// Equal to 2π.
    const TAU: Self;

    /// The golden ratio (φ)
    const GOLDEN_RATIO: Self;

    /// The Euler-Mascheroni constant (γ)
    const EULER_GAMMA: Self;

    /// π/2
    const FRAC_PI_2: Self;

    /// π/3
    const FRAC_PI_3: Self;

    /// π/4
    const FRAC_PI_4: Self;

    /// π/6
    const FRAC_PI_6: Self;

    /// π/8
    const FRAC_PI_8: Self;

    /// 1/π
    const FRAC_1_PI: Self;

    /// 1/sqrt(π)
    const FRAC_1_SQRT_PI: Self;

    /// 1/sqrt(2π)
    const FRAC_1_SQRT_2PI: Self;

    /// 2/π
    const FRAC_2_PI: Self;

    /// 2/sqrt(π)
    const FRAC_2_SQRT_PI: Self;

    /// sqrt(2)
    const SQRT_2: Self;

    /// 1/sqrt(2)
    const FRAC_1_SQRT_2: Self;

    /// sqrt(3)
    const SQRT_3: Self;

    /// 1/sqrt(3)
    const FRAC_1_SQRT_3: Self;

    /// sqrt(5)
    const SQRT_5: Self;

    /// 1/sqrt(5)
    const FRAC_1_SQRT_5: Self;

    /// Euler's number (e)
    const E: Self;

    /// log<sub>2</sub>(e)
    const LOG2_E: Self;

    /// log<sub>2</sub>(10)
    const LOG2_10: Self;

    /// log<sub>10</sub>(e)
    const LOG10_E: Self;

    /// log<sub>10</sub>(2)
    const LOG10_2: Self;

    /// ln(2)
    const LN_2: Self;

    /// ln(10)
    const LN_10: Self;

    /// [Machine epsilon] value for `f32`.
    ///
    /// This is the difference between `1.0` and the next larger representable number.
    ///
    /// Equal to 2<sup>1&nbsp;&minus;&nbsp;[`MANTISSA_DIGITS`]</sup>.
    ///
    /// [Machine epsilon]: https://en.wikipedia.org/wiki/Machine_epsilon
    /// [`MANTISSA_DIGITS`]: f32::MANTISSA_DIGITS
    #[feature(assoc_int_consts)]
    const EPSILON: Self;

    /// Smallest finite `f32` value.
    ///
    /// Equal to &minus;[`MAX`].
    ///
    /// [`MAX`]: f32::MAX
    #[feature(assoc_int_consts)]
    const MIN: Self;
    /// Smallest positive normal `f32` value.
    ///
    /// Equal to 2<sup>[`MIN_EXP`]&nbsp;&minus;&nbsp;1</sup>.
    ///
    /// [`MIN_EXP`]: f32::MIN_EXP
    #[feature(assoc_int_consts)]
    const MIN_POSITIVE: Self;
    /// Largest finite `f32` value.
    ///
    /// Equal to
    /// (1&nbsp;&minus;&nbsp;2<sup>&minus;[`MANTISSA_DIGITS`]</sup>)&nbsp;2<sup>[`MAX_EXP`]</sup>.
    ///
    /// [`MANTISSA_DIGITS`]: f32::MANTISSA_DIGITS
    /// [`MAX_EXP`]: f32::MAX_EXP
    #[feature(assoc_int_consts)]
    const MAX: Self;

    /// Not a Number (NaN).
    ///
    /// Note that IEEE 754 doesn't define just a single NaN value; a plethora of bit patterns are
    /// considered to be NaN. Furthermore, the standard makes a difference between a "signaling" and
    /// a "quiet" NaN, and allows inspecting its "payload" (the unspecified bits in the bit pattern)
    /// and its sign. See the [specification of NaN bit patterns](f32#nan-bit-patterns) for more
    /// info.
    ///
    /// This constant is guaranteed to be a quiet NaN (on targets that follow the Rust assumptions
    /// that the quiet/signaling bit being set to 1 indicates a quiet NaN). Beyond that, nothing is
    /// guaranteed about the specific bit pattern chosen here: both payload and sign are arbitrary.
    /// The concrete bit pattern may change across Rust versions and target platforms.
    #[feature(assoc_int_consts)]
    const NAN: Self;

    /// Infinity (∞).
    #[feature(assoc_int_consts)]
    const INFINITY: Self;

    /// Negative infinity (−∞).
    #[feature(assoc_int_consts)]
    const NEG_INFINITY: Self;

    /// Returns `true` if this value is `NaN` and false otherwise.
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let nan = f64::NAN;
    /// let f = 7.0;
    ///
    /// assert!(nan.is_nan());
    /// assert!(!f.is_nan());
    /// ```
    fn is_nan(self) -> bool;

    /// Returns `true` if this value is positive infinity or negative infinity and
    /// false otherwise.
    ///
    /// ```
    /// use number::Float;
    /// use std::f32;
    ///
    /// let f = 7.0f32;
    /// let inf: f32 = Float::INFINITY;
    /// let neg_inf: f32 = Float::NEG_INFINITY;
    /// let nan: f32 = f32::NAN;
    ///
    /// assert!(!f.is_infinite());
    /// assert!(!nan.is_infinite());
    ///
    /// assert!(inf.is_infinite());
    /// assert!(neg_inf.is_infinite());
    /// ```
    fn is_infinite(self) -> bool;

    /// Returns `true` if this number is neither infinite nor `NaN`.
    ///
    /// ```
    /// use number::Float;
    /// use std::f32;
    ///
    /// let f = 7.0f32;
    /// let inf: f32 = Float::INFINITY;
    /// let neg_inf: f32 = Float::NEG_INFINITY;
    /// let nan: f32 = f32::NAN;
    ///
    /// assert!(f.is_finite());
    ///
    /// assert!(!nan.is_finite());
    /// assert!(!inf.is_finite());
    /// assert!(!neg_inf.is_finite());
    /// ```
    fn is_finite(self) -> bool;

    /// Returns `true` if the number is neither zero, infinite,
    /// [subnormal][subnormal], or `NaN`.
    ///
    /// ```
    /// use number::Float;
    /// use std::f32;
    ///
    /// let min = f32::MIN_POSITIVE; // 1.17549435e-38f32
    /// let max = f32::MAX;
    /// let lower_than_min = 1.0e-40_f32;
    /// let zero = 0.0f32;
    ///
    /// assert!(min.is_normal());
    /// assert!(max.is_normal());
    ///
    /// assert!(!zero.is_normal());
    /// assert!(!f32::NAN.is_normal());
    /// assert!(!f32::INFINITY.is_normal());
    /// // Values between `0` and `min` are Subnormal.
    /// assert!(!lower_than_min.is_normal());
    /// ```
    /// [subnormal]: http://en.wikipedia.org/wiki/Subnormal_number
    fn is_normal(self) -> bool;

    /// Returns `true` if the number is [subnormal].
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let min = f64::MIN_POSITIVE; // 2.2250738585072014e-308_f64
    /// let max = f64::MAX;
    /// let lower_than_min = 1.0e-308_f64;
    /// let zero = 0.0_f64;
    ///
    /// assert!(!min.is_subnormal());
    /// assert!(!max.is_subnormal());
    ///
    /// assert!(!zero.is_subnormal());
    /// assert!(!f64::NAN.is_subnormal());
    /// assert!(!f64::INFINITY.is_subnormal());
    /// // Values between `0` and `min` are Subnormal.
    /// assert!(lower_than_min.is_subnormal());
    /// ```
    /// [subnormal]: https://en.wikipedia.org/wiki/Subnormal_number
    #[inline]
    fn is_subnormal(self) -> bool;

    /// Returns the floating point category of the number. If only one property
    /// is going to be tested, it is generally faster to use the specific
    /// predicate instead.
    ///
    /// ```
    /// use number::Float;
    /// use std::num::FpCategory;
    /// use std::f32;
    ///
    /// let num = 12.4f32;
    /// let inf = f32::INFINITY;
    ///
    /// assert_eq!(num.classify(), FpCategory::Normal);
    /// assert_eq!(inf.classify(), FpCategory::Infinite);
    /// ```
    fn classify(self) -> FpCategory;

    /// Returns the largest integer less than or equal to a number.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let f = 3.99;
    /// let g = 3.0;
    ///
    /// assert_eq!(f.floor(), 3.0);
    /// assert_eq!(g.floor(), 3.0);
    /// ```
    fn floor(self) -> Self;

    /// Returns the smallest integer greater than or equal to a number.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let f = 3.01;
    /// let g = 4.0;
    ///
    /// assert_eq!(f.ceil(), 4.0);
    /// assert_eq!(g.ceil(), 4.0);
    /// ```
    fn ceil(self) -> Self;

    /// Returns the nearest integer to a number. Round half-way cases away from
    /// `0.0`.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let f = 3.3;
    /// let g = -3.3;
    ///
    /// assert_eq!(f.round(), 3.0);
    /// assert_eq!(g.round(), -3.0);
    /// ```
    fn round(self) -> Self;

    /// Return the integer part of a number.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let f = 3.3;
    /// let g = -3.7;
    ///
    /// assert_eq!(f.trunc(), 3.0);
    /// assert_eq!(g.trunc(), -3.0);
    /// ```
    fn trunc(self) -> Self;

    /// Returns the fractional part of a number.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let x = 3.5;
    /// let y = -3.5;
    /// let abs_difference_x = (x.fract() - 0.5).abs();
    /// let abs_difference_y = (y.fract() - (-0.5)).abs();
    ///
    /// assert!(abs_difference_x < 1e-10);
    /// assert!(abs_difference_y < 1e-10);
    /// ```
    fn fract(self) -> Self;

    /// Computes the absolute value of `self`. Returns `Float::nan()` if the
    /// number is `Float::nan()`.
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let x = 3.5;
    /// let y = -3.5;
    ///
    /// let abs_difference_x = (x.abs() - x).abs();
    /// let abs_difference_y = (y.abs() - (-y)).abs();
    ///
    /// assert!(abs_difference_x < 1e-10);
    /// assert!(abs_difference_y < 1e-10);
    ///
    /// assert!(f64::NAN.abs().is_nan());
    /// ```
    fn abs(self) -> Self;

    /// Returns a number that represents the sign of `self`.
    ///
    /// - `1.0` if the number is positive, `+0.0` or `Float::infinity()`
    /// - `-1.0` if the number is negative, `-0.0` or `Float::neg_infinity()`
    /// - `Float::nan()` if the number is `Float::nan()`
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let f = 3.5;
    ///
    /// assert_eq!(f.signum(), 1.0);
    /// assert_eq!(f64::NEG_INFINITY.signum(), -1.0);
    ///
    /// assert!(f64::NAN.signum().is_nan());
    /// ```
    fn signum(self) -> Self;

    /// Returns `true` if `self` is positive, including `+0.0`,
    /// `Float::infinity()`, and `Float::nan()`.
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let nan: f64 = f64::NAN;
    /// let neg_nan: f64 = -f64::NAN;
    ///
    /// let f = 7.0;
    /// let g = -7.0;
    ///
    /// assert!(f.is_sign_positive());
    /// assert!(!g.is_sign_positive());
    /// assert!(nan.is_sign_positive());
    /// assert!(!neg_nan.is_sign_positive());
    /// ```
    fn is_sign_positive(self) -> bool;

    /// Returns `true` if `self` is negative, including `-0.0`,
    /// `Float::neg_infinity()`, and `-Float::nan()`.
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let nan: f64 = f64::NAN;
    /// let neg_nan: f64 = -f64::NAN;
    ///
    /// let f = 7.0;
    /// let g = -7.0;
    ///
    /// assert!(!f.is_sign_negative());
    /// assert!(g.is_sign_negative());
    /// assert!(!nan.is_sign_negative());
    /// assert!(neg_nan.is_sign_negative());
    /// ```
    fn is_sign_negative(self) -> bool;

    /// Fused multiply-add. Computes `(self * a) + b` with only one rounding
    /// error, yielding a more accurate result than an unfused multiply-add.
    ///
    /// Using `mul_add` can be more performant than an unfused multiply-add if
    /// the target architecture has a dedicated `fma` CPU instruction.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let m = 10.0;
    /// let x = 4.0;
    /// let b = 60.0;
    ///
    /// // 100.0
    /// let abs_difference = (m.mul_add(x, b) - (m*x + b)).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn mul_add(self, a: Self, b: Self) -> Self;
    /// Take the reciprocal (inverse) of a number, `1/x`.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let x = 2.0;
    /// let abs_difference = (x.recip() - (1.0/x)).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn recip(self) -> Self;

    /// Raise a number to an integer power.
    ///
    /// Using this function is generally faster than using `powf`
    ///
    /// ```
    /// use number::Float;
    ///
    /// let x = 2.0;
    /// let abs_difference = (x.powi(2) - x*x).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn powi(self, n: i32) -> Self;

    /// Raise a number to a floating point power.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let x = 2.0;
    /// let abs_difference = (x.powf(2.0) - x*x).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn powf(self, n: Self) -> Self;

    /// Take the square root of a number.
    ///
    /// Returns NaN if `self` is a negative number.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let positive = 4.0;
    /// let negative = -4.0;
    ///
    /// let abs_difference = (positive.sqrt() - 2.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// assert!(negative.sqrt().is_nan());
    /// ```
    fn sqrt(self) -> Self;

    /// Returns `e^(self)`, (the exponential function).
    ///
    /// ```
    /// use number::Float;
    ///
    /// let one = 1.0;
    /// // e^1
    /// let e = one.exp();
    ///
    /// // ln(e) - 1 == 0
    /// let abs_difference = (e.ln() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn exp(self) -> Self;

    /// Returns `2^(self)`.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let f = 2.0;
    ///
    /// // 2^2 - 4 == 0
    /// let abs_difference = (f.exp2() - 4.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn exp2(self) -> Self;

    /// Returns the natural logarithm of the number.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let one = 1.0;
    /// // e^1
    /// let e = one.exp();
    ///
    /// // ln(e) - 1 == 0
    /// let abs_difference = (e.ln() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn ln(self) -> Self;

    /// Returns the logarithm of the number with respect to an arbitrary base.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let ten = 10.0;
    /// let two = 2.0;
    ///
    /// // log10(10) - 1 == 0
    /// let abs_difference_10 = (ten.log(10.0) - 1.0).abs();
    ///
    /// // log2(2) - 1 == 0
    /// let abs_difference_2 = (two.log(2.0) - 1.0).abs();
    ///
    /// assert!(abs_difference_10 < 1e-10);
    /// assert!(abs_difference_2 < 1e-10);
    /// ```
    fn log(self, base: Self) -> Self;

    /// Returns the base 2 logarithm of the number.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let two = 2.0;
    ///
    /// // log2(2) - 1 == 0
    /// let abs_difference = (two.log2() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn log2(self) -> Self;

    /// Returns the base 10 logarithm of the number.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let ten = 10.0;
    ///
    /// // log10(10) - 1 == 0
    /// let abs_difference = (ten.log10() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn log10(self) -> Self;

    /// Converts radians to degrees.
    ///
    /// ```
    /// use std::f64::consts;
    ///
    /// let angle = consts::PI;
    ///
    /// let abs_difference = (angle.to_degrees() - 180.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    fn to_degrees(self) -> Self;

    /// Converts degrees to radians.
    ///
    /// ```
    /// use std::f64::consts;
    ///
    /// let angle = 180.0_f64;
    ///
    /// let abs_difference = (angle.to_radians() - consts::PI).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    #[inline]
    fn to_radians(self) -> Self;

    /// The positive difference of two numbers.
    ///
    /// * If `self <= other`: `0:0`
    /// * Else: `self - other`
    ///
    /// ```
    /// use number::Float;
    ///
    /// let x = 3.0;
    /// let y = -3.0;
    ///
    /// let abs_difference_x = (x.abs_sub(1.0) - 2.0).abs();
    /// let abs_difference_y = (y.abs_sub(1.0) - 0.0).abs();
    ///
    /// assert!(abs_difference_x < 1e-10);
    /// assert!(abs_difference_y < 1e-10);
    /// ```
    fn abs_sub(self, other: Self) -> Self;

    /// Take the cubic root of a number.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let x = 8.0;
    ///
    /// // x^(1/3) - 2 == 0
    /// let abs_difference = (x.cbrt() - 2.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn cbrt(self) -> Self;

    /// Calculate the length of the hypotenuse of a right-angle triangle given
    /// legs of length `x` and `y`.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let x = 2.0;
    /// let y = 3.0;
    ///
    /// // sqrt(x^2 + y^2)
    /// let abs_difference = (x.hypot(y) - (x.powi(2) + y.powi(2)).sqrt()).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn hypot(self, other: Self) -> Self;

    /// Computes the sine of a number (in radians).
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let x = f64::consts::PI/2.0;
    ///
    /// let abs_difference = (x.sin() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn sin(self) -> Self;

    /// Computes the cosine of a number (in radians).
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let x = 2.0*f64::consts::PI;
    ///
    /// let abs_difference = (x.cos() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn cos(self) -> Self;

    /// Computes the tangent of a number (in radians).
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let x = f64::consts::PI/4.0;
    /// let abs_difference = (x.tan() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-14);
    /// ```
    fn tan(self) -> Self;

    /// Computes the arcsine of a number. Return value is in radians in
    /// the range [-pi/2, pi/2] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let f = f64::consts::PI / 2.0;
    ///
    /// // asin(sin(pi/2))
    /// let abs_difference = (f.sin().asin() - f64::consts::PI / 2.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn asin(self) -> Self;

    /// Computes the arccosine of a number. Return value is in radians in
    /// the range [0, pi] or NaN if the number is outside the range
    /// [-1, 1].
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let f = f64::consts::PI / 4.0;
    ///
    /// // acos(cos(pi/4))
    /// let abs_difference = (f.cos().acos() - f64::consts::PI / 4.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn acos(self) -> Self;

    /// Computes the arctangent of a number. Return value is in radians in the
    /// range [-pi/2, pi/2];
    ///
    /// ```
    /// use number::Float;
    ///
    /// let f = 1.0;
    ///
    /// // atan(tan(1))
    /// let abs_difference = (f.tan().atan() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn atan(self) -> Self;

    /// Computes the four quadrant arctangent of `self` (`y`) and `other` (`x`).
    ///
    /// * `x = 0`, `y = 0`: `0`
    /// * `x >= 0`: `arctan(y/x)` -> `[-pi/2, pi/2]`
    /// * `y >= 0`: `arctan(y/x) + pi` -> `(pi/2, pi]`
    /// * `y < 0`: `arctan(y/x) - pi` -> `(-pi, -pi/2)`
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let pi = f64::consts::PI;
    /// // All angles from horizontal right (+x)
    /// // 45 deg counter-clockwise
    /// let x1 = 3.0;
    /// let y1 = -3.0;
    ///
    /// // 135 deg clockwise
    /// let x2 = -3.0;
    /// let y2 = 3.0;
    ///
    /// let abs_difference_1 = (y1.atan2(x1) - (-pi/4.0)).abs();
    /// let abs_difference_2 = (y2.atan2(x2) - 3.0*pi/4.0).abs();
    ///
    /// assert!(abs_difference_1 < 1e-10);
    /// assert!(abs_difference_2 < 1e-10);
    /// ```
    fn atan2(self, other: Self) -> Self;

    /// Simultaneously computes the sine and cosine of the number, `x`. Returns
    /// `(sin(x), cos(x))`.
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let x = f64::consts::PI/4.0;
    /// let f = x.sin_cos();
    ///
    /// let abs_difference_0 = (f.0 - x.sin()).abs();
    /// let abs_difference_1 = (f.1 - x.cos()).abs();
    ///
    /// assert!(abs_difference_0 < 1e-10);
    /// assert!(abs_difference_0 < 1e-10);
    /// ```
    fn sin_cos(self) -> (Self, Self);

    /// Returns `e^(self) - 1` in a way that is accurate even if the
    /// number is close to zero.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let x = 7.0;
    ///
    /// // e^(ln(7)) - 1
    /// let abs_difference = (x.ln().exp_m1() - 6.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn exp_m1(self) -> Self;

    /// Returns `ln(1+n)` (natural logarithm) more accurately than if
    /// the operations were performed separately.
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let x = f64::consts::E - 1.0;
    ///
    /// // ln(1 + (e - 1)) == ln(e) == 1
    /// let abs_difference = (x.ln_1p() - 1.0).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn ln_1p(self) -> Self;

    /// Hyperbolic sine function.
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let e = f64::consts::E;
    /// let x = 1.0;
    ///
    /// let f = x.sinh();
    /// // Solving sinh() at 1 gives `(e^2-1)/(2e)`
    /// let g = (e*e - 1.0)/(2.0*e);
    /// let abs_difference = (f - g).abs();
    ///
    /// assert!(abs_difference < 1e-10);
    /// ```
    fn sinh(self) -> Self;

    /// Hyperbolic cosine function.
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let e = f64::consts::E;
    /// let x = 1.0;
    /// let f = x.cosh();
    /// // Solving cosh() at 1 gives this result
    /// let g = (e*e + 1.0)/(2.0*e);
    /// let abs_difference = (f - g).abs();
    ///
    /// // Same result
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    fn cosh(self) -> Self;

    /// Hyperbolic tangent function.
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let e = f64::consts::E;
    /// let x = 1.0;
    ///
    /// let f = x.tanh();
    /// // Solving tanh() at 1 gives `(1 - e^(-2))/(1 + e^(-2))`
    /// let g = (1.0 - e.powi(-2))/(1.0 + e.powi(-2));
    /// let abs_difference = (f - g).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    fn tanh(self) -> Self;

    /// Inverse hyperbolic sine function.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let x = 1.0;
    /// let f = x.sinh().asinh();
    ///
    /// let abs_difference = (f - x).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    fn asinh(self) -> Self;

    /// Inverse hyperbolic cosine function.
    ///
    /// ```
    /// use number::Float;
    ///
    /// let x = 1.0;
    /// let f = x.cosh().acosh();
    ///
    /// let abs_difference = (f - x).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    fn acosh(self) -> Self;

    /// Inverse hyperbolic tangent function.
    ///
    /// ```
    /// use number::Float;
    /// use std::f64;
    ///
    /// let e = f64::consts::E;
    /// let f = e.tanh().atanh();
    ///
    /// let abs_difference = (f - e).abs();
    ///
    /// assert!(abs_difference < 1.0e-10);
    /// ```
    fn atanh(self) -> Self;

    /// Returns a number composed of the magnitude of `self` and the sign of
    /// `sign`.
    ///
    /// Equal to `self` if the sign of `self` and `sign` are the same, otherwise
    /// equal to `-self`. If `self` is a `NAN`, then a `NAN` with the sign of
    /// `sign` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use number::Float;
    ///
    /// let f = 3.5_f32;
    ///
    /// assert_eq!(f.copysign(0.42), 3.5_f32);
    /// assert_eq!(f.copysign(-0.42), -3.5_f32);
    /// assert_eq!((-f).copysign(0.42), 3.5_f32);
    /// assert_eq!((-f).copysign(-0.42), -3.5_f32);
    ///
    /// assert!(f32::NAN.copysign(1.0).is_nan());
    /// ```
    fn copysign(self, sign: Self) -> Self;
}
macro_rules! impl_float {
    ($T:ident) => {
        impl Float for $T {
            const PI: Self = std::$T::consts::PI;

            const TAU: Self = std::$T::consts::TAU;

            const GOLDEN_RATIO: Self = std::$T::consts::GOLDEN_RATIO;

            const EULER_GAMMA: Self = std::$T::consts::EULER_GAMMA;

            const FRAC_PI_2: Self = std::$T::consts::FRAC_PI_2;

            const FRAC_PI_3: Self = std::$T::consts::FRAC_PI_3;

            const FRAC_PI_4: Self = std::$T::consts::FRAC_PI_4;

            const FRAC_PI_6: Self = std::$T::consts::FRAC_PI_6;

            const FRAC_PI_8: Self = std::$T::consts::FRAC_PI_8;

            const FRAC_1_PI: Self = std::$T::consts::FRAC_1_PI;

            const FRAC_1_SQRT_PI: Self = std::$T::consts::FRAC_1_SQRT_PI;

            const FRAC_1_SQRT_2PI: Self = std::$T::consts::FRAC_1_SQRT_2PI;

            const FRAC_2_PI: Self = std::$T::consts::FRAC_2_PI;

            const FRAC_2_SQRT_PI: Self = std::$T::consts::FRAC_2_SQRT_PI;

            const SQRT_2: Self = std::$T::consts::SQRT_2;

            const FRAC_1_SQRT_2: Self = std::$T::consts::FRAC_1_SQRT_2;

            const SQRT_3: Self = std::$T::consts::SQRT_3;

            const FRAC_1_SQRT_3: Self = std::$T::consts::FRAC_1_SQRT_3;

            const SQRT_5: Self = std::$T::consts::SQRT_5;

            const FRAC_1_SQRT_5: Self = std::$T::consts::FRAC_1_SQRT_5;

            const E: Self = std::$T::consts::E;

            const LOG2_E: Self = std::$T::consts::LOG2_E;

            const LOG2_10: Self = std::$T::consts::LOG2_10;

            const LOG10_E: Self = std::$T::consts::LOG10_E;

            const LOG10_2: Self = std::$T::consts::LOG10_2;

            const LN_2: Self = std::$T::consts::LN_2;

            const LN_10: Self = std::$T::consts::LN_10;

            #[feature(assoc_int_consts)]
            const EPSILON: Self = std::$T::EPSILON;

            #[feature(assoc_int_consts)]
            const MIN: Self = std::$T::MIN;
            #[feature(assoc_int_consts)]
            const MIN_POSITIVE: Self = std::$T::MIN_POSITIVE;
            #[feature(assoc_int_consts)]
            const MAX: Self = std::$T::MAX;

            #[feature(assoc_int_consts)]
            const NAN: Self = std::$T::NAN;

            #[feature(assoc_int_consts)]
            const INFINITY: Self = std::$T::INFINITY;

            #[feature(assoc_int_consts)]
            const NEG_INFINITY: Self = std::$T::NEG_INFINITY;

            #[inline]
            #[allow(deprecated)]
            fn abs_sub(self, other: Self) -> Self {
                <$T>::abs_sub(self, other)
            }

            #[inline]
            fn is_nan(self) -> bool {
                Self::is_nan(self)
            }
            #[inline]
            fn is_infinite(self) -> bool {
                Self::is_infinite(self)
            }
            #[inline]
            fn is_finite(self) -> bool {
                Self::is_finite(self)
            }
            #[inline]
            fn is_normal(self) -> bool {
                Self::is_normal(self)
            }
            #[inline]
            fn is_subnormal(self) -> bool {
                Self::is_subnormal(self)
            }
            #[inline]
            fn classify(self) -> FpCategory {
                Self::classify(self)
            }
            #[inline]
            fn floor(self) -> Self {
                Self::floor(self)
            }
            #[inline]
            fn ceil(self) -> Self {
                Self::ceil(self)
            }
            #[inline]
            fn round(self) -> Self {
                Self::round(self)
            }
            #[inline]
            fn trunc(self) -> Self {
                Self::trunc(self)
            }
            #[inline]
            fn fract(self) -> Self {
                Self::fract(self)
            }
            #[inline]
            fn abs(self) -> Self {
                Self::abs(self)
            }
            #[inline]
            fn signum(self) -> Self {
                Self::signum(self)
            }
            #[inline]
            fn is_sign_positive(self) -> bool {
                Self::is_sign_positive(self)
            }
            #[inline]
            fn is_sign_negative(self) -> bool {
                Self::is_sign_negative(self)
            }
            #[inline]
            fn mul_add(self, a: Self, b: Self) -> Self {
                Self::mul_add(self, a, b)
            }
            #[inline]
            fn recip(self) -> Self {
                Self::recip(self)
            }
            #[inline]
            fn powi(self, n: i32) -> Self {
                Self::powi(self, n)
            }
            #[inline]
            fn powf(self, n: Self) -> Self {
                Self::powf(self, n)
            }
            #[inline]
            fn sqrt(self) -> Self {
                Self::sqrt(self)
            }
            #[inline]
            fn exp(self) -> Self {
                Self::exp(self)
            }
            #[inline]
            fn exp2(self) -> Self {
                Self::exp2(self)
            }
            #[inline]
            fn ln(self) -> Self {
                Self::ln(self)
            }
            #[inline]
            fn log(self, base: Self) -> Self {
                Self::log(self, base)
            }
            #[inline]
            fn log2(self) -> Self {
                Self::log2(self)
            }
            #[inline]
            fn log10(self) -> Self {
                Self::log10(self)
            }
            #[inline]
            fn to_degrees(self) -> Self {
                Self::to_degrees(self)
            }
            #[inline]
            fn to_radians(self) -> Self {
                Self::to_radians(self)
            }
            #[inline]
            fn cbrt(self) -> Self {
                Self::cbrt(self)
            }
            #[inline]
            fn hypot(self, other: Self) -> Self {
                Self::hypot(self, other)
            }
            #[inline]
            fn sin(self) -> Self {
                Self::sin(self)
            }
            #[inline]
            fn cos(self) -> Self {
                Self::cos(self)
            }
            #[inline]
            fn tan(self) -> Self {
                Self::tan(self)
            }
            #[inline]
            fn asin(self) -> Self {
                Self::asin(self)
            }
            #[inline]
            fn acos(self) -> Self {
                Self::acos(self)
            }
            #[inline]
            fn atan(self) -> Self {
                Self::atan(self)
            }
            #[inline]
            fn atan2(self, other: Self) -> Self {
                Self::atan2(self, other)
            }
            #[inline]
            fn sin_cos(self) -> (Self, Self) {
                Self::sin_cos(self)
            }
            #[inline]
            fn exp_m1(self) -> Self {
                Self::exp_m1(self)
            }
            #[inline]
            fn ln_1p(self) -> Self {
                Self::ln_1p(self)
            }
            #[inline]
            fn sinh(self) -> Self {
                Self::sinh(self)
            }
            #[inline]
            fn cosh(self) -> Self {
                Self::cosh(self)
            }
            #[inline]
            fn tanh(self) -> Self {
                Self::tanh(self)
            }
            #[inline]
            fn asinh(self) -> Self {
                Self::asinh(self)
            }
            #[inline]
            fn acosh(self) -> Self {
                Self::acosh(self)
            }
            #[inline]
            fn atanh(self) -> Self {
                Self::atanh(self)
            }
            #[inline]
            fn copysign(self, sign: Self) -> Self {
                Self::copysign(self, sign)
            }
        }
    };
}

impl_float!(f32);
impl_float!(f64);

