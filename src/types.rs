//! Generic photography-related types
use std::num::{NonZeroU8, TryFromIntError};

use rust_decimal::{
    prelude::{FromPrimitive, Zero},
    Decimal, MathematicalOps,
};
use serde::{Deserialize, Deserializer};
use serde_with::{DeserializeAs, DeserializeFromStr};

/// A geographical position
#[derive(Copy, Clone, Default, PartialEq, PartialOrd, Debug)]
pub struct Position {
    pub lat: f64,
    pub lon: f64,
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let precision = f.precision().unwrap_or(3);
        let lat = dms_coordinates::DMS::from_ddeg_latitude(self.lat);
        let lon = dms_coordinates::DMS::from_ddeg_longitude(self.lon);
        write!(
            f,
            "{0}° {1}′ {2:.8$}″{3}, {4}° {5}′ {6:.8$}″{7}",
            lat.degrees,
            lat.minutes,
            lat.seconds,
            lat.cardinal.map(|c| format!(" {c}")).unwrap_or_default(),
            lon.degrees,
            lon.minutes,
            lon.seconds,
            lon.cardinal.map(|c| format!(" {c}")).unwrap_or_default(),
            precision,
        )
    }
}

/// A focal length (both real and 35mm equivalent)
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub struct FocalLength {
    pub real: rust_decimal::Decimal,
    pub equiv: Option<rust_decimal::Decimal>,
}

impl std::fmt::Display for FocalLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let real = self.real;
        write!(f, "{} mm", real.round_sf(2).unwrap_or(real).normalize())
    }
}

/// A shutter speed setting
///
/// As shutter speeds are commonly defined in terms of fractions,
/// this type represents them using the `Ratio` type from the
/// *num-rational* crate.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(DeserializeFromStr)]
pub enum ShutterSpeed {
    /// Shutter speed, in seconds (manual or known)
    Manual(num_rational::Rational32),

    /// Unknown shutter speed, aperture priority
    AperturePriority,
}

impl<'de> DeserializeAs<'de, ShutterSpeed> for f64 {
    fn deserialize_as<D>(deserializer: D) -> Result<ShutterSpeed, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        f64::deserialize(deserializer)
            .map(num_rational::Rational32::from_f64)
            .and_then(|v| v.ok_or(Error::custom("Rational32::from_f64 failed")))
            .map(ShutterSpeed::Manual)
    }
}

impl std::str::FromStr for ShutterSpeed {
    type Err = num_rational::ParseRatioError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Av" => Ok(Self::AperturePriority),
            value => num_rational::Rational32::from_str(value).map(Self::Manual),
        }
    }
}

impl From<num_rational::Rational32> for ShutterSpeed {
    fn from(value: num_rational::Rational32) -> Self {
        Self::Manual(value)
    }
}

impl std::fmt::Display for ShutterSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual(value) => write!(f, "{value} s"),
            Self::AperturePriority => write!(f, "Av"),
        }
    }
}

/// An exposure bias, in EV units
///
/// Exposure compensation is typically set in half- or third-step
/// increments, and therefore this type represents them using the
/// `Ratio` type from the *num-rational* crate.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(DeserializeFromStr)]
pub struct ExposureBias(pub num_rational::Rational32);

impl std::str::FromStr for ExposureBias {
    type Err = num_rational::ParseRatioError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        num_rational::Rational32::from_str(s).map(Self)
    }
}

impl From<num_rational::Rational32> for ExposureBias {
    fn from(value: num_rational::Rational32) -> Self {
        Self(value)
    }
}

impl Default for ExposureBias {
    fn default() -> Self {
        Self(num_rational::Rational32::zero())
    }
}

impl std::fmt::Display for ExposureBias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} EV", self.0)
    }
}

/// An aperture (f-stop) setting
///
/// Although apertures technically map to a series of (fractional)
/// powers of the square root of two, this is an impractical way to
/// represent them - there are exceptions (such as ƒ/0.95), and EXIF
/// represents them using floating-point numbers anyway. To avoid
/// discarding information, this type uses `Decimal` instead.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(DeserializeFromStr)]
pub enum Aperture {
    /// Aperture value, in f-stops (manual or known)
    Manual(rust_decimal::Decimal),

    /// Unknown aperture, shutter priority
    ShutterPriority,
}

impl<'de> DeserializeAs<'de, Aperture> for f64 {
    fn deserialize_as<D>(deserializer: D) -> Result<Aperture, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        f64::deserialize(deserializer)
            .map(rust_decimal::Decimal::try_from)
            .and_then(|v| v.map_err(D::Error::custom))
            .map(Aperture::Manual)
    }
}

