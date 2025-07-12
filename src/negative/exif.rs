//! Wrapper around EXIF metadata for a single image
use little_exif::exif_tag::ExifTag;
use little_exif::ifd::ExifTagGroup;
use little_exif::rational::{iR64, uR64};

use crate::metadata::Metadata;
use crate::rolls::{Frame, Roll};
use crate::types::*;

impl super::ApplyMetadata for little_exif::metadata::Metadata {
    fn apply_roll_data(&mut self, data: &Roll) -> Result<(), super::NegativeError> {
        // Set camera make & model, if available
        if let Some(camera) = &data.camera {
            self.set_tag(ExifTag::UnknownSTRING(
                camera.to_string(),
                0xc615,
                ExifTagGroup::GENERIC,
            ));
            if !camera.make.is_empty() {
                self.set_tag(ExifTag::Make(camera.make.clone()));
            }
            if !camera.model.is_empty() {
                self.set_tag(ExifTag::Model(camera.model.clone()));
            }
        }

        // Set film name in user comment, if available
        if let Some(film) = &data.film {
            self.set_tag(ExifTag::UserComment(to_exif_undef(
                &film.to_string(),
                self.get_endian(),
            )));
        }

        // Set film ISO speed
        let iso: i64 = data.speed.iso().as_rational().to_integer();
        self.set_tag(ExifTag::ISO(vec![
            iso.clamp(u16::MIN as i64, u16::MAX as i64) as u16,
        ]));
        self.set_tag(ExifTag::ISOSpeed(vec![
            iso.clamp(u32::MIN as i64, u32::MAX as i64) as u32,
        ]));
        self.set_tag(ExifTag::SensitivityType(vec![3u16])); // "ISO Speed"

        // Success!
        Ok(())
    }

    fn apply_frame_data(&mut self, data: &Frame) -> Result<(), super::NegativeError> {
        // Set original date/time
        self.set_tag(ExifTag::DateTimeOriginal(
            data.datetime.format("%Y:%m:%d %H:%M:%S").to_string(),
        ));

        // Set lens make & model, if available
        if let Some(lens) = &data.lens {
            self.set_tag(ExifTag::UnknownSTRING(
                lens.to_string(),
                0xfdea,
                ExifTagGroup::EXIF,
            ));
            if !lens.make.is_empty() {
                self.set_tag(ExifTag::LensMake(lens.make.clone()));
            }
            if !lens.model.is_empty() {
                self.set_tag(ExifTag::LensModel(lens.model.clone()));
            }
        }

        // Set focal length and optionally 35mm equivalent focal length
        if let Some(focal_length) = data.focal_length {
            let ratio: num_rational::Ratio<i64> = focal_length.real.as_rational();
            self.set_tag(ExifTag::FocalLength(vec![uR64::from_rational(ratio)]));
            if let Some(equiv) = focal_length.equiv {
                let equiv = equiv
                    .round()
                    .normalize()
                    .mantissa()
                    .clamp(u16::MIN as i128, u16::MAX as i128) as u16;
                self.set_tag(ExifTag::FocalLengthIn35mmFormat(vec![equiv]));
            }
        }

        // Set shutter speed, aperture, and exposure program
        if let Some(ShutterSpeed::Manual(value)) = data.shutter_speed {
            self.set_tag(ExifTag::ExposureTime(vec![
                uR64::from_rational(value), //
            ]));
            self.set_tag(ExifTag::ShutterSpeedValue(vec![
                iR64::from_rational(log2(value.recip())), // APEX value
            ]));
        }

        if let Some(Aperture::Manual(value)) = data.aperture {
            let ratio: num_rational::Ratio<i64> = value.as_rational();
            self.set_tag(ExifTag::FNumber(vec![
                uR64::from_rational(ratio), //
            ]));
            self.set_tag(ExifTag::ApertureValue(vec![
                uR64::from_rational(log2(ratio.pow(2))), // APEX value
            ]));
        }

        match (data.shutter_speed, data.aperture) {
            (Some(ShutterSpeed::AperturePriority), Some(Aperture::ShutterPriority)) => {
                self.set_tag(ExifTag::ExposureProgram(vec![2u16])) // "Program AE"
            }
            (Some(ShutterSpeed::AperturePriority), Some(Aperture::Manual(_))) => {
                self.set_tag(ExifTag::ExposureProgram(vec![3u16])) // "Aperture Priority AE"
            }
            (Some(ShutterSpeed::Manual(_)), Some(Aperture::ShutterPriority)) => {
                self.set_tag(ExifTag::ExposureProgram(vec![4u16])) // "Shutter Priority AE"
            }
            (Some(ShutterSpeed::Manual(_)), Some(Aperture::Manual(_))) => {
                self.set_tag(ExifTag::ExposureProgram(vec![1u16])) // "Manual"
            }
            (_, _) => {
                self.set_tag(ExifTag::ExposureProgram(vec![0u16])) // "Not Defined"
            }
        }

        // Set the EV compensation, if available
        if let Some(ExposureBias(bias)) = data.compensation {
            self.set_tag(ExifTag::ExposureCompensation(vec![
                iR64::from_rational(bias), //
            ]))
        }

        // Set the GPS position of this shot
        set_longitude(self, data.position.lon);
        set_latitude(self, data.position.lat);

        // Success!
        Ok(())
    }

