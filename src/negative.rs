//! Trait for applying various metadata to a single image
use std::path::{Path, PathBuf};

use crate::metadata::Metadata;
use crate::rolls::{Frame, Roll};

mod exif;

/// Metadata application errors
#[derive(Debug)]
#[derive(thiserror::Error)]
pub enum NegativeError {
    /// Generic I/O error
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

/// A "negative" (image with metadata)
#[derive(Clone)]
pub struct Negative {
    exif: little_exif::metadata::Metadata,
    path: PathBuf,
    roll: Option<String>,
}

impl std::fmt::Debug for Negative {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Negative")
            .field("exif", &"..")
            .field("path", &self.path)
            .field("roll", &self.roll)
            .finish()
    }
}

impl Negative {
    /// Create a new negative based on the given image
    pub fn new_from_path(path: &Path) -> Result<Negative, NegativeError> {
        Ok(Self {
            exif: little_exif::metadata::Metadata::new_from_path(path)?,
            path: path.into(),
            roll: None,
        })
    }

    /// Create a new, empty, path-less negative
    #[cfg(test)]
    pub(crate) fn new() -> Negative {
        Self {
            exif: little_exif::metadata::Metadata::new(),
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
    pub fn save(&self) -> Result<(), NegativeError> {
        self.exif.write_to_file(&self.path)?;
        Ok(())
    }
}

/// Trait for applying metadata to an image
pub trait ApplyMetadata {
    fn apply_roll_data(&mut self, data: &Roll) -> Result<(), NegativeError>;
    fn apply_frame_data(&mut self, data: &Frame) -> Result<(), NegativeError>;
    fn apply_author_data(
        &mut self,
        data: &Metadata,
        date: &Option<chrono::NaiveDate>,
    ) -> Result<(), NegativeError>;
}

impl ApplyMetadata for Negative {
    fn apply_roll_data(&mut self, data: &Roll) -> Result<(), NegativeError> {
        self.exif.apply_roll_data(data)?;
        self.roll = Some(data.id.clone());
        Ok(())
    }

    fn apply_frame_data(&mut self, data: &Frame) -> Result<(), NegativeError> {
        self.exif.apply_frame_data(data)?;
        Ok(())
    }

    fn apply_author_data(
        &mut self,
        data: &Metadata,
        date: &Option<chrono::NaiveDate>,
    ) -> Result<(), NegativeError> {
        let date = date.or_else(|| self.date().map(|d| d.date()));
        self.exif.apply_author_data(data, &date)?;
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
        let datetime = chrono::Utc::now();
        negative
            .apply_roll_data(&Roll {
                id: "A1234".into(),
                film: None,
                speed: FilmSpeed::from_din(21),
                camera: None,
                load: chrono::NaiveDateTime::MIN.and_utc().into(),
                unload: chrono::NaiveDateTime::MAX.and_utc().into(),
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
                datetime: datetime.into(),
                position: Default::default(),
                note: None,
            })
            .expect("frame data should be applicable to negative");

        assert_eq!(negative.path(), PathBuf::new());
        assert_eq!(negative.roll(), Some("A1234"));
        assert_eq!(
            negative.date(),
            datetime.with_nanosecond(0).map(|d| d.naive_utc())
        );
    }
}
