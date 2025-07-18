//! Command-line interface definition
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use ::clap::{Args, Parser, Subcommand};
use color_eyre::eyre::{Result, WrapErr};

use crate::negative::ApplyMetadata;
use crate::{cmds, metadata, negative, rolls};

#[doc(hidden)]
mod shadow {
    shadow_rs::shadow!(build);
}

/// Tag TIFF files with EXIF data extracted from XML
///
/// Reads XML data from the Film Rolls iOS app and displays or applies this data
/// as EXIF tags to a given set of files.
#[derive(Parser)]
#[command(
    name = "filmrolls",
    version,
    author,
    long_version = shadow::build::CLAP_LONG_VERSION,
    arg_required_else_help(true),
)]
pub struct Cli {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[command(subcommand)]
    command: Commands,
}

impl Cli {
    /// Initialize logging, based on arguments
    pub fn init_logging(&self) {
        env_logger::Builder::new()
            .filter_level(self.global_opts.verbose.log_level_filter())
            .init();
    }

    /// Initialize color handling, based on arguments
    pub fn init_colors(&self) -> Result<()> {
        let hooks = color_eyre::config::HookBuilder::default();
        match self.global_opts.color {
            clap::ColorChoice::Always => {
                owo_colors::set_override(true);
                hooks.theme(color_eyre::config::Theme::dark()).install()
            }
            clap::ColorChoice::Auto => {
                owo_colors::unset_override();
                hooks.theme(color_eyre::config::Theme::dark()).install()
            }
            clap::ColorChoice::Never => {
                owo_colors::set_override(false);
                hooks.theme(color_eyre::config::Theme::new()).install()
            }
        }
    }

    /// Run the selected subcommand
    pub fn run_command(self) -> Result<ExitCode> {
        self.command.run()
    }
}

#[derive(Debug, Args)]
struct GlobalOpts {
    /// Whether to use colors or not
    #[clap(long, global = true, value_name = "WHEN", default_value = "auto")]
    color: clap::ColorChoice,

    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct FilmRoll {
    /// Input film roll data file(s)
    #[clap(long, short = 'r', value_parser, value_name = "FILE")]
    rolls: Vec<clio::Input>,
}

impl FilmRoll {
    /// Read & parse the given film roll data file
    fn into_rolls(self) -> impl Iterator<Item = Result<rolls::Roll>> {
        self.rolls.into_iter().flat_map(|input| {
            let path = input.path().path();
            let reader = BufReader::new(input.clone());
            use rolls::SourceError::UnsupportedFormat;
            match mime_guess::from_path(path)
                .first_or_octet_stream()
                .essence_str()
            {
                "text/xml" => RollIter::XmlSource(rolls::from_filmrolls(reader)),
                "application/json" => RollIter::JsonSource(rolls::from_lightme(reader)),
                mime => RollIter::from_error(UnsupportedFormat(mime.to_owned())),
            }
            .map(move |result| -> Result<rolls::Roll> {
                result.wrap_err_with(|| format!("Failed to read roll data from {}", path.display()))
            })
            .collect::<Vec<_>>()
        })
    }
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct Metadata {
    /// Author metadata
    #[clap(long, short = 'm', value_parser, value_name = "FILE")]
    meta: clio::Input,
}

impl Metadata {
    /// Read & parse the given author metadata file
    fn into_meta(mut self) -> Result<metadata::Metadata> {
        let mut buf = String::new();
        self.meta.read_to_string(&mut buf).wrap_err_with(|| {
            format!(
                "Failed to read author metadata from {}",
                self.meta.path().display()
            )
        })?;
        toml::de::from_str(&buf).wrap_err_with(|| {
            format!(
                "Failed to parse author metadata from {}",
                self.meta.path().display()
            )
        })
    }
}

#[derive(Args)]
#[group(required = false, multiple = false)]
struct Images {
    /// Image file(s) to modify
    #[clap(value_parser)]
    images: Vec<PathBuf>,
}

impl Images {
    /// Read metadata from all input images
    fn into_negatives(self) -> impl Iterator<Item = Result<negative::Negative>> {
        self.images
            .into_iter()
            .map(|p| negative::Negative::new_from_path(p.as_ref()).map_err(Into::into))
    }
}

#[derive(Subcommand)]
enum Commands {
    /// List ID and additional data for all film rolls in input
    ListRolls {
        #[clap(flatten)]
        film_roll: FilmRoll,
    },

    /// List frames from film roll with ID in input
    ListFrames {
        #[clap(flatten)]
        film_roll: FilmRoll,

        /// Use data from roll with id ID
        #[clap(long, short)]
        id: String,
    },