    fn apply_author_data(
        &mut self,
        data: &Metadata,
        date: &Option<chrono::NaiveDate>,
    ) -> Result<(), super::NegativeError> {
        // Figure out what year this negative was shot, for the copyright
        let date = date.unwrap_or_else(|| chrono::Utc::now().date_naive());

        // Set the Artist & Copyright EXIF tags
        self.set_tag(ExifTag::Artist(data.author.name.to_owned()));
        self.set_tag(ExifTag::Copyright(data.copyright(date)));

        // Success!
        Ok(())
    }
}

/// Helper function for setting the GPS latitude EXIF tags
fn set_latitude(exif: &mut little_exif::metadata::Metadata, latitude: f64) {
    use dms_coordinates::{Cardinal, DMS};
    use num_traits::FromPrimitive;

    let lat = DMS::from_ddeg_latitude(latitude);
    exif.set_tag(ExifTag::GPSLatitude(vec![
        uR64::from_rational(num_rational::Rational32::from_integer(lat.degrees.into())),
        uR64::from_rational(num_rational::Rational32::from_integer(lat.minutes.into())),
        uR64::from_rational(num_rational::Rational32::from_f64(lat.seconds).unwrap_or_default()),
    ]));
    match lat.cardinal {
        Some(Cardinal::North) => exif.set_tag(ExifTag::GPSLatitudeRef("N".into())),
        Some(Cardinal::South) => exif.set_tag(ExifTag::GPSLatitudeRef("S".into())),
        _ => panic!("expected a valid latitude cardinal"),
    }
}

/// Helper function for setting the GPS longitude EXIF tags
fn set_longitude(exif: &mut little_exif::metadata::Metadata, longitude: f64) {
    use dms_coordinates::{Cardinal, DMS};
    use num_traits::FromPrimitive;

    let lon = DMS::from_ddeg_longitude(longitude);
    exif.set_tag(ExifTag::GPSLongitude(vec![
        uR64::from_rational(num_rational::Rational32::from_integer(lon.degrees.into())),
        uR64::from_rational(num_rational::Rational32::from_integer(lon.minutes.into())),
        uR64::from_rational(num_rational::Rational32::from_f64(lon.seconds).unwrap_or_default()),
    ]));
    match lon.cardinal {
        Some(Cardinal::East) => exif.set_tag(ExifTag::GPSLongitudeRef("E".into())),
        Some(Cardinal::West) => exif.set_tag(ExifTag::GPSLongitudeRef("W".into())),
        _ => panic!("expected a valid longitude cardinal"),
    }
}

/// Helper trait converting Rational to uR64/iR64
trait FromRational<T> {
    fn from_rational(value: num_rational::Ratio<T>) -> Self;
}