impl std::str::FromStr for Aperture {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Tv" | "S" => Ok(Self::ShutterPriority),
            value => rust_decimal::Decimal::from_str(value).map(Self::Manual),
        }
    }
}

impl From<rust_decimal::Decimal> for Aperture {
    fn from(value: rust_decimal::Decimal) -> Self {
        Self::Manual(value)
    }
}

impl std::fmt::Display for Aperture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Manual(value) => {
                write!(f, "ƒ/{}", value.round_sf(2).unwrap_or(*value).normalize())
            }
            Self::ShutterPriority => write!(f, "Tv"),
        }
    }
}

/// An ISO film speed value
///
/// Film speeds are standardized, and this type uses the logarithmic
/// ISO speed for storage (sometimes also referred to as DIN film
/// speed). It provides accessors for arithmetic (ASA) film speed,
/// and an ISO alias which also returns the arithmetic speed.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FilmSpeed {
    din: u8,
}

impl FilmSpeed {
    /// Construct from a logarithmic DIN film speed
    pub fn from_din(value: u8) -> Self {
        Self { din: value }
    }

    /// Construct from an arithmetic ASA film speed
    pub fn from_asa(value: Decimal) -> Result<Self, TryFromIntError> {
        match value {
            v if v.is_zero() => NonZeroU8::try_from(0).map(Into::into),
            v => (Decimal::TEN * v.log10() + Decimal::ONE)
                .round()
                .normalize()
                .mantissa()
                .try_into(),
        }
        .map(|din: u8| Self::from_din(din))
    }

    /// Construct from an arithmetic ISO film speed
    pub fn from_iso(value: Decimal) -> Result<Self, TryFromIntError> {
        Self::from_asa(value)
    }

    /// The logarithmic (DIN) value of this film speed
    pub fn din(&self) -> u8 {
        self.din
    }

    /// The arithmetic (ASA) value of this film speed
    pub fn asa(&self) -> Decimal {
        let shift: u32 = match self.din.div_euclid(10) {
            v @ 0..=4 => (4 - v).into(),
            _ => unreachable!(),
        };
        let base: i64 = match self.din.rem_euclid(10) {
            0 => 8000,
            1 => 10000,
            2 => 12500,
            3 => 16000,
            4 => 20000,
            5 => 25000,
            6 => 32000,
            7 => 40000,
            8 => 50000,
            9 => 64000,
            _ => unreachable!(),
        };
        let width: u32 = match (base, shift) {
            (12500, 4) => 2,
            (12500, 3) => 2,
            (12500, _) => 3,
            (32000, 4) => 1,
            (64000, 4) => 1,
            (_, _) => 2,
        };
        match Decimal::new(base, shift).round_sf(width) {
            Some(v) => v.normalize(),
            None => unreachable!(),
        }
    }

    /// The arithmetic (ISO) value of this film speed
    pub fn iso(&self) -> Decimal {
        self.asa()
    }
}

impl std::fmt::Display for FilmSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}°", self.asa(), self.din())
    }
}

/// Helper trait converting Decimal to Rational
pub(crate) trait AsRational<T> {
    fn as_rational(&self) -> num_rational::Ratio<T>;
}

impl AsRational<i32> for rust_decimal::Decimal {
    fn as_rational(&self) -> num_rational::Ratio<i32> {
        let v = self.normalize();
        num_rational::Ratio::new(v.mantissa() as i32, 10i32.pow(v.scale()))
    }
}

