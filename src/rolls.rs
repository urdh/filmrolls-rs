//! Common film roll & frame definitions
//!
//! This module provides a common film roll definition which can be
//! deserialized from all supported input data formats, and converted
//! to EXIF data or displayed using the `Display` trait.
use std::str::FromStr;

use chrono::NaiveDateTime;
use itertools::Itertools;
use lazy_regex::regex_replace;
use serde_with::DeserializeFromStr;

use crate::types::*;
mod filmrolls;
mod lightme;

/// Data deserialization errors
#[derive(Debug)]
#[derive(thiserror::Error)]
pub enum SourceError {
    /// Invalid XML input
    #[error(transparent)]
    InvalidXml(#[from] quick_xml::de::DeError),

    /// Invalid JSON input
    #[error(transparent)]
    InvalidJson(#[from] serde_json::error::Error),

    /// Missing input data
    #[error("Missing data: {0}")]
    MissingData(&'static str),

    /// Invalid input data
    #[error("Invalid data: {0}")]
    InvalidData(&'static str),

    /// Unsupported file format
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
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
pub enum Camera {
    Simple { full_name: String },
    MakeModel { make: String, model: String },
}

impl Camera {
    pub fn from_make_model(make: Option<String>, model: String) -> Self {
        match make {
            Some(make) => Self::MakeModel { make, model },
            None => Self::Simple { full_name: model },
        }
    }

    pub fn make(&self) -> Option<&str> {
        match self {
            Self::MakeModel { make, .. } => Some(make),
            Self::Simple { .. } => None,
        }
    }

    pub fn model(&self) -> &str {
        match self {
            Self::MakeModel { model, .. } => model,
            Self::Simple { full_name } => full_name,
        }
    }
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
        Ok(Self::Simple {
            full_name: s.trim().into(),
        })
    }
}

impl std::fmt::Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple { full_name } => full_name.fmt(f),
            Self::MakeModel { make, model } => write!(f, "{make} {model}"),
        }
    }
}

/// A lens make/model, e.g. "Voigtländer Color Skopar 35/2.5 Pancake II"
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Lens {
    Simple { full_name: String },
    MakeModel { make: String, model: String },
}

impl Lens {
    pub fn from_make_model(make: Option<String>, model: String) -> Self {
        match make {
            Some(make) => Self::MakeModel { make, model },
            None => Self::Simple { full_name: model },
        }
    }

    pub fn make(&self) -> Option<&str> {
        match self {
            Self::MakeModel { make, .. } => Some(make),
            Self::Simple { .. } => None,
        }
    }

    pub fn model(&self) -> &str {
        match self {
            Self::MakeModel { model, .. } => model,
            Self::Simple { full_name } => full_name,
        }
    }
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
        Ok(Self::Simple {
            full_name: s.trim().into(),
        })
    }
}

impl std::fmt::Display for Lens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple { full_name } => full_name.fmt(f),
            Self::MakeModel { make, model } => write!(f, "{make} {model}"),
        }
    }
}

/// A single exposed frame
#[derive(Clone, PartialEq, Debug)]
pub struct Frame {
    pub lens: Option<Lens>,
    pub aperture: Option<Aperture>,
    pub shutter_speed: Option<ShutterSpeed>,
    pub focal_length: Option<FocalLength>,
    pub compensation: Option<ExposureBias>,
    pub datetime: NaiveDateTime,
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
            aperture: value.aperture,
            shutter_speed: value.shutter_speed,
            focal_length: None,
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

impl TryFrom<lightme::Frame<'_>> for Frame {
    type Error = SourceError;