impl FromRational<i32> for uR64 {
    fn from_rational(value: num_rational::Ratio<i32>) -> Self {
        Self {
            nominator: (*value.numer() as i64).clamp(u32::MIN as i64, u32::MAX as i64) as u32,
            denominator: (*value.denom() as i64).clamp(u32::MIN as i64, u32::MAX as i64) as u32,
        }
    }
}

impl FromRational<i64> for uR64 {
    fn from_rational(value: num_rational::Ratio<i64>) -> Self {
        Self {
            nominator: (*value.numer()).clamp(u32::MIN as i64, u32::MAX as i64) as u32,
            denominator: (*value.denom()).clamp(u32::MIN as i64, u32::MAX as i64) as u32,
        }
    }
}

impl FromRational<i32> for iR64 {
    fn from_rational(value: num_rational::Ratio<i32>) -> Self {
        Self {
            nominator: *value.numer(),
            denominator: *value.denom(),
        }
    }
}

impl FromRational<i64> for iR64 {
    fn from_rational(value: num_rational::Ratio<i64>) -> Self {
        Self {
            nominator: (*value.numer()).clamp(i32::MIN as i64, i32::MAX as i64) as i32,
            denominator: (*value.denom()).clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        }
    }
}

/// Calculate the base-2 logarithm of a ratio
fn log2<T>(value: num_rational::Ratio<T>) -> num_rational::Ratio<T>
where
    T: Clone + num_traits::ToPrimitive + num_integer::Integer,
    num_rational::Ratio<T>: num_traits::FromPrimitive + num_traits::ToPrimitive,
{
    use num_traits::{FromPrimitive, ToPrimitive};
    || -> Option<num_rational::Ratio<T>> {
        num_rational::Ratio::<T>::from_f64(value.to_f64()?.log2())
    }()
    .expect("could not calculate base-2 logarithm of {value}")
    .reduced()
}

