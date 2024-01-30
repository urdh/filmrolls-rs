//! Common film roll & frame definitions
//!
//! This module provides a common film roll definition which can be
//! deserialized from all supported input data formats, and converted
//! to EXIF data or displayed using the `Display` trait.
use std::str::FromStr;

use chrono::{DateTime, FixedOffset};
use itertools::Itertools;
use serde_with::DeserializeFromStr;

use crate::types::*;
mod filmrolls;

/// Data deserialization errors
#[derive(Clone, Debug)]
#[derive(thiserror::Error)]
pub enum SourceError {
    /// Invalid XML input
    #[error(transparent)]
    InvalidXml(#[from] quick_xml::de::DeError),

    /// Missing input data
    #[error("Missing data: {0}")]
    MissingData(&'static str),

    /// Invalid input data
    #[error("Invalid data: {0}")]
    InvalidData(&'static str),
}

impl PartialEq for SourceError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidXml(l), Self::InvalidXml(r)) => {
                std::mem::discriminant(l) == std::mem::discriminant(r)
            }
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

/// A film type, e.g. "Ilford Delta 100"
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Film(pub String);

impl From<&str> for Film {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl std::fmt::Display for Film {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A camera make/model, e.g. "Voigtländer Bessa R2M"
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(DeserializeFromStr)]
pub struct Camera {
    pub make: String,
    pub model: String,
}

impl TryFrom<&str> for Camera {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl FromStr for Camera {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().split_once(' ') {
            Some((make, model)) => Ok(Self {
                make: make.trim().into(),
                model: model.trim().into(),
            }),
            None => Ok(Self {
                make: Default::default(),
                model: s.into(),
            }),
        }
    }
}

impl std::fmt::Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.make, self.model)
    }
}

/// A lens make/model, e.g. "Voigtländer Color Skopar 35/2.5 Pancake II"
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Lens {
    pub make: String,
    pub model: String,
}

impl TryFrom<&str> for Lens {
    type Error = <Self as FromStr>::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl FromStr for Lens {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().split_once(' ') {
            Some((make, model)) => Ok(Self {
                make: make.trim().into(),
                model: model.trim().into(),
            }),
            None => Ok(Self {
                make: Default::default(),
                model: s.into(),
            }),
        }
    }
}

impl std::fmt::Display for Lens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.make, self.model)
    }
}

/// A single exposed frame
#[derive(Clone, PartialEq, Debug)]
pub struct Frame {
    pub lens: Option<Lens>,
    pub aperture: Aperture,
    pub shutter_speed: ShutterSpeed,
    pub compensation: Option<ExposureBias>,
    pub datetime: DateTime<FixedOffset>,
    pub position: Position,
    pub note: Option<String>,
}

impl TryFrom<filmrolls::Frame<'_>> for Frame {
    type Error = SourceError;

    fn try_from(value: filmrolls::Frame<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            lens: value
                .lens
                .as_deref()
                .map(TryInto::try_into)
                .transpose()
                .map_err(|_| SourceError::InvalidData("lens (`<lens>`)"))?,
            aperture: value
                .aperture
                .ok_or(SourceError::MissingData("aperture (`<aperture>`)"))?,
            shutter_speed: value
                .shutter_speed
                .ok_or(SourceError::MissingData("shutter speed (`<shutterSpeed>`)"))?,
            compensation: value.compensation,
            datetime: value.date.into(),
            position: Position {
                lat: value.latitude,
                lon: value.longitude,
            },
            note: value.note.map(Into::into),
        })
    }
}

/// A complete film roll
///
/// The film roll contains a `Vec<Option<Frame>>`, which includes all
/// frames of the roll with potential gaps, starting at frame 1. If the
/// input data is missing information for any frame between frame 1 and
/// the last known frame of the input, it will be replaced with `None`;
/// this allows users to index into the list without knowing whether
/// there are any missing frames.
#[derive(Clone, PartialEq, Debug)]
pub struct Roll {
    pub id: String,
    pub film: Option<Film>,
    pub speed: FilmSpeed,
    pub camera: Option<Camera>,
    pub load: DateTime<FixedOffset>,
    pub unload: DateTime<FixedOffset>,
    pub frames: Vec<Option<Frame>>,
}

