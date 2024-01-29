//! Command-line interface implementations
use std::string::ToString;

use color_eyre::eyre::Result;
use comfy_table::{Row, Table};

use crate::rolls;

/// Generate a `Table` containing the given `rolls`
///
/// This function generates a [comfy-table] `Table` containing information
/// about all film rolls in the input iterator. If any of the film rolls
/// resolve to an error, this function will return that error instead of
/// a table; all rolls must be successfully parsed in order to generate a
/// valid table.
///
/// [comfy-table]: https://docs.rs/comfy-table/latest/comfy_table/
pub fn list_rolls<I>(mut rolls: I) -> Result<Table>
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
    rolls.try_fold(table, |mut table, roll| {
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

/// Generate a `Table` containing information about a given roll
///
/// This function searches through the input iterator to find a matching
/// film roll (based on the ID field), and generates a [comfy-table] `Table`
/// containing information about the frames of that roll. If any of the film
/// rolls resolve to an error, this function will return that error instead
/// of a table, and if no roll with a matching ID can be found this function
/// will return `Ok(None)`.
///
/// [comfy-table]: https://docs.rs/comfy-table/latest/comfy_table/
pub fn list_frames<I>(mut rolls: I, id: &str) -> Result<Option<Table>>
where
    I: Iterator<Item = Result<rolls::Roll>>,
{
    rolls
        .find_map(|roll| match roll {
            Err(error) => Some(Err(error)),
            Ok(roll) if roll.id == id => {
                let mut table = Table::new();
                table.set_header(vec![
                    "#",        // frame_nbr
                    "Lens",     // frame.lens
                    "Aperture", // frame.aperture
                    "Shutter",  // frame.shutter_speed
                    "Comp.",    // frame.compensation
                    "Date",     // frame.datetime
                    "Location", // frame.position
                    "Notes",    // frame.note
                ]);
                table.add_rows(roll.frames.into_iter().enumerate().map(|(idx, frame)| {
                    let frame_nbr = idx + 1;
                    if let Some(frame) = frame {
                        Row::from(vec![
                            frame_nbr.to_string(), //
                            frame
                                .lens
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_default(),
                            frame.aperture.to_string(),
                            frame.shutter_speed.to_string(),
                            frame.compensation.to_string(),
                            frame.datetime.to_string(),
                            frame.position.to_string(),
                            frame
                                .note
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_default(),
                        ])
                    } else {
                        Row::from(vec![frame_nbr.to_string()])
                    }
                }));
                Some(Ok(table))
            }
            _ => None,
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rolls::*;
    use crate::types::*;
    use chrono::{DateTime, Utc};
    use itertools::assert_equal;
    use pretty_assertions::assert_eq;
    use rust_decimal::prelude::Zero;

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
                    aperture: Aperture::from(rust_decimal::Decimal::new(56, 1)),
                    shutter_speed: ShutterSpeed::from(num_rational::Ratio::new(1, 500)),
                    compensation: ExposureBias::from(num_rational::Ratio::zero()),
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
    fn list_frames_no_match() {
        let table = list_frames(std::iter::once(get_test_roll()), "A0013")
            .expect("an iterator with no errors should not propagate any errors");
        assert!(table.is_none());
    }

    #[test]
    fn list_frames_one_match() {
        let mut table = list_frames(std::iter::once(get_test_roll()), "A0012")
            .expect("an iterator with no errors should not propagate any errors")
            .expect("when there's a matching frame, the result is not `None`");
        assert_eq!(table.column_count(), 8);
        assert_eq!(table.row_count(), 3);
        assert_equal(table.row_iter().map(Row::cell_count), vec![1, 8, 1]);
    }

    #[test]
    fn list_frames_error() {
        let error = crate::rolls::SourceError::InvalidData("...");
        let table = list_frames(std::iter::once(Err(error.into())), "A0012")
            .expect_err("all errors should propagate to the caller");
        assert_eq!(
            table.downcast_ref::<crate::rolls::SourceError>(),
            Some(&crate::rolls::SourceError::InvalidData("..."))
        );
    }
}
