//! Interface to an on-disk "negative"
//!
//! This module provides a *negative* definition which represents an
//! on-disk image file with associated EXIF and XMP metadata. It also
//! provides a trait allowing film roll and author metadata to be
//! applied to the on-disk image.
use little_exif::{exif_tag::ExifTag, ifd::ExifTagGroup};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::metadata::Metadata;
use crate::rolls::{Frame, Roll};

mod exif;
mod xmp;

/// Metadata application errors
#[derive(Debug)]
#[derive(thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum NegativeError {
    /// Generic I/O error
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    /// XMP Toolkit error
    #[error(transparent)]
    XmpError(#[from] xmp_toolkit::XmpError),

    /// UTF8 conversion error
    #[error(transparent)]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

/// A "negative" (image with metadata)
#[derive(Clone)]
pub struct Negative {
    exif: little_exif::metadata::Metadata,
    xmp: xmp_toolkit::XmpMeta,
    path: PathBuf,
    roll: Option<String>,
}

impl std::fmt::Debug for Negative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Negative")
            .field("exif", &"..")
            .field("xmp", &self.xmp)
            .field("path", &self.path)
            .field("roll", &self.roll)
            .finish()
    }
}

impl Negative {
    /// Create a new negative based on the given image
    ///
    /// This will open up the given path for reading, and extract both EXIF
    /// and XMP data if available. Only file formats supported by [little_exif]
    /// are supported; XMP data is extracted from the EXIF IFD and fed directly
    /// to the XMP Toolkit to avoid the toolkit reconciling legacy tags.
    ///
    /// [little_exif]: https://docs.rs/little_exif/latest/little_exif/
    pub fn new_from_path(path: &Path) -> Result<Negative, NegativeError> {
        let exif_data = little_exif::metadata::Metadata::new_from_path(path)?;
        let xmp_data = exif_data
            .get_tag(&ExifTag::UnknownINT8U(
                vec![],
                0x02bc,
                ExifTagGroup::GENERIC,
            ))
            .next()
            .and_then(|tag| match tag {
                ExifTag::UnknownUNDEF(value, _, _) => Some(value),
                ExifTag::UnknownINT8U(value, _, _) => Some(value),
                _ => None,
            })
            .map(|data| -> Result<xmp_toolkit::XmpMeta, NegativeError> {
                String::from_utf8(data.to_vec())
                    .map_err(Into::<NegativeError>::into)
                    .and_then(|s| Ok(FromStr::from_str(&s)?))
            })
            .unwrap_or_else(|| Ok(xmp_toolkit::XmpMeta::new()?));
        Ok(Self {
            exif: exif_data,
            xmp: xmp_data?,
            path: path.into(),
            roll: None,
        })
    }

    #[cfg(test)]
    /// Create a new, empty, path-less negative
    pub(crate) fn new() -> Negative {
        Self {
            exif: little_exif::metadata::Metadata::new(),
            xmp: xmp_toolkit::XmpMeta::new()
                .expect("it should be possible to create empty XMP metadata"),
            path: PathBuf::new(),
            roll: None,
        }
    }

    /// Get the path of this negative
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the roll ID of this negative, if any
    pub fn roll(&self) -> Option<&str> {
        self.roll.as_deref()
    }

    /// Get the original date/time of this negative, if any
    pub fn date(&self) -> Option<chrono::NaiveDateTime> {
        use little_exif::exif_tag::ExifTag;
        None.or_else(|| {
            self.exif
                .get_tag(&ExifTag::DateTimeOriginal(String::new()))
                .next()
        })
        .or_else(|| {
            self.exif
                .get_tag(&ExifTag::CreateDate(String::new()))
                .next()
        })
        .and_then(|tag| match tag {
            ExifTag::DateTimeOriginal(s) | ExifTag::CreateDate(s) => {
                chrono::NaiveDateTime::parse_from_str(s, "%Y:%m:%d %H:%M:%S").ok()
            }
            _ => None,
        })
    }

