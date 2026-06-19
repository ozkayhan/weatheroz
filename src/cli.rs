use clap::builder::styling::{AnsiColor, Styles};
use clap::{Parser, ValueEnum};
use clap_complete::Shell;

/// Visual output layouts. Variant names map to kebab-case CLI values (e.g. `AsciiBanner` -> `ascii-banner`).
#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Mode {
    Default,
    Compact,
    Inline,
    Json,
    Emoji,
    AsciiBanner,
    Sparkline,
    BorderedCard,
    Markdown,
    HtmlPreview,
}

impl Mode {
    /// Canonical lowercase string consumed by the output renderer's match.
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Default => "default",
            Mode::Compact => "compact",
            Mode::Inline => "inline",
            Mode::Json => "json",
            Mode::Emoji => "emoji",
            Mode::AsciiBanner => "ascii-banner",
            Mode::Sparkline => "sparkline",
            Mode::BorderedCard => "bordered-card",
            Mode::Markdown => "markdown",
            Mode::HtmlPreview => "html-preview",
        }
    }
}

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
    name = "weatheroz",
    about = "☀️  weatheroz - Get weather information for any location with caching and parallel provider race.",
    styles = get_styles(),
    after_help = "EXAMPLES:\n  weatheroz Istanbul\n  weatheroz Berlin --days 3\n  weatheroz Tokyo -m sparkline\n  weatheroz \"Central Park, NY\" --enrich --mode html-preview"
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
        help = "Visual output layout. default (table), compact (ASCII art), inline, json, emoji, ascii-banner, sparkline (24h trend), bordered-card, markdown, html-preview (launches browser)"
    )]
    pub mode: Option<Mode>,

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

    #[arg(
        long = "completions",
        help = "Generate a shell completion script for the given shell (bash, zsh, fish, powershell, elvish) and exit"
    )]
    pub completions: Option<Shell>,
}

pub fn validate_date(date_str: &str) -> Result<(), String> {
    if chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err() {
        return Err("Date must be in YYYY-MM-DD format".to_string());
    }
    Ok(())
}
