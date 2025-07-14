//! Deserialization for *lightme* JSON data
use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_with::{serde_as, DeserializeFromStr};

use crate::types::{Aperture, ShutterSpeed};

/// Data container alias
pub(super) type Data<'a> = Vec<Frame<'a>>;

/// Frame object
#[serde_as]
#[derive(Clone, PartialEq, PartialOrd, Debug)]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct Frame<'a> {
    pub date_time_original: CustomDateTime,
    pub description: Option<Text<'a>>,
    pub document_name: Option<Text<'a>>,
    #[serde_as(as = "Option<f64>")]
    pub exposure_time: Option<ShutterSpeed>,
    #[serde_as(as = "Option<f64>")]
    pub f_number: Option<Aperture>,
    pub focal_length: Option<f64>,
    #[serde(rename = "FocalLengthIn35mmFormat")]
    pub focal_length_equiv: Option<f64>,
    #[serde(rename = "GPSLatitude", deserialize_with = "deserialize_gps_coord")]
    pub gps_latitude: f64,
    #[serde(rename = "GPSLongitude", deserialize_with = "deserialize_gps_coord")]
    pub gps_longitude: f64,
    pub image_number: usize,
    #[serde(rename = "ISOSpeed")]
    pub iso_speed: u32,
    pub lens_make: Option<Text<'a>>,
    pub lens_model: Option<Text<'a>>,
    pub make: Option<Text<'a>>,
    pub model: Option<Text<'a>>,
    pub reel_name: Option<Text<'a>>,
    pub user_comment: Option<Notes>,
}

/// Custom notes object
#[derive(Clone, PartialEq, PartialOrd, Debug)]
#[derive(DeserializeFromStr)]
pub(super) struct Notes {
    pub load_date: CustomDateTime,
    pub unload_date: CustomDateTime,
}

impl std::str::FromStr for Notes {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use lazy_regex::regex_captures;
        let load = regex_captures!(r"^load_date:\n(.+)$"m, s).map(|(_, v)| v);
        let unload = regex_captures!(r"^unload_date:\n(.+)$"m, s).map(|(_, v)| v);
        Ok(Notes {
            load_date: CustomDateTime::from_str(load.unwrap_or_default())?,
            unload_date: CustomDateTime::from_str(unload.unwrap_or_default())?,
        })
    }
}

/// Copy-on-write text value from the JSON source
pub type Text<'a> = std::borrow::Cow<'a, str>;

/// Custom date/time type with bespoke parsing
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[derive(DeserializeFromStr)]
pub struct CustomDateTime(NaiveDateTime);

impl From<CustomDateTime> for NaiveDateTime {
    fn from(value: CustomDateTime) -> Self {
        value.0
    }
}

impl From<NaiveDateTime> for CustomDateTime {
    fn from(value: NaiveDateTime) -> Self {
        Self(value)
    }
}

impl std::str::FromStr for CustomDateTime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        chrono::NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S")
            .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%d %b %Y at %H:%M"))
            .map(Self)
    }
}

