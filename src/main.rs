#![allow(dead_code)]
#![allow(unknown_lints)] // for forward-compat with newer clippy lints
#![allow(clippy::manual_is_multiple_of)] // is_multiple_of() requires nightly
mod charstate;
mod easing;
mod effects;
mod engine;
mod gradient;
#[cfg(test)]
mod tests;

use clap::Parser;
use engine::{run_animation, Grid};
use rand::seq::SliceRandom;
use std::io::{self, Read};

fn build_effects_help() -> String {
    let mut s = String::from("TTE Effects:\n");
    for info in effects::ALL_EFFECTS {
        if !info.extra_effect {
            s.push_str(&format!("  {:<20}{}\n", info.name, info.description));
        }
    }
    let extras: Vec<_> = effects::ALL_EFFECTS
        .iter()
        .filter(|e| e.extra_effect)
        .collect();
    if !extras.is_empty() {
        s.push_str("\nExtra Effects:\n");
        for info in extras {
            s.push_str(&format!("  {:<20}{}\n", info.name, info.description));
        }
    }
    s.push_str("\nEx: ls -a | rtte decrypt\n    echo HELLO | rtte --random-effect");
    s
}

#[derive(Parser)]
#[command(
    name = "rtte",
    version = env!("CARGO_PKG_VERSION"),
    disable_version_flag = true,
    about = "Rust port of terminaltexteffects (tte).",
    after_long_help = build_effects_help(),
)]
struct Cli {
    /// Effect to apply. Use -h to see the list of available effects
    #[arg(value_name = "EFFECT")]
    effect: Option<String>,

    /// Randomly select an effect to apply
    #[arg(short = 'R', long = "random-effect")]
    random_effect: bool,

    /// Target frame rate for the animation
    #[arg(long = "frame-rate", default_value = "60")]
    frame_rate: u32,

    /// File to read input from
    #[arg(short = 'i', long = "input-file")]
    input_file: Option<String>,

    /// Space-separated list of effects to include when randomly selecting an effect
    #[arg(long = "include-effects", num_args = 1..)]
    include_effects: Option<Vec<String>>,

    /// Space-separated list of effects to exclude when randomly selecting an effect
    #[arg(long = "exclude-effects", num_args = 1..)]
    exclude_effects: Option<Vec<String>>,

    /// Do not restore cursor visibility after the effect
    #[arg(long = "no-restore-cursor")]
    _no_restore_cursor: bool,

    /// Show version
    #[arg(short = 'v', long = "version", action = clap::ArgAction::Version)]
    _version: (),
}

fn main() {
    let cli = Cli::parse();

    // Read input
    let input = if let Some(path) = &cli.input_file {
        std::fs::read_to_string(path).expect("Failed to read input file")
    } else {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .expect("Failed to read stdin");
        buf
    };

    if input.trim().is_empty() {
        return;
    }

    // Select effect
    let mut rng = rand::thread_rng();
    let effect_name = if cli.random_effect {
        let mut pool: Vec<&str> = if let Some(ref inc) = cli.include_effects {
            effects::ALL_EFFECTS
                .iter()
                .filter(|e| inc.iter().any(|i| i == e.name))
                .map(|e| e.name)
                .collect()
        } else if let Some(ref exc) = cli.exclude_effects {
            effects::ALL_EFFECTS
                .iter()
                .filter(|e| !exc.iter().any(|x| x == e.name))
                .map(|e| e.name)
                .collect()
        } else {
            effects::ALL_EFFECTS.iter().map(|e| e.name).collect()
        };
        if pool.is_empty() {
            pool = effects::ALL_EFFECTS.iter().map(|e| e.name).collect();
        }
        pool.choose(&mut rng).unwrap().to_string()
    } else if let Some(ref e) = cli.effect {
        e.clone()
    } else {
        eprintln!("Error: specify an effect or use --random-effect");
        std::process::exit(1);
    };

    let mut grid = Grid::from_input(&input);

    let info = effects::ALL_EFFECTS
        .iter()
        .find(|e| e.name == effect_name)
        .unwrap_or_else(|| {
            eprintln!("Unknown effect: {effect_name}");
            std::process::exit(1);
        });

    let mut effect = (info.create)(&grid);

    run_animation(&mut grid, cli.frame_rate, |grid, _frame| effect.tick(grid));
}
