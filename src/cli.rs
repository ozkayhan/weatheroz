use clap::Parser;
use clap::builder::styling::{AnsiColor, Styles};

// Custom color palette for the Clap CLI help output
pub fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().bold())
        .usage(AnsiColor::Green.on_default().bold())
        .literal(AnsiColor::Cyan.on_default().bold())
        .placeholder(AnsiColor::Magenta.on_default())
}

#[derive(Parser, Debug, Clone)]
#[command(
    name = "weather",
    about = "☀️  Weather OZ tool - Get weather information for any location with caching and parallel provider race.",
    styles = get_styles(),
    after_help = "EXAMPLES:\n  weather Istanbul\n  weather Berlin --days 3\n  weather Tokyo -m sparkline\n  weather \"Central Park, NY\" --enrich --mode html-preview"
)]
pub struct Args {
    #[arg(
        help = "City, district, neighborhood, coordinates, or any place name (e.g. 'Istanbul', 'Kadıköy', '41.0082,28.9784')",
        num_args = 1..
    )]
    pub location: Vec<String>,

    #[arg(
        short = 'f',
        long = "from-date",
        help = "Start date in YYYY-MM-DD format (default: today). Example: 2026-05-21"
    )]
    pub from_date: Option<String>,

    #[arg(
        short = 't',
        long = "to-date",
        help = "End date in YYYY-MM-DD format (default: today). Example: 2026-05-28"
    )]
    pub to_date: Option<String>,

    #[arg(
        short = 'a',
        long = "all-hours",
        help = "Show all hours (168 hours for 7 days) instead of just the first 24 hours"
    )]
    pub all_hours: bool,

    #[arg(
        short = 'j',
        long = "json-output",
        help = "Output raw JSON containing coordinates, full weather metrics, and race stats (shortcut for -m json)"
    )]
    pub json_output: bool,

    #[arg(
        short = 'v',
        long = "verbose",
        help = "Show detailed performance logs, API latency details, caching, and provider race times"
    )]
    pub verbose: bool,

    #[arg(
        short = 'm',
        long = "mode",
        help = "Visual output layout. Supported: default (table), compact (ASCII art), inline, json, emoji, ascii-banner, sparkline (24h trend), bordered-card, markdown, html-preview (launches browser)"
    )]
    pub mode: Option<String>,

    #[arg(
        short = 'e',
        long = "enrich",
        help = "Enable data enrichment (soil metrics, visibility, UV index, and AQI where supported by provider)"
    )]
    pub enrich: bool,

    #[arg(
        short = 'd',
        long = "days",
        help = "Fetch forecast for a custom number of days (up to 40 days where supported)"
    )]
    pub days: Option<u32>,

    #[arg(
        long = "minute",
        help = "Fetch minute-by-minute or high-frequency weather updates (where supported)"
    )]
    pub minute: bool,

    #[arg(
        long = "cache-ttl",
        help = "Override cache Time-To-Live in minutes. Set to 0 to disable cache and force a live API reload (default: 15)"
    )]
    pub cache_ttl: Option<u64>,
}

pub fn validate_date(date_str: &str) -> Result<(), String> {
    if chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err() {
        return Err("Date must be in YYYY-MM-DD format".to_string());
    }
    Ok(())
}

