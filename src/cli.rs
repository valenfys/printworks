use std::path::PathBuf;
use std::str::FromStr;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "printworks",
    version,
    about = "Convert camera RAW photos into JPEGs"
)]
pub struct Cli {
    /// Increase log verbosity (-v, -vv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress non-error output
    #[arg(short, long, global = true)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Convert RAW files into JPEGs
    Convert(ConvertArgs),
    /// Print metadata for RAW files without converting them
    Info(InfoArgs),
}

#[derive(Args)]
pub struct ConvertArgs {
    /// RAW files or directories to convert
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,

    /// Directory to write JPEGs into (default: alongside each source file)
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,

    /// Recurse into subdirectories
    #[arg(short = 'r', long)]
    pub recursive: bool,

    /// JPEG quality (1-100)
    #[arg(long, default_value_t = 90, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub quality: u8,

    /// Number of parallel worker threads (default: number of CPUs)
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,

    /// Overwrite existing output files
    #[arg(long)]
    pub overwrite: bool,

    /// Exposure compensation in stops, applied on top of the camera default
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    pub exposure: f32,

    /// White balance: as-shot, a named preset (daylight, cloudy, shade,
    /// tungsten, fluorescent, flash), or <temp_kelvin>:<tint>
    #[arg(long, default_value = "as-shot")]
    pub wb: WhiteBalance,

    /// Rotation applied to the output
    #[arg(long, value_enum, default_value = "auto")]
    pub rotate: Rotate,

    /// Output file extension
    #[arg(long, value_enum, default_value = "jpg")]
    pub ext: OutputExt,
}

#[derive(Args)]
pub struct InfoArgs {
    /// RAW files or directories to inspect
    #[arg(required = true)]
    pub inputs: Vec<PathBuf>,

    /// Recurse into subdirectories
    #[arg(short = 'r', long)]
    pub recursive: bool,

    /// Print machine-readable JSON instead of text
    #[arg(long)]
    pub json: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum WhiteBalance {
    /// Use the white balance the camera recorded in the RAW file
    AsShot,
    /// Override with a specific color temperature (Kelvin) and tint
    Temp(f32, f32),
}

impl FromStr for WhiteBalance {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "as-shot" | "asshot" | "camera" => Ok(WhiteBalance::AsShot),
            "daylight" => Ok(WhiteBalance::Temp(5500.0, 1.0)),
            "cloudy" => Ok(WhiteBalance::Temp(6500.0, 1.0)),
            "shade" => Ok(WhiteBalance::Temp(7500.0, 1.0)),
            "tungsten" | "incandescent" => Ok(WhiteBalance::Temp(2850.0, 1.0)),
            "fluorescent" => Ok(WhiteBalance::Temp(4000.0, 1.0)),
            "flash" => Ok(WhiteBalance::Temp(5500.0, 1.0)),
            other => {
                let (temp, tint) = other.split_once(':').ok_or_else(|| {
                    format!("invalid --wb value '{other}': expected a preset name or <temp>:<tint>")
                })?;
                let temp: f32 = temp
                    .parse()
                    .map_err(|_| format!("invalid white balance temperature '{temp}'"))?;
                let tint: f32 = tint
                    .parse()
                    .map_err(|_| format!("invalid white balance tint '{tint}'"))?;
                Ok(WhiteBalance::Temp(temp, tint))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Rotate {
    /// Use the orientation recorded in the RAW file (default)
    Auto,
    /// Ignore the recorded orientation
    None,
    #[value(name = "90")]
    R90,
    #[value(name = "180")]
    R180,
    #[value(name = "270")]
    R270,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum OutputExt {
    Jpg,
    Jpeg,
}

impl OutputExt {
    pub fn as_str(self) -> &'static str {
        match self {
            OutputExt::Jpg => "jpg",
            OutputExt::Jpeg => "jpeg",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_definition_is_valid() {
        // Catches issues like duplicate short flags across global + subcommand args.
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_convert_flags() {
        let cli = Cli::parse_from([
            "printworks",
            "convert",
            "a.CR2",
            "-o",
            "out",
            "-r",
            "--quality",
            "80",
            "--exposure",
            "-0.5",
            "--wb",
            "5000:1.1",
            "--rotate",
            "90",
            "--ext",
            "jpeg",
        ]);
        let Command::Convert(args) = cli.command else {
            panic!("expected convert subcommand");
        };
        assert_eq!(args.inputs, vec![PathBuf::from("a.CR2")]);
        assert_eq!(args.output, Some(PathBuf::from("out")));
        assert!(args.recursive);
        assert_eq!(args.quality, 80);
        assert_eq!(args.exposure, -0.5);
        assert!(matches!(args.wb, WhiteBalance::Temp(t, ti) if t == 5000.0 && ti == 1.1));
        assert_eq!(args.rotate, Rotate::R90);
        assert_eq!(args.ext, OutputExt::Jpeg);
    }

    #[test]
    fn quality_out_of_range_is_rejected() {
        let result = Cli::try_parse_from(["printworks", "convert", "a.CR2", "--quality", "0"]);
        assert!(result.is_err());
    }

    #[test]
    fn white_balance_presets_parse() {
        assert!(matches!(
            "as-shot".parse::<WhiteBalance>().unwrap(),
            WhiteBalance::AsShot
        ));
        assert!(matches!(
            "daylight".parse::<WhiteBalance>().unwrap(),
            WhiteBalance::Temp(t, ti) if t == 5500.0 && ti == 1.0
        ));
        assert!("not-a-preset".parse::<WhiteBalance>().is_err());
    }

    #[test]
    fn parses_info_flags() {
        let cli = Cli::parse_from(["printworks", "info", "a.NEF", "--json"]);
        let Command::Info(args) = cli.command else {
            panic!("expected info subcommand");
        };
        assert_eq!(args.inputs, vec![PathBuf::from("a.NEF")]);
        assert!(args.json);
    }
}