    /// Write EXIF tags to a set of images using data from film roll with ID in input
    Tag {
        #[clap(flatten)]
        film_roll: FilmRoll,

        /// Use data from roll with id ID
        #[clap(long, short)]
        id: String,

        /// Don't actually modify any files
        #[clap(long, short = 'n')]
        dry_run: bool,

        #[clap(flatten)]
        images: Images,
    },

    /// Write author metadata to a set of images using YAML data from file
    ApplyMetadata {
        #[clap(flatten)]
        metadata: Metadata,

        /// Don't actually modify any files
        #[clap(long, short = 'n')]
        dry_run: bool,

        #[clap(flatten)]
        images: Images,
    },
}

impl Commands {
    /// Run the selected subcommand
    fn run(self) -> Result<ExitCode> {
        match self {
            Self::ListRolls { film_roll } => {
                let table = cmds::list_rolls(film_roll.into_rolls())?;
                println!("{}", Self::format_table(table).trim_fmt());
                Ok(ExitCode::SUCCESS)
            }
            Self::ListFrames { film_roll, id } => {
                if let Some(roll) = cmds::find_roll(film_roll.into_rolls(), &id)? {
                    let table = cmds::list_frames(roll);
                    println!("{}", Self::format_table(table).trim_fmt());
                    Ok(ExitCode::SUCCESS)
                } else {
                    println!("Could not find film roll with ID `{id}`");
                    Ok(ExitCode::FAILURE)
                }
            }
            Self::Tag {
                film_roll,
                id,
                dry_run,
                images,
            } => {
                if let Some(roll) = cmds::find_roll(film_roll.into_rolls(), &id)? {
                    // Match frames & images, apply metadata, and optionally save to file
                    let negatives =
                        cmds::match_negatives(roll.frames.iter(), images.into_negatives())?
                            .into_iter()
                            .map(|(frame, mut negative)| {
                                negative.apply_roll_data(&roll)?;
                                negative.apply_frame_data(frame)?;
                                if !dry_run {
                                    negative.save()?;
                                }
                                Ok(negative)
                            });

                    // Print a brief summary of the images being modified
                    let table = cmds::list_negatives(negatives)?;
                    println!("{}", Self::format_table(table).trim_fmt());
                    Ok(ExitCode::SUCCESS)
                } else {
                    println!("Could not find film roll with ID `{id}`");
                    Ok(ExitCode::FAILURE)
                }
            }
            Self::ApplyMetadata {
                metadata,
                dry_run,
                images,
            } => {
                // Load negatives, apply metadata, and optionally save to file
                let metadata = metadata.into_meta()?;
                let negatives = images.into_negatives().map(|negative| {
                    negative.and_then(|mut negative| {
                        negative.apply_author_data(&metadata, &None)?;
                        if !dry_run {
                            negative.save()?;
                        }
                        Ok(negative)
                    })
                });

                // Print a brief summary of the images being modified
                let table = cmds::list_negatives(negatives)?;
                println!("{}", Self::format_table(table).trim_fmt());
                Ok(ExitCode::SUCCESS)
            }
        }
    }

    // Apply formatting to the given table
    fn format_table(mut table: comfy_table::Table) -> comfy_table::Table {
        use comfy_table::presets::UTF8_HORIZONTAL_ONLY;
        use comfy_table::ContentArrangement;
        table
            .load_preset(UTF8_HORIZONTAL_ONLY)
            .set_content_arrangement(ContentArrangement::Dynamic);
        table
    }
}

enum RollIter<E, XmlIter, JsonIter>
where
    XmlIter: Iterator<Item = Result<rolls::Roll, E>>,
    JsonIter: Iterator<Item = Result<rolls::Roll, E>>,
{
    XmlSource(XmlIter),
    JsonSource(JsonIter),
    Error(std::iter::Once<Result<rolls::Roll, E>>),
}

impl<E, XmlIter, JsonIter> RollIter<E, XmlIter, JsonIter>
where
    XmlIter: Iterator<Item = Result<rolls::Roll, E>>,
    JsonIter: Iterator<Item = Result<rolls::Roll, E>>,
{
    pub fn from_error(error: E) -> Self {
        Self::Error(std::iter::once(Err(error)))
    }
}

impl<E, XmlIter, JsonIter> Iterator for RollIter<E, XmlIter, JsonIter>
where
    XmlIter: Iterator<Item = Result<rolls::Roll, E>>,
    JsonIter: Iterator<Item = Result<rolls::Roll, E>>,
{
    type Item = Result<rolls::Roll, E>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::XmlSource(iter) => iter.next(),
            Self::JsonSource(iter) => iter.next(),
            Self::Error(iter) => iter.next(),
        }
    }
}
