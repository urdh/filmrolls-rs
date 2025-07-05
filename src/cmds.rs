//! Command-line interface implementations
use std::string::ToString;

use color_eyre::eyre::{Report, Result};
use comfy_table::Table;
use itertools::{EitherOrBoth, Itertools};

use crate::{negative, rolls};

/// Generate a `Table` containing the given `rolls`
///
/// This function generates a [comfy-table] `Table` containing information
/// about all film rolls in the input iterator. If any of the film rolls
/// resolve to an error, this function will return that error instead of
/// a table; all rolls must be successfully parsed in order to generate a
/// valid table.
///
/// [comfy-table]: https://docs.rs/comfy-table/latest/comfy_table/
pub fn list_rolls<I>(rolls: I) -> Result<Table>
where
    I: Iterator<Item = Result<rolls::Roll>>,
{
    let mut table = Table::new();
    table.set_header(vec![
        "ID",       // roll.id
        "Frames",   // roll.frames.len(),
        "Film",     // roll.film + roll.speed
        "Camera",   // roll.camera
        "Loaded",   // roll.load
        "Unloaded", // roll.unload
    ]);
    rolls
        .sorted_by_cached_key(|roll| roll.as_ref().map(|r| r.id.clone()).unwrap_or_default())
        .try_fold(table, |mut table, roll| {
            let roll = roll?;
            table.add_row(vec![
                roll.id.to_string(),
                roll.frames.len().to_string(),
                format!(
                    "{} @ {}",
                    roll.film
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                    roll.speed
                ),
                roll.camera
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_default(),
                roll.load.to_string(),
                roll.unload.to_string(),
            ]);
            Ok(table)
        })
}

/// Find a specific roll given its ID
///
/// This function searches through the input iterator to find a matching
/// film roll (based on the ID field), and returns that specific roll if
/// found.  If any of the film rolls resolve to an error, this function
/// will return that error instead of a roll, and if no roll with a matching
/// ID can be found this function will return `Ok(None)`.
pub fn find_roll<I>(mut rolls: I, id: &str) -> Result<Option<rolls::Roll>>
where
    I: Iterator<Item = Result<rolls::Roll>>,
{
    rolls
        .find_map(|roll| match roll {
            Err(error) => Some(Err(error)),
            Ok(roll) if roll.id == id => Some(Ok(roll)),
            _ => None,
        })
        .transpose()
}

/// Generate a `Table` containing information about a given roll
///
/// This function generates a [comfy-table] `Table` containing information
/// about all frames in the input film roll.
///
/// [comfy-table]: https://docs.rs/comfy-table/latest/comfy_table/
pub fn list_frames(roll: rolls::Roll) -> Table {
    let mut table = Table::new();
    table.set_header(vec![
        "#",          // frame_nbr
        "Lens",       // frame.lens
        "Focal len.", // frame.focal_length
        "Aperture",   // frame.aperture
        "Shutter",    // frame.shutter_speed
        "Comp.",      // frame.compensation
        "Date",       // frame.datetime
        "Location",   // frame.position
        "Notes",      // frame.note
    ]);
    roll.frames
        .into_iter()
        .enumerate()
        .fold(table, |mut table, (idx, frame)| {
            let frame_nbr = idx + 1;
            table.add_row(
                frame
                    .map(|frame| {
                        vec![
                            frame_nbr.to_string(), //
                            frame
                                .lens
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_default(),
                            frame
                                .focal_length
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_default(),
                            frame
                                .aperture
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_default(),
                            frame
                                .shutter_speed
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_default(),
                            frame
                                .compensation
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_default(),
                            frame.datetime.to_string(),
                            frame.position.to_string(),
                            frame
                                .note
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_default(),
                        ]
                    })
                    .unwrap_or_else(|| vec![frame_nbr.to_string()]),
            );
            table
        })
}

/// Get a list of frame/negative pairs
///
/// Constructs a list of frame/negative pairs by matching each input frame
/// with the corresponding negative, where the order of the images is assumed
/// to match the frame order. If the number of images does not match the
/// number of frames, or if an error occurs while opening any image, an error
/// is returned instead.
pub fn match_negatives<'a>(
    frames: impl Iterator<Item = &'a Option<rolls::Frame>>,
    negatives: impl Iterator<Item = Result<negative::Negative>>,
) -> Result<Vec<(&'a rolls::Frame, negative::Negative)>> {
    frames
        .filter_map(|s| s.as_ref())
        .zip_longest(negatives)
        .map(|pair| match pair {
            EitherOrBoth::Left(_) | EitherOrBoth::Right(_) => {
                Err(Report::msg("Frame count does not match image count"))
            }
            EitherOrBoth::Both(_, Err(err)) => Err(err)?,
            EitherOrBoth::Both(frame, Ok(negative)) => Ok((frame, negative)),
        })
        .try_collect()
}

