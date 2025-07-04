//! Command-line interface definition
use std::io::BufReader;
use std::process::ExitCode;

use ::clap::{Args, Parser, Subcommand};
use color_eyre::eyre::{Result, WrapErr};

use crate::{cmds, rolls};

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
        self.rolls
            .into_iter()
            .map(|input| {
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
                    result.wrap_err_with(|| {
                        format!("Failed to read roll data from {}", path.display())
                    })
                })
                .collect::<Vec<_>>()
            })
            .flatten()
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
                if let Some(table) = cmds::list_frames(film_roll.into_rolls(), &id)? {
                    println!("{}", Self::format_table(table).trim_fmt());
                    Ok(ExitCode::SUCCESS)
                } else {
                    println!("Could not find film roll with ID `{id}`");
                    Ok(ExitCode::FAILURE)
                }
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