/// Convert textual GPS coords to decimal lat/long
fn deserialize_gps_coord<'de, D>(de: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use lazy_regex::regex_captures;
    use serde::de::Error;
    let string = String::deserialize(de)?;
    let (_, deg, min, sec, cardinal) = regex_captures!(
        r#"
            (?:(?P<deg>\d+)deg\s+)
            (?:(?P<min>\d+)'\s+)?
            (?:(?P<sec>\d+(?:\.\d*)?)"\s+)?
            (?P<cardinal>[NEWS])
        "#x,
        &string
    )
    .ok_or(Error::custom("could not parse DMS coordinates"))?;
    Ok(dms_coordinates::DMS::new(
        deg.parse().map_err(Error::custom)?,
        min.parse().unwrap_or_default(),
        sec.parse().unwrap_or_default(),
        match cardinal {
            "N" => Some(dms_coordinates::Cardinal::North),
            "E" => Some(dms_coordinates::Cardinal::East),
            "W" => Some(dms_coordinates::Cardinal::West),
            "S" => Some(dms_coordinates::Cardinal::South),
            _ => None,
        },
    )
    .to_ddeg_angle())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use num_rational::Rational32;
    use pretty_assertions::assert_eq;
    use rust_decimal::Decimal;
    use serde_json::{from_str, Error};

    #[test]
    fn parse_custom_dates() -> Result<(), chrono::ParseError> {
        use std::str::FromStr;
        assert_eq!(
            CustomDateTime::from_str("2022:04:30 18:29:15")?.0,
            NaiveDate::from_ymd_opt(2022, 4, 30)
                .and_then(|d| d.and_hms_opt(18, 29, 15))
                .unwrap()
        );
        assert_eq!(
            CustomDateTime::from_str("30 Apr 2022 at 17:57")?.0,
            NaiveDate::from_ymd_opt(2022, 4, 30)
                .and_then(|d| d.and_hms_opt(17, 57, 00))
                .unwrap()
        );
        Ok(())
    }

    #[test]
    fn empty_document() -> Result<(), Error> {
        assert_eq!(
            from_str::<Data>(
                r#"
            [
            ]
                "#
            )?,
            vec![]
        );
        Ok(())
    }

    #[test]
    fn full_document() -> Result<(), Error> {
        assert_eq!(
            from_str::<Data>(
                r#"
            [
              {
                "DateTimeOriginal" : "2022:04:30 18:29:15",
                "Description" : "Ilford SFX 200 (135)",
                "DocumentName" : "Ilford SFX 200",
                "ExposureTime" : 0.008,
                "FileSource" : 1,
                "FNumber" : 8,
                "FocalLength" : 35,
                "FocalLengthIn35mmFormat" : 35,
                "GPSLatitude" : "57deg 42' 3\" N",
                "GPSLatitudeRef" : "North",
                "GPSLongitude" : "11deg 58' 27\" E",
                "GPSLongitudeRef" : "East",
                "ImageNumber" : 1,
                "ImageUniqueId" : "A0020_1",
                "ISO" : 200,
                "ISOSpeed" : 200,
                "LensMake" : "Voigtländer",
                "LensModel" : "35mm f\/2,5 Color Skopar Pancake II (35mm)",
                "Make" : "Voigtländer",
                "Model" : "Bessa R2M (Voigtländer)",
                "Notes" : "",
                "ReelName" : "A0020",
                "SensitivityType" : 3,
                "Software" : "Lightme - Logbook 2.2.3",
                "SourceFile" : ".\/1.",
                "SpectralSensitivity" : "Ilford SFX 200",
                "UserComment" : "roll_notes:\n \ndev_notes:\n \nload_date:\n30 Apr 2022 at 17:57\nunload_date:\n1 May 2022 at 15:12"
              }
            ]
                "#
            )?,
            vec![Frame {
                date_time_original: NaiveDate::from_ymd_opt(2022, 4, 30)
                    .and_then(|d| d.and_hms_opt(18, 29, 15))
                    .unwrap()
                    .into(),
                description: Some("Ilford SFX 200 (135)".into()),
                document_name: Some("Ilford SFX 200".into()),
                exposure_time: Some(Rational32::new(1, 125).into()),
                f_number: Some(Decimal::new(8, 0).into()),
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
                user_comment: Some(Notes {
                    load_date: NaiveDate::from_ymd_opt(2022, 4, 30)
                        .and_then(|d| d.and_hms_opt(17, 57, 00))
                        .unwrap()
                        .into(),
                    unload_date: NaiveDate::from_ymd_opt(2022, 5, 1)
                        .and_then(|d| d.and_hms_opt(15, 12, 00))
                        .unwrap()
                        .into(),
                })
            }]
        );
        Ok(())
    }
}