    /// Save the metadata back to the source file
    ///
    /// As with [`Negative::new_from_path`], this will use [little_exif] to write
    /// EXIF tags to the source file, bypassing the XMP Toolkit reconciliation.
    ///
    /// [little_exif]: https://docs.rs/little_exif/latest/little_exif/
    pub fn save(&mut self) -> Result<(), NegativeError> {
        use xmp_toolkit::ToStringOptions;
        self.exif.set_tag(ExifTag::UnknownINT8U(
            self.xmp
                .to_string_with_options(ToStringOptions::default().use_compact_format())?
                .into_bytes(),
            0x02bc,
            ExifTagGroup::GENERIC,
        ));
        self.exif.write_to_file(&self.path)?;
        Ok(())
    }
}

/// Apply film roll and author metadata to a negative
///
/// This trait is used to apply [`Roll`], [`Frame`], and [`Metadata`] to `Self`, which
/// when applied to [`Negative`] will simply forward the application to the underlying
/// EXIF and XMP representations held in memory.
pub trait ApplyMetadata {
    /// Apply [`Roll`] metadata to `self`
    fn apply_roll_data(&mut self, data: &Roll) -> Result<(), NegativeError>;

    /// Apply [`Frame`] metadata to `self`
    fn apply_frame_data(&mut self, data: &Frame) -> Result<(), NegativeError>;

    /// Apply author metadata to `self`
    ///
    /// Since the author metadata (in particular, the copyright information)
    /// requires an associated `date`, this may be passed externally. If not
    /// included, a sensible fall-back value should be used (i.e. the original
    /// date/time of the image if available, or if all else fails the current
    /// date/time).
    fn apply_author_data(
        &mut self,
        data: &Metadata,
        date: &Option<chrono::NaiveDate>,
    ) -> Result<(), NegativeError>;
}

impl ApplyMetadata for Negative {
    fn apply_roll_data(&mut self, data: &Roll) -> Result<(), NegativeError> {
        self.exif.apply_roll_data(data)?;
        self.xmp.apply_roll_data(data)?;
        self.roll = Some(data.id.clone());
        Ok(())
    }

    fn apply_frame_data(&mut self, data: &Frame) -> Result<(), NegativeError> {
        self.exif.apply_frame_data(data)?;
        self.xmp.apply_frame_data(data)?;
        Ok(())
    }

    fn apply_author_data(
        &mut self,
        data: &Metadata,
        date: &Option<chrono::NaiveDate>,
    ) -> Result<(), NegativeError> {
        let date = date.or_else(|| self.date().map(|d| d.date()));
        self.exif.apply_author_data(data, &date)?;
        self.xmp.apply_author_data(data, &date)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rolls::*;
    use crate::types::*;
    use chrono::Timelike;
    use pretty_assertions::assert_eq;

    #[test]
    fn default_frame_details() {
        let negative = Negative::new();
        assert_eq!(negative.path(), PathBuf::new());
        assert_eq!(negative.roll(), None);
        assert_eq!(negative.date(), None);
    }

    #[test]
    fn updated_frame_details() {
        let mut negative = Negative::new();
        let datetime = chrono::Utc::now().naive_local();
        negative
            .apply_roll_data(&Roll {
                id: "A1234".into(),
                film: None,
                speed: FilmSpeed::from_din(21),
                camera: None,
                load: chrono::NaiveDateTime::MIN,
                unload: chrono::NaiveDateTime::MAX,
                frames: vec![],
            })
            .expect("roll data should be applicable to negative");
        negative
            .apply_frame_data(&Frame {
                lens: None,
                aperture: None,
                shutter_speed: None,
                focal_length: None,
                compensation: None,
                datetime,
                position: Default::default(),
                note: None,
            })
            .expect("frame data should be applicable to negative");

        assert_eq!(negative.path(), PathBuf::new());
        assert_eq!(negative.roll(), Some("A1234"));
        assert_eq!(negative.date(), datetime.with_nanosecond(0));
    }
}
