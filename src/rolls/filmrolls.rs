//! Deserialization for *Film Rolls* XML data
use chrono::NaiveDateTime;
use quick_xml::serde_helpers::text_content;
use serde::Deserialize;
use serde_with::DeserializeFromStr;

use crate::types::{Aperture, ExposureBias, ShutterSpeed};

/// Outer `<data>` element
#[derive(Clone, PartialEq, PartialOrd, Debug)]
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct Data<'a> {
    #[serde(default)]
    pub cameras: Cameras<'a>,
    #[serde(default)]
    pub lenses: Lenses<'a>,
    #[serde(default)]
    pub accessories: Accessories<'a>,
    #[serde(default)]
    pub film_rolls: FilmRolls<'a>,
}

/// Camera list element (`<cameras>`)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct Cameras<'a> {
    #[serde(default)]
    pub camera: Vec<Camera<'a>>,
}

/// Camera container (`<camera>`)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(Deserialize, Default)]
pub(super) struct Camera<'a> {
    #[serde(rename = "$text")]
    pub value: Text<'a>,
}

/// Lens list element (`<lenses>`)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct Lenses<'a> {
    #[serde(default)]
    pub lens: Vec<Lens<'a>>,
}

/// Lens container (`<lens>`)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(Deserialize, Default)]
pub(super) struct Lens<'a> {
    #[serde(rename = "$text")]
    pub value: Text<'a>,
}

/// Accessory list element (`<accessories>`)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct Accessories<'a> {
    #[serde(default)]
    pub accessory: Vec<Accessory<'a>>,
}

/// Accessory container (`<accessory>`)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(Deserialize, Default)]
pub(super) struct Accessory<'a> {
    #[serde(rename = "$text")]
    pub value: Text<'a>,
}

/// Film roll list element (`<filmRolls>`)
#[derive(Clone, PartialEq, PartialOrd, Debug)]
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct FilmRolls<'a> {
    #[serde(default)]
    pub film_roll: Vec<FilmRoll<'a>>,
}

/// Film roll container (`<filmRoll>`)
#[derive(Clone, PartialEq, PartialOrd, Debug)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FilmRoll<'a> {
    #[serde(with = "text_content")]
    pub title: Option<Text<'a>>,
    #[serde(with = "text_content")]
    pub speed: u32,
    #[serde(with = "text_content")]
    pub camera: Option<Text<'a>>,
    #[serde(with = "text_content")]
    pub load: XmlDateTime,
    #[serde(with = "text_content")]
    pub unload: XmlDateTime,
    #[serde(with = "text_content")]
    pub note: Option<Text<'a>>,
    pub frames: Frames<'a>,
}

/// Frame list element (`<frames>`)
#[derive(Clone, PartialEq, PartialOrd, Debug)]
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct Frames<'a> {
    #[serde(default)]
    pub frame: Vec<Frame<'a>>,
}

/// Frame container (`<frame>`)
#[derive(Clone, PartialEq, PartialOrd, Debug)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Frame<'a> {
    #[serde(with = "text_content")]
    pub lens: Option<Text<'a>>,
    #[serde(with = "text_content")]
    pub aperture: Option<Aperture>,
    #[serde(with = "text_content")]
    pub shutter_speed: Option<ShutterSpeed>,
    #[serde(with = "text_content")]
    pub compensation: Option<ExposureBias>,
    #[serde(with = "text_content")]
    pub accessory: Option<Text<'a>>,
    #[serde(with = "text_content")]
    pub number: usize,
    #[serde(with = "text_content")]
    pub date: XmlDateTime,
    #[serde(with = "text_content")]
    pub latitude: f64,
    #[serde(with = "text_content")]
    pub longitude: f64,
    #[serde(with = "text_content")]
    pub note: Option<Text<'a>>,
}

/// Copy-on-write text value from the XML source
pub(super) type Text<'a> = std::borrow::Cow<'a, str>;

/// Sloppy RFC3339 date/time type with lax parsing
///
/// In addition to plain RFC3339, this type supports RFC3339-like date/time
/// values without timezone but *with* fractional seconds, as well as supporting
/// plain ISO8601 dates without an associated time (falling back to midnight).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[derive(DeserializeFromStr)]
pub(super) struct XmlDateTime(NaiveDateTime);

impl From<XmlDateTime> for NaiveDateTime {
    fn from(value: XmlDateTime) -> Self {
        value.0
    }
}

impl From<NaiveDateTime> for XmlDateTime {
    fn from(value: NaiveDateTime) -> Self {
        Self(value)
    }
}

