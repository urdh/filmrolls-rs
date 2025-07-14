//! Implements [`super::ApplyMetadata`] for [xmp_toolkit]
//!
//! [xmp_toolkit]: https://docs.rs/xmp_toolkit/latest/xmp_toolkit/
use xmp_toolkit::xmp_ns::{DC, PHOTOSHOP, XMP_RIGHTS};
use xmp_toolkit::XmpValue;

use crate::metadata::{License, Metadata};
use crate::rolls::{Frame, Roll};

/// Creative commons XMP namespace
const CC: &str = "http://creativecommons.org/ns#";

impl super::ApplyMetadata for xmp_toolkit::XmpMeta {
    fn apply_roll_data(&mut self, _data: &Roll) -> Result<(), super::NegativeError> {
        Ok(())
    }

    fn apply_frame_data(&mut self, data: &Frame) -> Result<(), super::NegativeError> {
        // Photoshop tags
        self.set_property_date(
            PHOTOSHOP,
            "DateCreated",
            &XmpValue::new(data.datetime.and_utc().fixed_offset().into()),
        )?;

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
        let author = XmpValue::new(data.author.name.clone());

        // Clear the array tags, to have a clean slate
        self.delete_property(DC, "creator")?;
        self.delete_property(XMP_RIGHTS, "Owner")?;

        // Dublin Core tags
        self.append_array_item(
            DC,
            &XmpValue::new("creator".into()).set_is_array(true),
            &author,
        )?;
        self.set_localized_text(DC, "rights", None, "x-default", &data.copyright(date))?;

        // Photoshop tags
        self.set_property(
            PHOTOSHOP,
            "AuthorsPosition",
            &XmpValue::new("Photographer".into()),
        )?;

        // XMP Rights tags
        self.append_array_item(
            XMP_RIGHTS,
            &XmpValue::new("Owner".into()).set_is_array(true),
            &author,
        )?;
        if let Some(terms) = data.usage_terms() {
            let marked = data.license != Some(License::PublicDomain);
            self.set_property_bool(XMP_RIGHTS, "Marked", &XmpValue::new(marked))?;
            self.set_localized_text(XMP_RIGHTS, "UsageTerms", None, "x-default", &terms)?;
        }

        // Set the Artist & Copyright EXIF tags
        if let Some(license) = &data.license {
            let _ = Self::register_namespace(CC, "cc")?;
            self.set_property(CC, "license", &XmpValue::new(license.url().into()))?;
            self.set_property(CC, "attributionName", &author)?;
            if let Some(url) = &data.author.url {
                self.set_property(CC, "attributionURL", &XmpValue::new(url.clone()))?;
            }
        }

        // Success!
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::*;
    use crate::negative::ApplyMetadata;
    use crate::rolls::*;
    use crate::types::*;
    use itertools::assert_equal;
    use num_rational::Ratio;
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    #[test]
    fn apply_roll_data() {
        let mut xmp = xmp_toolkit::XmpMeta::new() //
            .expect("should be possible to initialize empty XMP data");
        let roll = Roll {
            id: "A1234".into(),
            film: Some(Film("Ilford Delta 100".into())),
            speed: FilmSpeed::from_din(21),
            camera: Some(Camera::MakeModel {
                make: "Voigtländer".into(),
                model: "Bessa R2M".into(),
            }),
            load: chrono::NaiveDateTime::MIN,
            unload: chrono::NaiveDateTime::MAX,
            frames: vec![],
        };
        xmp.apply_roll_data(&roll)
            .expect("roll data should be applicable as XMP");
    }

    #[test]
    fn apply_frame_data() {
        let mut xmp = xmp_toolkit::XmpMeta::new() //
            .expect("should be possible to initialize empty XMP data");
        let datetime = chrono::NaiveDate::from_ymd_opt(2025, 6, 1)
            .and_then(|date| date.and_hms_opt(12, 15, 00));
        let frame = Frame {
            lens: Some(Lens::MakeModel {
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
            datetime: datetime.unwrap(),
            position: Position { lat: 0.0, lon: 0.0 },
            note: None,
        };
        xmp.apply_frame_data(&frame)
            .expect("frame data should be applicable as XMP");

        assert_eq!(
            xmp.property_date(PHOTOSHOP, "DateCreated"),
            Some(XmpValue::new(
                frame.datetime.and_utc().fixed_offset().into()
            ))
        );
    }

    #[test]
    fn apply_author_data() {
        let mut xmp = xmp_toolkit::XmpMeta::new() //
            .expect("should be possible to initialize empty XMP data");
        let datetime = chrono::NaiveDate::from_ymd_opt(2025, 6, 1);
        let date = datetime.expect("should have a valid test case date/time");
        let metadata = Metadata {
            author: Author {
                name: "Simon Sigurdhsson".into(),
                url: None,
            },
            license: None,
        };
        xmp.apply_author_data(&metadata, &datetime)
            .expect("author/license data should be applicable as XMP");

        assert_equal(
            xmp.property_array(DC, "creator"),
            [XmpValue::new(metadata.author.name.clone())],
        );
        assert_eq!(
            xmp.localized_text(DC, "rights", None, "x-default"),
            Some((
                XmpValue::new(metadata.copyright(date))
                    .set_has_qualifiers(true)
                    .set_has_lang(true),
                "x-default".into()
            ))
        );
        assert_eq!(
            xmp.property(PHOTOSHOP, "AuthorsPosition"),
            Some(XmpValue::new("Photographer".into()))
        );
        assert_equal(
            xmp.property_array(XMP_RIGHTS, "Owner"),
            [XmpValue::new(metadata.author.name.clone())],
        );
        assert_eq!(xmp.property_bool(XMP_RIGHTS, "Marked"), None);
        assert_eq!(
            xmp.localized_text(XMP_RIGHTS, "UsageTerms", None, "x-default"),
            None
        );
        assert_eq!(xmp.property(CC, "license"), None);
        assert_eq!(xmp.property(CC, "attributionName"), None);
        assert_eq!(xmp.property(CC, "attributionURL"), None);
    }

    #[test]
    fn apply_author_data_replace_arrays() {
        let mut xmp = xmp_toolkit::XmpMeta::new() //
            .expect("should be possible to initialize empty XMP data");
        let metadata = Metadata {
            author: Author {
                name: "Simon Sigurdhsson".into(),
                url: None,
            },
            license: None,
        };

        // Start with populated creator/owner arrays
        xmp.append_array_item(
            DC,
            &XmpValue::new("creator".into()).set_is_array(true),
            &XmpValue::new("Existing Author".into()),
        )
        .expect("should be possible to set Dublin Core creator");
        xmp.append_array_item(
            XMP_RIGHTS,
            &XmpValue::new("Owner".into()).set_is_array(true),
            &XmpValue::new("Existing Author".into()),
        )
        .expect("should be possible to set XMP rights owner");

        xmp.apply_author_data(&metadata, &None)
            .expect("author/license data should be applicable as XMP");

        assert_equal(
            xmp.property_array(DC, "creator"),
            [XmpValue::new(metadata.author.name.clone())],
        );
        assert_equal(
            xmp.property_array(XMP_RIGHTS, "Owner"),
            [XmpValue::new(metadata.author.name.clone())],
        );
    }

    #[test]
    fn apply_author_data_with_license() {
        let mut xmp = xmp_toolkit::XmpMeta::new() //
            .expect("should be possible to initialize empty XMP data");
        let metadata = Metadata {
            author: Author {
                name: "Simon Sigurdhsson".into(),
                url: Some("http://photography.sigurdhsson.org/".into()),
            },
            license: Some(License::Attribution),
        };
        xmp.apply_author_data(&metadata, &None)
            .expect("author/license data should be applicable as XMP");

        assert_eq!(
            xmp.property_bool(XMP_RIGHTS, "Marked"),
            Some(XmpValue::new(true))
        );
        assert_eq!(
            xmp.localized_text(XMP_RIGHTS, "UsageTerms", None, "x-default"),
            metadata.usage_terms().map(|t| (
                XmpValue::new(t).set_has_qualifiers(true).set_has_lang(true),
                "x-default".into()
            ))
        );
        assert_eq!(
            xmp.property(CC, "license"),
            metadata.license.map(|l| XmpValue::new(l.url().into()))
        );
        assert_eq!(
            xmp.property(CC, "attributionName"),
            Some(XmpValue::new(metadata.author.name))
        );
        assert_eq!(
            xmp.property(CC, "attributionURL"),
            metadata.author.url.map(XmpValue::new)
        );
    }
}