// Convert a string to an EXIF UCS-2 UNDEF value
fn to_exif_undef(
    value: &str,
    endian: little_exif::endian::Endian,
) -> little_exif::exif_tag_format::UNDEF {
    let mut bytes = Vec::<u8>::new();

    // Try storing UCS-2 only if necessary
    if !value.is_ascii() {
        let mut buffer = vec![0xFEFFu16; value.len() + 1];
        if let Ok(len) = ucs2::encode(value, &mut buffer[1..]) {
            buffer.truncate(len + 1);
            bytes.extend([0x55, 0x4E, 0x49, 0x43, 0x4F, 0x44, 0x45, 0x00]);
            match endian {
                little_exif::endian::Endian::Big => {
                    bytes.extend(buffer.into_iter().flat_map(u16::to_be_bytes))
                }
                little_exif::endian::Endian::Little => {
                    bytes.extend(buffer.into_iter().flat_map(u16::to_le_bytes))
                }
            };
            return bytes;
        }
    }

    // Otherwise, just store (sanitized) ASCII
    bytes.extend([0x41, 0x53, 0x43, 0x49, 0x49, 0x00, 0x00, 0x00]);
    bytes.extend(value.chars().filter(char::is_ascii).map(|c| c as u8));
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::*;
    use crate::negative::ApplyMetadata;
    use crate::rolls::*;
    use num_rational::Ratio;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn from_rational() {
        assert_eq!(
            uR64::from_rational(Ratio::new(i32::MIN, i32::MAX)),
            uR64 {
                nominator: u32::MIN,
                denominator: i32::MAX as u32,
            }
        );
        assert_eq!(
            uR64::from_rational(Ratio::new(i64::MIN, i64::MAX)),
            uR64 {
                nominator: u32::MIN,
                denominator: u32::MAX,
            }
        );
        assert_eq!(
            iR64::from_rational(Ratio::new(i32::MIN, i32::MAX)),
            iR64 {
                nominator: i32::MIN,
                denominator: i32::MAX,
            }
        );
        assert_eq!(
            iR64::from_rational(Ratio::new(i64::MIN, i64::MAX)),
            iR64 {
                nominator: i32::MIN,
                denominator: i32::MAX,
            }
        );
    }

    #[test]
    fn rational_log2() {
        assert_eq!(log2(Ratio::new(1, 2)), Ratio::new(-1, 1));
        assert_eq!(log2(Ratio::new(1, 1)), Ratio::new(0, 1));
        assert_eq!(log2(Ratio::new(2, 1)), Ratio::new(1, 1));
        assert_eq!(log2(Ratio::new(3, 1)), Ratio::new(85137581, 53715833));
        assert_eq!(log2(Ratio::new(25, 4)), Ratio::new(78830509, 29816489));
        assert_eq!(log2(Ratio::new(125, 1)), Ratio::new(343910773, 49371436));
    }

    #[test]
    fn exif_undef_encoding() {
        use little_exif::endian::Endian;
        assert_eq!(
            to_exif_undef("hello", Endian::Little),
            b"ASCII\x00\x00\x00hello"
        );
        assert_eq!(
            to_exif_undef("həˈləʊ", Endian::Little),
            b"UNICODE\x00\xFF\xFE\x68\x00\x59\x02\xC8\x02\x6C\x00\x59\x02\x8A\x02"
        );
        assert_eq!(
            to_exif_undef("həˈləʊ", Endian::Big),
            b"UNICODE\x00\xFE\xFF\x00\x68\x02\x59\x02\xC8\x00\x6C\x02\x59\x02\x8A"
        );
    }

    #[test]
    fn apply_roll_data() {
        let mut exif = little_exif::metadata::Metadata::new();
        let roll = Roll {
            id: "A1234".into(),
            film: Some(Film("Ilford Delta 100".into())),
            speed: FilmSpeed::from_din(21),
            camera: Some(Camera {
                make: "Voigtländer".into(),
                model: "Bessa R2M".into(),
            }),
            load: chrono::NaiveDateTime::MIN.and_utc().into(),
            unload: chrono::NaiveDateTime::MAX.and_utc().into(),
            frames: vec![],
        };
        exif.apply_roll_data(&roll)
            .expect("roll data should be applicable as EXIF");

        assert_eq!(
            exif.get_tag(&ExifTag::UnknownSTRING(
                String::new(),
                0xc615,
                ExifTagGroup::GENERIC
            ))
            .next(),
            roll.camera
                .as_ref()
                .map(|camera| ExifTag::UnknownSTRING(
                    camera.to_string(), //
                    0xc615,
                    ExifTagGroup::GENERIC
                ))
                .as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::Make(String::new())).next(),
            roll.camera
                .as_ref()
                .map(|c| c.make.clone())
                .map(ExifTag::Make)
                .as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::Model(String::new())).next(),
            roll.camera
                .as_ref()
                .map(|c| c.model.clone())
                .map(ExifTag::Model)
                .as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::UserComment(vec![])).next(),
            roll.film
                .map(|f| to_exif_undef(&f.0, exif.get_endian()))
                .map(ExifTag::UserComment)
                .as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::ISO(vec![])).next(),
            Some(ExifTag::ISO(vec![100u16])).as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::ISOSpeed(vec![])).next(),
            Some(ExifTag::ISOSpeed(vec![100u32])).as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::SensitivityType(vec![])).next(),
            Some(ExifTag::SensitivityType(vec![3u16])).as_ref()
        );
    }

    #[test]
    fn apply_frame_data() {
        let mut exif = little_exif::metadata::Metadata::new();
        let datetime = chrono::NaiveDate::from_ymd_opt(2025, 6, 1)
            .and_then(|date| date.and_hms_opt(12, 15, 00))
            .map(|date| date.and_utc());
        let frame = Frame {
            lens: Some(Lens {
                make: "Voigtländer".into(),
                model: "Color Skopar 35/2.5 Pancake II".into(),
            }),
            aperture: Some(Aperture::Manual(dec!(2.5))),
            shutter_speed: Some(ShutterSpeed::Manual(Ratio::new(1, 125))),
            focal_length: Some(FocalLength {
                real: dec!(35),
                equiv: Some(dec!(35)),
            }),
            compensation: Some(ExposureBias(Ratio::new(-1, 3))),
            datetime: datetime.unwrap().into(),
            position: Position { lat: 0.0, lon: 0.0 },
            note: None,
        };
        exif.apply_frame_data(&frame)
            .expect("frame data should be applicable as EXIF");

        assert_eq!(
            exif.get_tag(&ExifTag::DateTimeOriginal(String::new()))
                .next(),
            Some(ExifTag::DateTimeOriginal("2025:06:01 12:15:00".into())).as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::UnknownSTRING(
                String::new(),
                0xfdea,
                ExifTagGroup::EXIF
            ))
            .next(),
            frame
                .lens
                .as_ref()
                .map(|lens| ExifTag::UnknownSTRING(
                    lens.to_string(), //
                    0xfdea,
                    ExifTagGroup::EXIF
                ))
                .as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::LensMake(String::new())).next(),
            frame
                .lens
                .as_ref()
                .map(|c| c.make.clone())
                .map(ExifTag::LensMake)
                .as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::LensModel(String::new())).next(),
            frame
                .lens
                .as_ref()
                .map(|c| c.model.clone())
                .map(ExifTag::LensModel)
                .as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::FocalLength(vec![])).next(),
            Some(ExifTag::FocalLength(vec![35.into()])).as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::FocalLengthIn35mmFormat(vec![]))
                .next(),
            Some(ExifTag::FocalLengthIn35mmFormat(vec![35u16])).as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::ExposureTime(vec![])).next(),
            Some(ExifTag::ExposureTime(vec![0.008f64.into()])).as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::ShutterSpeedValue(vec![])).next(),
            Some(ExifTag::ShutterSpeedValue(vec![iR64 {
                nominator: 343910773,
                denominator: 49371436
            }]))
            .as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::FNumber(vec![])).next(),
            Some(ExifTag::FNumber(vec![2.5f64.into()])).as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::ApertureValue(vec![])).next(),
            Some(ExifTag::ApertureValue(vec![uR64 {
                nominator: 78830509,
                denominator: 29816489
            }]))
            .as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::ExposureProgram(vec![])).next(),
            Some(ExifTag::ExposureProgram(vec![1u16])).as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::ExposureCompensation(vec![])).next(),
            Some(ExifTag::ExposureCompensation(vec![iR64 {
                nominator: -1,
                denominator: 3
            }]))
            .as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::GPSLatitude(vec![])).next(),
            Some(ExifTag::GPSLatitude(vec![0.into(); 3])).as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::GPSLatitudeRef(String::new())).next(),
            Some(ExifTag::GPSLatitudeRef("N".into())).as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::GPSLongitude(vec![])).next(),
            Some(ExifTag::GPSLongitude(vec![0.into(); 3])).as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::GPSLongitudeRef(String::new()))
                .next(),
            Some(ExifTag::GPSLongitudeRef("E".into())).as_ref()
        );
    }

    #[test]
    fn apply_author_data() {
        let mut exif = little_exif::metadata::Metadata::new();
        let datetime = chrono::NaiveDate::from_ymd_opt(2025, 6, 1);
        let metadata = Metadata {
            author: Author {
                name: "Simon Sigurdhsson".into(),
                url: None,
            },
            license: None,
        };
        exif.apply_author_data(&metadata, &datetime)
            .expect("author/license data should be applicable as EXIF");

        assert_eq!(
            exif.get_tag(&ExifTag::Artist(String::new())).next(),
            Some(ExifTag::Artist(metadata.author.name.clone())).as_ref()
        );
        assert_eq!(
            exif.get_tag(&ExifTag::Copyright(String::new())).next(),
            Some(ExifTag::Copyright(metadata.copyright(datetime.unwrap()))).as_ref()
        );
    }
}