impl std::str::FromStr for XmlDateTime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        chrono::DateTime::<chrono::FixedOffset>::parse_from_rfc3339(s)
            .map(|d| d.naive_local())
            .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f"))
            .or_else(|_| {
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .map(|date| date.and_time(chrono::NaiveTime::default()))
            })
            .map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use num_rational::Rational32;
    use pretty_assertions::assert_eq;
    use quick_xml::de::{from_str, DeError};
    use rust_decimal::Decimal;

    #[test]
    fn parse_sloppy_rfc3339() -> Result<(), chrono::ParseError> {
        use std::str::FromStr;
        assert_eq!(
            XmlDateTime::from_str("2016-03-28T15:16:36+05:00")?.0,
            NaiveDate::from_ymd_opt(2016, 3, 28)
                .and_then(|d| d.and_hms_opt(15, 16, 36))
                .unwrap()
        );
        assert_eq!(
            XmlDateTime::from_str("2016-03-28T15:16:36Z")?.0,
            NaiveDate::from_ymd_opt(2016, 3, 28)
                .and_then(|d| d.and_hms_opt(15, 16, 36))
                .unwrap()
        );
        assert_eq!(
            XmlDateTime::from_str("2019-07-17T15:47:53.208630")?.0,
            NaiveDate::from_ymd_opt(2019, 7, 17)
                .and_then(|d| d.and_hms_opt(15, 47, 53))
                .map(|date| date + chrono::Duration::microseconds(208630))
                .unwrap()
        );
        assert_eq!(
            XmlDateTime::from_str("2019-07-17T15:47:53")?.0,
            NaiveDate::from_ymd_opt(2019, 7, 17)
                .and_then(|d| d.and_hms_opt(15, 47, 53))
                .unwrap()
        );
        assert_eq!(
            XmlDateTime::from_str("2019-07-17")?.0,
            NaiveDate::from_ymd_opt(2019, 7, 17)
                .map(|d| d.and_time(chrono::NaiveTime::default()))
                .unwrap()
        );
        Ok(())
    }

    #[test]
    fn empty_document() -> Result<(), DeError> {
        assert_eq!(
            from_str::<Data>(
                r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <data xmlns="http://www.w3schools.com"
                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                xsi:schemaLocation="http://www.w3schools.com export.xsd">
            </data>
                "#
            )?,
            Default::default()
        );
        Ok(())
    }

    #[test]
    fn full_document() -> Result<(), DeError> {
        assert_eq!(
            from_str::<Data>(
                r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <data xmlns="http://www.w3schools.com"
                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                xsi:schemaLocation="http://www.w3schools.com export.xsd">
              <cameras>
                <camera>Yashica Electro 35 GT</camera>
                <camera>Voigtländer Bessa R2M</camera>
              </cameras>
              <lenses>
                <lens>Yashinon 45mm f/1.7</lens>
                <lens>Color Skopar 35/2.5 Pancake II</lens>
              </lenses>
              <accessories>
              </accessories>
              <filmRolls>
                <filmRoll>
                  <title>Ilford Delta 100</title>
                  <speed>100</speed>
                  <camera>Voigtländer Bessa R2M</camera>
                  <load>2016-03-28T15:16:36Z</load>
                  <unload>2016-05-21T14:13:15Z</unload>
                  <note>A0012</note>
                  <frames>
                    <frame>
                      <lens>Color Skopar 35/2.5 Pancake II</lens>
                      <aperture>5.6</aperture>
                      <shutterSpeed>1/500</shutterSpeed>
                      <compensation></compensation>
                      <accessory></accessory>
                      <number>1</number>
                      <date>2016-05-13T14:12:40Z</date>
                      <latitude>57.700767</latitude>
                      <longitude>11.953715</longitude>
                      <note></note>
                    </frame>
                  </frames>
                </filmRoll>
              </filmRolls>
            </data>
                "#
            )?,
            Data {
                cameras: Cameras {
                    camera: vec![
                        Camera {
                            value: "Yashica Electro 35 GT".into()
                        },
                        Camera {
                            value: "Voigtländer Bessa R2M".into()
                        },
                    ]
                },
                lenses: Lenses {
                    lens: vec![
                        Lens {
                            value: "Yashinon 45mm f/1.7".into()
                        },
                        Lens {
                            value: "Color Skopar 35/2.5 Pancake II".into()
                        },
                    ]
                },
                accessories: Accessories { accessory: vec![] },
                film_rolls: FilmRolls {
                    film_roll: vec![FilmRoll {
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
                        frames: Frames {
                            frame: vec![Frame {
                                lens: Some("Color Skopar 35/2.5 Pancake II".into()),
                                aperture: Some(Decimal::new(56, 1).into()),
                                shutter_speed: Some(Rational32::new(1, 500).into()),
                                compensation: None,
                                accessory: None,
                                number: 1,
                                date: NaiveDate::from_ymd_opt(2016, 5, 13)
                                    .and_then(|d| d.and_hms_opt(14, 12, 40))
                                    .unwrap()
                                    .into(),
                                latitude: 57.700767,
                                longitude: 11.953715,
                                note: None,
                            }]
                        }
                    }]
                },
            }
        );
        Ok(())
    }
}
