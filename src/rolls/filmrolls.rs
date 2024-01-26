//! Deserialization for *Film Rolls* XML data
use std::borrow::Cow;

use chrono::{DateTime, FixedOffset};
use serde::Deserialize;

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
    pub camera: Vec<TextValue<'a>>,
}

/// Lens list element (`<lenses>`)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct Lenses<'a> {
    #[serde(default)]
    pub lens: Vec<TextValue<'a>>,
}

/// Accessory list element (`<accessories>`)
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct Accessories<'a> {
    #[serde(default)]
    pub accessory: Vec<TextValue<'a>>,
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
    pub title: TextValue<'a>,
    pub speed: Value<u32>,
    pub camera: TextValue<'a>,
    pub load: Value<DateTime<FixedOffset>>,
    pub unload: Value<DateTime<FixedOffset>>,
    pub note: TextValue<'a>,
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
    pub lens: TextValue<'a>,
    pub aperture: Value<rust_decimal::Decimal>,
    pub shutter_speed: Value<num_rational::Rational32>,
    pub compensation: Value<num_rational::Rational32>,
    pub accessory: TextValue<'a>,
    pub number: Value<usize>,
    pub date: Value<DateTime<FixedOffset>>,
    pub latitude: Value<f64>,
    pub longitude: Value<f64>,
    pub note: TextValue<'a>,
}

/// Generic value wrapper
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(Deserialize, Default)]
pub(super) struct Value<T>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    #[serde(default, rename = "$text", deserialize_with = "deserialize_from_str")]
    pub value: T,
}

/// String value wrapper
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(Deserialize, Default)]
pub(super) struct TextValue<'a> {
    #[serde(default, rename = "$text")]
    pub value: Cow<'a, str>,
}

/// Deserialize values using `FromStr::from_str`
fn deserialize_from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    String::deserialize(deserializer)
        .and_then(|s| T::from_str(&s).map_err(serde::de::Error::custom))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use num_rational::Rational32;
    use pretty_assertions::assert_eq;
    use quick_xml::de::{from_str, DeError};
    use rust_decimal::Decimal;
    type DateTime = chrono::DateTime<FixedOffset>;

    #[test]
    fn empty_values() -> Result<(), DeError> {
        let xml = "<empty></empty>";
        assert_eq!(from_str::<TextValue>(xml)?, Default::default());
        assert_eq!(from_str::<Value<DateTime>>(xml)?, Default::default());
        assert_eq!(from_str::<Value<f64>>(xml)?, Default::default());
        assert_eq!(from_str::<Value<Rational32>>(xml)?, Default::default());
        assert_eq!(from_str::<Value<Decimal>>(xml)?, Default::default());
        assert_eq!(from_str::<Value<u32>>(xml)?, Default::default());
        assert_eq!(from_str::<Value<usize>>(xml)?, Default::default());
        Ok(())
    }

    #[test]
    fn filled_values() -> Result<(), DeError> {
        assert_eq!(
            from_str::<TextValue>("<value>Plain text value</value>")?,
            TextValue {
                value: "Plain text value".into()
            },
        );
        assert_eq!(
            from_str::<Value<DateTime>>("<value>2016-03-28T15:16:36Z</value>")?,
            Value {
                value: chrono::Utc
                    .with_ymd_and_hms(2016, 3, 28, 15, 16, 36)
                    .unwrap()
                    .into()
            }
        );
        assert_eq!(
            from_str::<Value<f64>>("<value>57.700767</value>")?,
            Value { value: 57.700767 }
        );
        assert_eq!(
            from_str::<Value<Rational32>>("<value>1/500</value>")?,
            Value {
                value: Rational32::new(1, 500)
            }
        );
        assert_eq!(
            from_str::<Value<Decimal>>("<value>5.6</value>")?,
            Value {
                value: Decimal::new(56, 1)
            }
        );
        assert_eq!(
            from_str::<Value<u32>>("<value>100</value>")?,
            Value { value: 100 }
        );
        assert_eq!(
            from_str::<Value<usize>>("<value>1</value>")?,
            Value { value: 1 }
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
                        TextValue {
                            value: "Yashica Electro 35 GT".into()
                        },
                        TextValue {
                            value: "Voigtländer Bessa R2M".into()
                        },
                    ]
                },
                lenses: Lenses {
                    lens: vec![
                        TextValue {
                            value: "Yashinon 45mm f/1.7".into()
                        },
                        TextValue {
                            value: "Color Skopar 35/2.5 Pancake II".into()
                        },
                    ]
                },
                accessories: Accessories { accessory: vec![] },
                film_rolls: FilmRolls {
                    film_roll: vec![FilmRoll {
                        title: TextValue {
                            value: "Ilford Delta 100".into()
                        },
                        speed: Value { value: 100 },
                        camera: TextValue {
                            value: "Voigtländer Bessa R2M".into()
                        },
                        load: Value {
                            value: chrono::Utc
                                .with_ymd_and_hms(2016, 3, 28, 15, 16, 36)
                                .unwrap()
                                .into()
                        },
                        unload: Value {
                            value: chrono::Utc
                                .with_ymd_and_hms(2016, 5, 21, 14, 13, 15)
                                .unwrap()
                                .into()
                        },
                        note: TextValue {
                            value: "A0012".into()
                        },
                        frames: Frames {
                            frame: vec![Frame {
                                lens: TextValue {
                                    value: "Color Skopar 35/2.5 Pancake II".into()
                                },
                                aperture: Value {
                                    value: Decimal::new(56, 1)
                                },
                                shutter_speed: Value {
                                    value: Rational32::new(1, 500)
                                },
                                compensation: Value::default(),
                                accessory: TextValue::default(),
                                number: Value { value: 1 },
                                date: Value {
                                    value: chrono::Utc
                                        .with_ymd_and_hms(2016, 5, 13, 14, 12, 40)
                                        .unwrap()
                                        .into()
                                },
                                latitude: Value { value: 57.700767 },
                                longitude: Value { value: 11.953715 },
                                note: TextValue::default(),
                            }]
                        }
                    }]
                },
            }
        );
        Ok(())
    }
}