/// Generate a `Table` containing the given `rolls`
///
/// This function generates a [comfy-table] `Table` containing information
/// about all negatives in the input iterator. If any of the negatives
/// resolve to an error, this function will return that error instead of
/// a table; all negatives must be successfully parsed in order to generate
/// a valid table.
///
/// [comfy-table]: https://docs.rs/comfy-table/latest/comfy_table/
pub fn list_negatives<I>(mut negatives: I) -> Result<Table>
where
    I: Iterator<Item = Result<negative::Negative>>,
{
    let mut table = Table::new();
    table.set_header(vec![
        "Roll", // negative.roll()
        "Date", // negative.date()
        "Path", // negative.path()
    ]);
    negatives.try_fold(table, |mut table, negative| {
        let negative = negative?;
        table.add_row(vec![
            negative.roll().map(ToString::to_string).unwrap_or_default(),
            negative
                .date()
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            negative.path().display().to_string(),
        ]);
        Ok(table)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::negative::*;
    use crate::rolls::*;
    use crate::types::*;
    use chrono::{DateTime, Utc};
    use itertools::assert_equal;
    use pretty_assertions::assert_eq;

    fn get_test_roll() -> Result<Roll> {
        Ok(Roll {
            id: "A0012".into(),
            film: Some(Film("Ilford Delta 100".into())),
            speed: FilmSpeed::from_din(21), // ISO 100/21°
            camera: "Voigtländer Bessa R2M".try_into().ok(),
            load: DateTime::<Utc>::UNIX_EPOCH.into(),
            unload: DateTime::<Utc>::UNIX_EPOCH.into(),
            frames: vec![
                None,
                Some(Frame {
                    lens: "Voigtländer Color Skopar 35/2.5 Pancake II".try_into().ok(),
                    aperture: Some(Aperture::from(rust_decimal::Decimal::new(56, 1))),
                    shutter_speed: Some(ShutterSpeed::from(num_rational::Ratio::new(1, 500))),
                    focal_length: None,
                    compensation: None,
                    datetime: DateTime::<Utc>::UNIX_EPOCH.into(),
                    position: Position {
                        lat: 57.700767,
                        lon: 11.953715,
                    },
                    note: None,
                }),
                None,
            ],
        })
    }

    #[test]
    fn list_rolls_empty() {
        let mut table = list_rolls(std::iter::empty()) //
            .expect("an empty iterator should not propagate any errors");
        assert_eq!(table.column_count(), 6);
        assert_eq!(table.row_count(), 0);
    }

    #[test]
    fn list_rolls_single() {
        let mut table = list_rolls(std::iter::once(get_test_roll()))
            .expect("an iterator with no errors should not propagate any errors");
        assert_eq!(table.column_count(), 6);
        assert_eq!(table.row_count(), 1);
    }

    #[test]
    fn list_rolls_error() {
        let error = crate::rolls::SourceError::InvalidData("...");
        let table = list_rolls(std::iter::once(Err(error.into())))
            .expect_err("all errors should propagate to the caller");
        assert_eq!(
            table.downcast_ref::<crate::rolls::SourceError>(),
            Some(&crate::rolls::SourceError::InvalidData("..."))
        );
    }

    #[test]
    fn find_roll_no_match() {
        let rolls = find_roll(std::iter::once(get_test_roll()), "A0013")
            .expect("an iterator with no errors should not propagate any errors");
        assert!(rolls.is_none());
    }

    #[test]
    fn find_roll_one_match() {
        let rolls = find_roll(std::iter::once(get_test_roll()), "A0012")
            .expect("an iterator with no errors should not propagate any errors");
        assert!(rolls.is_some());
    }

    #[test]
    fn find_roll_error() {
        let error = crate::rolls::SourceError::InvalidData("...");
        let rolls = find_roll(std::iter::once(Err(error.into())), "A0012")
            .expect_err("all errors should propagate to the caller");
        assert_eq!(
            rolls.downcast_ref::<crate::rolls::SourceError>(),
            Some(&crate::rolls::SourceError::InvalidData("..."))
        );
    }

    #[test]
    fn list_frames_one_match() {
        let mut table = list_frames(get_test_roll().unwrap());
        assert_eq!(table.column_count(), 9);
        assert_eq!(table.row_count(), 3);
        assert_equal(
            table.row_iter().map(comfy_table::Row::cell_count),
            vec![1, 9, 1],
        );
    }

    #[test]
    fn match_negatives_short() {
        let roll = get_test_roll().unwrap();
        let _ = match_negatives(roll.frames.iter(), std::iter::empty())
            .expect_err("too few negatives should generate an error");
    }

    #[test]
    fn match_negatives_long() {
        let roll = get_test_roll().unwrap();
        let _ = match_negatives(
            roll.frames.iter(),
            std::iter::repeat(Negative::new()).map(Ok),
        )
        .expect_err("too many negatives should generate an error");
    }

    #[test]
    fn match_negatives_error() {
        let roll = get_test_roll().unwrap();
        let error = crate::rolls::SourceError::InvalidData("...");
        let pairs = match_negatives(roll.frames.iter(), std::iter::once(Err(error.into())))
            .expect_err("all errors should propagate to the caller");
        assert_eq!(
            pairs.downcast_ref::<crate::rolls::SourceError>(),
            Some(&crate::rolls::SourceError::InvalidData("..."))
        );
    }

    #[test]
    fn match_negatives_ok() {
        let roll = get_test_roll().unwrap();
        let pairs = match_negatives(roll.frames.iter(), std::iter::once(Ok(Negative::new())))
            .expect("matching lengths with no errors should not propagate any errors");
        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn list_negatives_empty() {
        let mut table = list_negatives(std::iter::empty()) //
            .expect("an empty iterator should not propagate any errors");
        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 0);
    }

    #[test]
    fn list_negatives_single() {
        let mut table = list_negatives(std::iter::once(Ok(Negative::new())))
            .expect("an iterator with no errors should not propagate any errors");
        assert_eq!(table.column_count(), 3);
        assert_eq!(table.row_count(), 1);
    }

    #[test]
    fn list_negatives_error() {
        let error = crate::rolls::SourceError::InvalidData("...");
        let table = list_negatives(std::iter::once(Err(error.into())))
            .expect_err("all errors should propagate to the caller");
        assert_eq!(
            table.downcast_ref::<crate::rolls::SourceError>(),
            Some(&crate::rolls::SourceError::InvalidData("..."))
        );
    }
}