    fn try_from(value: lightme::Frame<'_>) -> Result<Self, Self::Error> {
        Ok(Self {
            lens: value
                .lens_model
                .map(|v| regex_replace!(r"(\s+\(.*?\))$", v.as_ref(), "").into_owned())
                .map(|m| Lens::from_make_model(value.lens_make.map(Into::into), m)),
            aperture: value.f_number,
            shutter_speed: value.exposure_time,
            focal_length: (|| {
                Some(FocalLength {
                    real: value.focal_length.and_then(|v| v.try_into().ok())?,
                    equiv: value.focal_length_equiv.and_then(|v| v.try_into().ok()),
                })
            })(),
            compensation: None,
            datetime: value.date_time_original.into(),
            position: Position {
                lat: value.gps_latitude,
                lon: value.gps_longitude,
            },
            note: None,
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
    pub load: NaiveDateTime,
    pub unload: NaiveDateTime,
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

impl TryFrom<lightme::Data<'_>> for Roll {
    type Error = SourceError;

    fn try_from(value: lightme::Data) -> Result<Self, Self::Error> {
        let first = value
            .first()
            .ok_or(SourceError::MissingData("empty roll"))?
            .clone();
        let comment = first
            .user_comment
            .ok_or(SourceError::MissingData("load/unload date (`UserComment`)"))?;
        Ok(Self {
            id: first
                .reel_name
                .map(Into::into)
                .ok_or(SourceError::MissingData("roll ID (`ReelName`)"))?,
            film: first
                .document_name
                .as_deref()
                .map(TryInto::try_into)
                .transpose()
                .map_err(|_| SourceError::InvalidData("film (`DocumentName`)"))?,
            speed: FilmSpeed::from_iso(first.iso_speed.into())
                .map_err(|_| SourceError::InvalidData("film speed (`ISOSpeed`)"))?,
            camera: first
                .model
                .map(|v| regex_replace!(r"(\s+\(.*?\))$", v.as_ref(), "").into_owned())
                .map(|m| Camera::from_make_model(first.make.map(Into::into), m)),
            load: comment.load_date.into(),
            unload: comment.unload_date.into(),
            frames: expand_indexed(value.into_iter().map(|frame| -> (usize, Result<Frame, _>) {
                (frame.image_number, frame.try_into())
            }))
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
/// return exactly one `Err` element, otherwise an iterator of film rolls is returned.
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

/// Read lightme iOS app JSON data
///
/// Attempts to read film roll data from the lightme iOS app using the provided
/// [serde_json](https://docs.rs/serde_json/latest/serde_json/) reader. If a parsing
/// error occurs, or any data is missing or invalid, the resulting iterator will
/// return exactly one `Err` element, otherwise an iterator of film rolls is returned.
pub fn from_lightme<R>(reader: R) -> impl Iterator<Item = Result<Roll, SourceError>>
where
    R: std::io::BufRead,
{
    use itertools::Either::{Left, Right};
    match serde_json::de::from_reader::<R, lightme::Data>(reader) {
        Ok(data) => Left(
            data.into_iter()
                .into_group_map_by(|r| r.reel_name.to_owned())
                .into_values()
                .map(TryInto::try_into),
        ),
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
    use chrono::NaiveDate;
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
            vec![Some('A'), Some('B')],
        );
        assert_equal(
            super::expand_indexed(vec![(1, 'A'), (2, 'B'), (5, 'C')].into_iter()),
            vec![Some('A'), Some('B'), None, None, Some('C')],
        );
        assert_equal(
            super::expand_indexed(vec![(3, 'B')].into_iter()),
            vec![None, None, Some('B')],
        );
        assert_equal(
            super::expand_indexed(vec![(3, 'A'), (3, 'B')].into_iter()),
            vec![None, None, Some('A'), Some('B')],
        );
        assert_equal(
            super::expand_indexed(vec![(3, 'A'), (3, 'B'), (4, 'C')].into_iter()),
            vec![None, None, Some('A'), Some('B'), Some('C')],
        );
        assert_equal(
            super::expand_indexed(vec![(0, 'A')].into_iter()),
            vec![Some('A')],
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
            Ok(Camera::Simple {
                full_name: "Voigtländer Bessa R2M".into()
            })
        );
    }

    #[test]
    fn camera_from_make_model() {
        let make_model = Camera::from_make_model(Some("Voigtländer".into()), "Bessa R2M".into());
        let only_model = Camera::from_make_model(None, "Voigtländer Bessa R2M".into());
        assert_eq!(make_model.to_string(), "Voigtländer Bessa R2M");
        assert_eq!(only_model.to_string(), "Voigtländer Bessa R2M");
        assert_eq!(make_model.make(), Some("Voigtländer"));
        assert_eq!(only_model.make(), None);
        assert_eq!(make_model.model(), "Bessa R2M");
        assert_eq!(only_model.model(), "Voigtländer Bessa R2M");
    }

    #[test]
    fn parse_lens() {
        assert_eq!(
            Lens::try_from("Voigtländer Color Skopar 35/2.5 Pancake II"),
            Ok(Lens::Simple {
                full_name: "Voigtländer Color Skopar 35/2.5 Pancake II".into()
            })
        );
    }

    #[test]
    fn lens_from_make_model() {
        let make_model = Lens::from_make_model(
            Some("Voigtländer".into()),
            "Color Skopar 35/2.5 Pancake II".into(),
        );
        let only_model =
            Lens::from_make_model(None, "Voigtländer Color Skopar 35/2.5 Pancake II".into());
        assert_eq!(
            make_model.to_string(),
            "Voigtländer Color Skopar 35/2.5 Pancake II"
        );
        assert_eq!(
            only_model.to_string(),
            "Voigtländer Color Skopar 35/2.5 Pancake II"
        );
        assert_eq!(make_model.make(), Some("Voigtländer"));
        assert_eq!(only_model.make(), None);
        assert_eq!(make_model.model(), "Color Skopar 35/2.5 Pancake II");
        assert_eq!(
            only_model.model(),
            "Voigtländer Color Skopar 35/2.5 Pancake II"
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
            date: NaiveDate::from_ymd_opt(2016, 5, 13)
                .and_then(|d| d.and_hms_opt(14, 12, 40))
                .unwrap()
                .into(),
            latitude: 57.700767,
            longitude: 11.953715,
            note: Some("Notes for this frame!".into()),
        };
        let expected = Frame {
            lens: Some(Lens::Simple {
                full_name: "Voigtländer Color Skopar 35/2.5 Pancake II".into(),
            }),
            aperture: base_frame.aperture,
            shutter_speed: base_frame.shutter_speed,
            focal_length: None,
            compensation: base_frame.compensation,
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
            load: NaiveDate::from_ymd_opt(2016, 3, 28)
                .and_then(|d| d.and_hms_opt(15, 16, 36))
                .unwrap()
                .into(),
            unload: NaiveDate::from_ymd_opt(2016, 5, 21)
                .and_then(|d| d.and_hms_opt(14, 13, 15))
                .unwrap()
                .into(),
            note: Some("A0012".into()),
            frames: filmrolls::Frames { frame: vec![] },
        };
        let expected = Roll {
            id: base_roll.note.clone().unwrap().into(),
            film: Some(Film("Ilford Delta 100".into())),
            speed: FilmSpeed::from_din(21), // ISO 100/21°
            camera: Some(Camera::Simple {
                full_name: "Voigtländer Bessa R2M".into(),
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

    #[test]
    fn convert_lightme_frame() {
        let base_frame = lightme::Frame {
            date_time_original: NaiveDate::from_ymd_opt(2022, 4, 30)
                .and_then(|d| d.and_hms_opt(18, 29, 15))
                .unwrap()
                .into(),
            description: Some("Ilford SFX 200 (135)".into()),
            document_name: Some("Ilford SFX 200".into()),
            exposure_time: Some(num_rational::Rational32::new(1, 125).into()),
            f_number: Some(rust_decimal::Decimal::new(8, 0).into()),
            focal_length: Some(35.),
            focal_length_equiv: Some(35.),
            gps_latitude: 57.700833333333335,
            gps_longitude: 11.974166666666667,
            image_number: 1,
            iso_speed: 200,
            lens_make: Some("Voigtländer".into()),
            lens_model: Some("35mm f/2,5 Color Skopar Pancake II (35mm)".into()),
            make: Some("Voigtländer".into()),
            model: Some("Bessa R2M (Voigtländer)".into()),
            reel_name: Some("A0020".into()),
            user_comment: Some(lightme::Notes {
                load_date: NaiveDate::from_ymd_opt(2022, 4, 30)
                    .and_then(|d| d.and_hms_opt(17, 57, 00))
                    .unwrap()
                    .into(),
                unload_date: NaiveDate::from_ymd_opt(2022, 5, 1)
                    .and_then(|d| d.and_hms_opt(15, 12, 00))
                    .unwrap()
                    .into(),
            }),
        };
        let expected = Frame {
            lens: Some(Lens::MakeModel {
                make: "Voigtländer".into(),
                model: "35mm f/2,5 Color Skopar Pancake II".into(),
            }),
            aperture: base_frame.f_number,
            shutter_speed: base_frame.exposure_time,
            focal_length: Some(FocalLength {
                real: rust_decimal::Decimal::new(35, 0),
                equiv: Some(rust_decimal::Decimal::new(35, 0)),
            }),
            compensation: None,
            datetime: base_frame.date_time_original.clone().into(),
            position: Position {
                lat: base_frame.gps_latitude,
                lon: base_frame.gps_longitude,
            },
            note: None,
        };

        assert_eq!(Frame::try_from(base_frame.clone()), Ok(expected.clone()));
        assert_eq!(
            Frame::try_from(lightme::Frame {
                lens_make: None,
                ..base_frame.clone()
            }),
            Ok(Frame {
                lens: Some(Lens::Simple {
                    full_name: "35mm f/2,5 Color Skopar Pancake II".into()
                }),
                ..expected.clone()
            })
        );
        assert_eq!(
            Frame::try_from(lightme::Frame {
                lens_model: None,
                ..base_frame.clone()
            }),
            Ok(Frame {
                lens: None,
                ..expected.clone()
            })
        );
        assert_eq!(
            Frame::try_from(lightme::Frame {
                focal_length_equiv: None,
                ..base_frame.clone()
            }),
            Ok(Frame {
                focal_length: Some(FocalLength {
                    real: rust_decimal::Decimal::new(35, 0),
                    equiv: None,
                }),
                ..expected.clone()
            })
        );
        assert_eq!(
            Frame::try_from(lightme::Frame {
                focal_length: None,
                ..base_frame.clone()
            }),
            Ok(Frame {
                focal_length: None,
                ..expected.clone()
            })
        );
    }

    #[test]
    fn convert_lightme_roll() {
        let base_frame = lightme::Frame {
            date_time_original: NaiveDate::from_ymd_opt(2022, 4, 30)
                .and_then(|d| d.and_hms_opt(18, 29, 15))
                .unwrap()
                .into(),
            description: Some("Ilford SFX 200 (135)".into()),
            document_name: Some("Ilford SFX 200".into()),
            exposure_time: Some(num_rational::Rational32::new(1, 125).into()),
            f_number: Some(rust_decimal::Decimal::new(8, 0).into()),
            focal_length: Some(35.),
            focal_length_equiv: Some(35.),
            gps_latitude: 57.700833333333335,
            gps_longitude: 11.974166666666667,
            image_number: 1,
            iso_speed: 200,
            lens_make: Some("Voigtländer".into()),
            lens_model: Some("35mm f/2,5 Color Skopar Pancake II (35mm)".into()),
            make: Some("Voigtländer".into()),
            model: Some("Bessa R2M (Voigtländer)".into()),
            reel_name: Some("A0020".into()),
            user_comment: Some(lightme::Notes {
                load_date: NaiveDate::from_ymd_opt(2022, 4, 30)
                    .and_then(|d| d.and_hms_opt(17, 57, 00))
                    .unwrap()
                    .into(),
                unload_date: NaiveDate::from_ymd_opt(2022, 5, 1)
                    .and_then(|d| d.and_hms_opt(15, 12, 00))
                    .unwrap()
                    .into(),
            }),
        };
        let expected = Roll {
            id: base_frame.reel_name.clone().unwrap().into(),
            film: Some(Film("Ilford SFX 200".into())),
            speed: FilmSpeed::from_din(24), // ISO 200/24°
            camera: Some(Camera::MakeModel {
                make: "Voigtländer".into(),
                model: "Bessa R2M".into(),
            }),
            load: base_frame.user_comment.clone().unwrap().load_date.into(),
            unload: base_frame.user_comment.clone().unwrap().unload_date.into(),
            frames: vec![Some(Frame {
                lens: Some(Lens::MakeModel {
                    make: "Voigtländer".into(),
                    model: "35mm f/2,5 Color Skopar Pancake II".into(),
                }),
                aperture: base_frame.f_number,
                shutter_speed: base_frame.exposure_time,
                focal_length: Some(FocalLength {
                    real: rust_decimal::Decimal::new(35, 0),
                    equiv: Some(rust_decimal::Decimal::new(35, 0)),
                }),
                compensation: None,
                datetime: base_frame.date_time_original.clone().into(),
                position: Position {
                    lat: base_frame.gps_latitude,
                    lon: base_frame.gps_longitude,
                },
                note: None,
            })],
        };

        assert_eq!(
            Roll::try_from(vec![base_frame.clone()]),
            Ok(expected.clone())
        );
        assert_eq!(
            Roll::try_from(vec![lightme::Frame {
                reel_name: None,
                ..base_frame.clone()
            }]),
            Err(SourceError::MissingData("..."))
        );
        assert_eq!(
            Roll::try_from(vec![lightme::Frame {
                iso_speed: 0,
                ..base_frame.clone()
            }]),
            Err(SourceError::InvalidData("..."))
        );
        assert_eq!(
            Roll::try_from(vec![lightme::Frame {
                document_name: None,
                ..base_frame.clone()
            }]),
            Ok(Roll {
                film: None,
                ..expected.clone()
            })
        );
        assert_eq!(
            Roll::try_from(vec![lightme::Frame {
                make: None,
                ..base_frame.clone()
            }]),
            Ok(Roll {
                camera: Some(Camera::Simple {
                    full_name: "Bessa R2M".into()
                }),
                ..expected.clone()
            })
        );
        assert_eq!(
            Roll::try_from(vec![lightme::Frame {
                model: None,
                ..base_frame.clone()
            }]),
            Ok(Roll {
                camera: None,
                ..expected.clone()
            })
        );
    }
}