impl AsRational<i64> for rust_decimal::Decimal {
    fn as_rational(&self) -> num_rational::Ratio<i64> {
        let v = self.normalize();
        num_rational::Ratio::new(v.mantissa() as i64, 10i64.pow(v.scale()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_rational::Ratio;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    const VALID_FILM_SPEEDS: [(Decimal, u8, &str); 45] = [
        (dec!(0.8), 0, "0.8/0°"),
        (dec!(1), 1, "1/1°"),
        (dec!(1.2), 2, "1.2/2°"),
        (dec!(1.6), 3, "1.6/3°"),
        (dec!(2), 4, "2/4°"),
        (dec!(2.5), 5, "2.5/5°"),
        (dec!(3), 6, "3/6°"),
        (dec!(4), 7, "4/7°"),
        (dec!(5), 8, "5/8°"),
        (dec!(6), 9, "6/9°"),
        (dec!(8), 10, "8/10°"),
        (dec!(10), 11, "10/11°"),
        (dec!(12), 12, "12/12°"),
        (dec!(16), 13, "16/13°"),
        (dec!(20), 14, "20/14°"),
        (dec!(25), 15, "25/15°"),
        (dec!(32), 16, "32/16°"),
        (dec!(40), 17, "40/17°"),
        (dec!(50), 18, "50/18°"),
        (dec!(64), 19, "64/19°"),
        (dec!(80), 20, "80/20°"),
        (dec!(100), 21, "100/21°"),
        (dec!(125), 22, "125/22°"),
        (dec!(160), 23, "160/23°"),
        (dec!(200), 24, "200/24°"),
        (dec!(250), 25, "250/25°"),
        (dec!(320), 26, "320/26°"),
        (dec!(400), 27, "400/27°"),
        (dec!(500), 28, "500/28°"),
        (dec!(640), 29, "640/29°"),
        (dec!(800), 30, "800/30°"),
        (dec!(1000), 31, "1000/31°"),
        (dec!(1250), 32, "1250/32°"),
        (dec!(1600), 33, "1600/33°"),
        (dec!(2000), 34, "2000/34°"),
        (dec!(2500), 35, "2500/35°"),
        (dec!(3200), 36, "3200/36°"),
        (dec!(4000), 37, "4000/37°"),
        (dec!(5000), 38, "5000/38°"),
        (dec!(6400), 39, "6400/39°"),
        (dec!(8000), 40, "8000/40°"),
        (dec!(10000), 41, "10000/41°"),
        (dec!(12500), 42, "12500/42°"),
        (dec!(16000), 43, "16000/43°"),
        (dec!(20000), 44, "20000/44°"),
    ];

    #[test]
    fn film_speed_from_din() {
        // Make sure all supported film speeds work fine
        for (asa, din, text) in VALID_FILM_SPEEDS {
            let film_speed = FilmSpeed::from_din(din);
            assert_eq!(film_speed.asa(), asa);
            assert_eq!(film_speed.din(), din);
            assert_eq!(film_speed.to_string(), text);
        }
    }

    #[test]
    fn film_speed_from_asa() {
        // Make sure all supported film speeds work fine
        for (asa, din, text) in VALID_FILM_SPEEDS {
            let film_speed = FilmSpeed::from_asa(asa)
                .expect("should be possible to construct `FilmSpeed` from all valid ASA values");
            assert_eq!(film_speed.asa(), asa);
            assert_eq!(film_speed.din(), din);
            assert_eq!(film_speed.to_string(), text);
        }

        // Make sure some obviously invalid ASA speeds return errors
        // These correspond to DIN -inf°, -1° and 256° respectively
        assert!(FilmSpeed::from_asa(dec!(0)).is_err());
        assert!(FilmSpeed::from_asa(dec!(0.6)).is_err());
        assert!(FilmSpeed::from_asa(dec!(31_622_776_601_683_793_319_988_936)).is_err())
    }

    #[test]
    fn print_position() {
        let position = Position {
            lat: 38.8897,
            lon: -77.0089,
        };
        assert_eq!(
            format!("{:.0}", position), //
            "38° 53′ 23″ N, 77° 0′ 32″ W"
        );
        assert_eq!(
            format!("{:.1}", position),
            "38° 53′ 22.9″ N, 77° 0′ 32.0″ W"
        );
        assert_eq!(
            format!("{:.2}", position),
            "38° 53′ 22.92″ N, 77° 0′ 32.04″ W"
        );
    }

    #[test]
    fn parse_shutter_speed() {
        assert_eq!("Av".parse(), Ok(ShutterSpeed::AperturePriority));
        assert_eq!("1/10".parse(), Ok(ShutterSpeed::Manual(Ratio::new(1, 10))));
    }

    #[test]
    fn parse_aperture() {
        assert_eq!("S".parse(), Ok(Aperture::ShutterPriority));
        assert_eq!("Tv".parse(), Ok(Aperture::ShutterPriority));
        assert_eq!("5.6".parse(), Ok(Aperture::Manual(dec!(5.6))));
    }

    #[test]
    fn rational_from_decimal() {
        assert_eq!(dec!(1230.0).as_rational(), Ratio::<i32>::new(1230, 1));
        assert_eq!(dec!(12.30).as_rational(), Ratio::<i32>::new(123, 10));
        assert_eq!(dec!(0.1230).as_rational(), Ratio::<i32>::new(123, 1000));
        assert_eq!(dec!(1230.0).as_rational(), Ratio::<i64>::new(1230, 1));
        assert_eq!(dec!(12.30).as_rational(), Ratio::<i64>::new(123, 10));
        assert_eq!(dec!(0.1230).as_rational(), Ratio::<i64>::new(123, 1000));
    }
}
