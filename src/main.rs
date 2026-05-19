mod calendar;
mod display;

use chrono::{Datelike, Local, NaiveDate};
use clap::{Parser, Subcommand};

use calendar::WeekConfig;

// ── CLI definition ────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "calx", about = "Calendar Expanded — calendar info in your terminal", version)]
struct Cli {
    /// Show 3 months side by side (previous, current, next)
    #[arg(short = 'm', long)]
    month: bool,

    /// Show full year calendar
    #[arg(short = 'y', long)]
    year: bool,

    /// Sunday start, simple weeks (Jan 1 = week 1)
    #[arg(long)]
    simple_sunday: bool,

    /// Sunday start, ISO-style weeks
    #[arg(long)]
    iso_sunday: bool,

    /// Monday start, simple weeks (Jan 1 = week 1)
    #[arg(long)]
    simple_monday: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show calendar with a custom week configuration
    Show {
        /// First day of week: 1 = Monday … 7 = Sunday
        #[arg(
            short = 'd',
            long,
            default_value_t = 1,
            value_parser = clap::value_parser!(u8).range(1..=7)
        )]
        day: u8,

        /// Simple week numbering (Jan 1 is always in week 1)
        #[arg(short = 's', long)]
        simple: bool,

        /// ISO-style week numbering (default)
        #[arg(short = 'i', long)]
        iso: bool,

        /// Show 3 months side by side
        #[arg(short = 'm', long)]
        month: bool,

        /// Show full year calendar
        #[arg(short = 'y', long)]
        year: bool,
    },
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let today = Local::now().date_naive();

    match cli.command {
        Some(Commands::Show { day, simple, iso, month, year }) => {
            if simple && iso {
                eprintln!("Error: --simple and --iso are mutually exclusive");
                std::process::exit(1);
            }
            let min_days: u8 = if simple { 1 } else { 4 };
            let cfg = WeekConfig { first_day: day, min_days };
            dispatch(today, cfg, month, year);
        }
        None => {
            let cfg = resolve_shortcut(cli.simple_sunday, cli.iso_sunday, cli.simple_monday);
            dispatch(today, cfg, cli.month, cli.year);
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Turn shortcut flags into a WeekConfig.  Exits with an error if more than
/// one flag is set — they are mutually exclusive.
fn resolve_shortcut(simple_sunday: bool, iso_sunday: bool, simple_monday: bool) -> WeekConfig {
    let count = [simple_sunday, iso_sunday, simple_monday]
        .iter()
        .filter(|&&b| b)
        .count();

    if count > 1 {
        eprintln!("Error: only one shortcut flag (--simple-sunday / --iso-sunday / --simple-monday) may be used at a time");
        std::process::exit(1);
    }

    if simple_sunday  { return WeekConfig { first_day: 7, min_days: 1 }; }
    if iso_sunday     { return WeekConfig { first_day: 7, min_days: 4 }; }
    if simple_monday  { return WeekConfig { first_day: 1, min_days: 1 }; }

    WeekConfig::default() // ISO Monday
}

/// Route to the correct view based on `show_month` / `show_year` flags.
fn dispatch(today: NaiveDate, cfg: WeekConfig, show_month: bool, show_year: bool) {
    if show_year {
        print!("{}", display::render_year(today.year(), Some(today), cfg));
    } else if show_month {
        print!("{}", display::render_three_months(today.year(), today.month(), Some(today), cfg));
    } else {
        display::display_default_view(today, cfg);
    }
}