impl TryFrom<filmrolls::FilmRoll<'_>> for Roll {
    type Error = SourceError;

    fn try_from(value: filmrolls::FilmRoll) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value
                .note
                .map(Into::into)
                .ok_or(SourceError::MissingData("roll ID (`<note>`)"))?,
            film: value
                .title
                .as_deref()
                .map(TryInto::try_into)
                .transpose()
                .map_err(|_| SourceError::InvalidData("film (`<title>`)"))?,
            speed: FilmSpeed::from_iso(value.speed.into())
                .map_err(|_| SourceError::InvalidData("film speed (`<speed>`)"))?,
            camera: value
                .camera
                .as_deref()
                .map(TryInto::try_into)
                .transpose()
                .map_err(|_| SourceError::InvalidData("camera (`<camera>`)"))?,
            load: value.load.into(),
            unload: value.unload.into(),
            frames: expand_indexed(
                value
                    .frames
                    .frame
                    .into_iter()
                    .map(|frame| -> (usize, Result<Frame, _>) { (frame.number, frame.try_into()) }),
            )
            .map(Option::transpose)
            .try_collect()?,
        })
    }
}

/// Read Film Rolls iOS app XML data
///
/// Attempts to read film roll data from the Film Rolls iOS app using the provided
/// [quick-xml](https://docs.rs/quick-xml/latest/quick_xml/) reader. If a parsing
/// error occurs, or any data is missing or invalid, the resulting iterator will
/// return exactly one `Err` element, otherwise an iterator of film, rolls is returned.
pub fn from_filmrolls<R>(reader: R) -> impl Iterator<Item = Result<Roll, SourceError>>
where
    R: std::io::BufRead,
{
    use itertools::Either::{Left, Right};
    match quick_xml::de::from_reader::<R, filmrolls::Data>(reader) {
        Ok(data) => Left(data.film_rolls.film_roll.into_iter().map(TryInto::try_into)),
        Err(error) => Right(std::iter::once(Err(error.into()))),
    }
}

