use clap::{ArgAction, Parser};
use framework_lib::chromium_ec::commands::RgbS;
use framework_lib::chromium_ec::{CrosEcDriverType, EcError};

use fwd_rgb::{apply_colors, parse_color};

/// Control the Framework RGB keyboard colors using the EC command directly.
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Args {
    /// Starting key index to program.
    #[arg(short, long, default_value_t = 0)]
    start: u8,

    /// Optional explicit EC driver selection.
    #[arg(long, value_enum)]
    driver: Option<CrosEcDriverType>,

    /// Enable verbose logging output.
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    /// One or more colors in 0xRRGGBB or decimal form.
    #[arg(value_name = "COLOR", num_args = 1..)]
    colors: Vec<String>,
}

fn configure_logging(level: u8) {
    if level == 0 {
        return;
    }

    let level = match level {
        0 => "error",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    // Allow overriding via environment while still respecting the CLI flag.
    let env = env_logger::Env::default().default_filter_or(level);
    let _ = env_logger::Builder::from_env(env)
        .format_target(false)
        .format_timestamp(None)
        .try_init();
}

fn main() {
    let args = Args::parse();
    configure_logging(args.verbose);

    if args.colors.is_empty() {
        eprintln!("error: at least one COLOR argument is required");
        std::process::exit(2);
    }

    let colors: Vec<RgbS> = match args.colors.iter().map(|value| parse_color(value)).collect() {
        Ok(colors) => colors,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    };

    if colors.is_empty() {
        eprintln!("no colors parsed from input");
        std::process::exit(2);
    }

    let color_count = colors.len();

    if let Err(err) = apply_colors(args.start, colors, args.driver) {
        match err {
            EcError::DeviceError(msg) => {
                eprintln!("ec device error: {msg}");
                std::process::exit(4);
            }
            other => {
                eprintln!("ec command failed: {other:?}");
                std::process::exit(5);
            }
        }
    }

    println!(
        "Updated {} key{} starting at {}",
        color_count,
        if color_count == 1 { "" } else { "s" },
        args.start
    );
}