/// Expand an `(index, item)` iterator into `Option<item>`
///
/// This function iterates over the given index/value pairs, inserting
/// `None` elements wherever there are gaps in the provided index. Note
/// that indexing is assumed to start at 1.
fn expand_indexed<I, T>(items: I) -> impl Iterator<Item = Option<T>>
where
    I: Iterator<Item = (usize, T)>,
{
    items
        .into_iter()
        .sorted_by_key(|(idx, _)| *idx)
        .scan(1, |counter, (index, frame)| {
            let fillers = index.saturating_sub(*counter);
            *counter = index + 1;
            Some(
                std::iter::repeat_with(|| None)
                    .take(fillers)
                    .chain(std::iter::once(Some(frame))),
            )
        })
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use itertools::assert_equal;
    use pretty_assertions::assert_eq;

    #[test]
    fn expand_indexed() {
        assert_equal(
            super::expand_indexed(std::iter::empty::<(usize, char)>()),
            std::iter::empty::<Option<char>>(),
        );
        assert_equal(
            super::expand_indexed(vec![(1, 'A'), (2, 'B')].into_iter()),
            vec![Some('A'), Some('B')].into_iter(),
        );
        assert_equal(
            super::expand_indexed(vec![(1, 'A'), (2, 'B'), (5, 'C')].into_iter()),
            vec![Some('A'), Some('B'), None, None, Some('C')].into_iter(),
        );
        assert_equal(
            super::expand_indexed(vec![(3, 'B')].into_iter()),
            vec![None, None, Some('B')].into_iter(),
        );
        assert_equal(
            super::expand_indexed(vec![(3, 'A'), (3, 'B')].into_iter()),
            vec![None, None, Some('A'), Some('B')].into_iter(),
        );
        assert_equal(
            super::expand_indexed(vec![(3, 'A'), (3, 'B'), (4, 'C')].into_iter()),
            vec![None, None, Some('A'), Some('B'), Some('C')].into_iter(),
        );
        assert_equal(
            super::expand_indexed(vec![(0, 'A')].into_iter()),
            vec![Some('A')].into_iter(),
        );
    }

    #[test]
    fn parse_film() {
        assert_eq!(
            Film::from("Ilford Delta 100"),
            Film("Ilford Delta 100".into())
        );
    }

    #[test]
    fn parse_camera() {
        assert_eq!(
            Camera::try_from("Voigtländer Bessa R2M"),
            Ok(Camera {
                make: "Voigtländer".into(),
                model: "Bessa R2M".into()
            })
        );
        assert_eq!(
            Camera::try_from("Voigtländer"),
            Ok(Camera {
                make: "".into(),
                model: "Voigtländer".into()
            })
        );
        assert_eq!(
            Camera::try_from(""),
            Ok(Camera {
                make: "".into(),
                model: "".into()
            })
        );
    }

    #[test]
    fn parse_lens() {
        assert_eq!(
            Lens::try_from("Voigtländer Color Skopar 35/2.5 Pancake II"),
            Ok(Lens {
                make: "Voigtländer".into(),
                model: "Color Skopar 35/2.5 Pancake II".into()
            })
        );
        assert_eq!(
            Lens::try_from("Voigtländer"),
            Ok(Lens {
                make: "".into(),
                model: "Voigtländer".into()
            })
        );
        assert_eq!(
            Lens::try_from(""),
            Ok(Lens {
                make: "".into(),
                model: "".into()
            })
        );
    }

    #[test]
    fn convert_filmrolls_frame() {
        let base_frame = filmrolls::Frame {
            lens: Some("Voigtländer Color Skopar 35/2.5 Pancake II".into()),
            aperture: Some(rust_decimal::Decimal::new(56, 1).into()),
            shutter_speed: Some(num_rational::Rational32::new(1, 500).into()),
            compensation: None,
            accessory: None,
            number: 1,
            date: chrono::Utc
                .with_ymd_and_hms(2016, 5, 13, 14, 12, 40)
                .unwrap()
                .into(),
            latitude: 57.700767,
            longitude: 11.953715,
            note: Some("Notes for this frame!".into()),
        };
        let expected = Frame {
            lens: Some(Lens {
                make: "Voigtländer".into(),
                model: "Color Skopar 35/2.5 Pancake II".into(),
            }),
            aperture: base_frame.aperture.map(Aperture::from).unwrap(),
            shutter_speed: base_frame.shutter_speed.map(ShutterSpeed::from).unwrap(),
            compensation: base_frame.compensation.map(ExposureBias::from),
            datetime: base_frame.date.clone().into(),
            position: Position {
                lat: base_frame.latitude,
                lon: base_frame.longitude,
            },
            note: base_frame.note.clone().map(Into::into),
        };

        assert_eq!(Frame::try_from(base_frame.clone()), Ok(expected.clone()));
        assert_eq!(
            Frame::try_from(filmrolls::Frame {
                lens: None,
                ..base_frame.clone()
            }),
            Ok(Frame {
                lens: None,
                ..expected.clone()
            })
        );
        assert_eq!(
            Frame::try_from(filmrolls::Frame {
                note: None,
                ..base_frame.clone()
            }),
            Ok(Frame {
                note: None,
                ..expected.clone()
            })
        );
    }

    #[test]
    fn convert_filmrolls_roll() {
        let base_roll = filmrolls::FilmRoll {
            title: Some("Ilford Delta 100".into()),
            speed: 100,
            camera: Some("Voigtländer Bessa R2M".into()),
            load: chrono::Utc
                .with_ymd_and_hms(2016, 3, 28, 15, 16, 36)
                .unwrap()
                .into(),
            unload: chrono::Utc
                .with_ymd_and_hms(2016, 5, 21, 14, 13, 15)
                .unwrap()
                .into(),
            note: Some("A0012".into()),
            frames: filmrolls::Frames { frame: vec![] },
        };
        let expected = Roll {
            id: base_roll.note.clone().unwrap().into(),
            film: Some(Film("Ilford Delta 100".into())),
            speed: FilmSpeed::from_din(21), // ISO 100/21°
            camera: Some(Camera {
                make: "Voigtländer".into(),
                model: "Bessa R2M".into(),
            }),
            load: base_roll.load.clone().into(),
            unload: base_roll.unload.clone().into(),
            frames: vec![],
        };

        assert_eq!(Roll::try_from(base_roll.clone()), Ok(expected.clone()));
        assert_eq!(
            Roll::try_from(filmrolls::FilmRoll {
                note: None,
                ..base_roll.clone()
            }),
            Err(SourceError::MissingData("..."))
        );
        assert_eq!(
            Roll::try_from(filmrolls::FilmRoll {
                speed: 0,
                ..base_roll.clone()
            }),
            Err(SourceError::InvalidData("..."))
        );
        assert_eq!(
            Roll::try_from(filmrolls::FilmRoll {
                title: None,
                ..base_roll.clone()
            }),
            Ok(Roll {
                film: None,
                ..expected.clone()
            })
        );
        assert_eq!(
            Roll::try_from(filmrolls::FilmRoll {
                camera: None,
                ..base_roll.clone()
            }),
            Ok(Roll {
                camera: None,
                ..expected.clone()
            })
        );
    }
}
