pub mod cli;
pub mod geocoding;
pub mod orchestrator;
pub mod output;
pub mod providers;
pub mod shared;
pub mod tui;
pub mod weather_cache;

use chrono::{NaiveDate, Utc};
use clap::Parser;
use std::io::IsTerminal;
use std::sync::{Arc, Mutex};

use crate::cli::{validate_date, Args};
use crate::output::{print_date_range_info, print_hourly_table, print_location_info};
use crate::tui::ProcessState;

fn get_log_path() -> std::path::PathBuf {
    let mut path = if let Ok(home) = std::env::var("HOME") {
        std::path::PathBuf::from(home)
    } else {
        std::path::PathBuf::from(".")
    };
    path.push(".cache");
    path.push("weather_oz");
    let _ = std::fs::create_dir_all(&path);
    path.push("weather.log");
    path
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Load JSON config
    let config_service = crate::shared::domain::config::JsonConfigService::new();
    use crate::shared::domain::config::ConfigService;
    let config = config_service.load_config().await.unwrap_or_default();

    let today_str = Utc::now().naive_utc().date().format("%Y-%m-%d").to_string();
    let start_date = args.from_date.clone().unwrap_or_else(|| today_str.clone());
    let end_date = args.to_date.clone().unwrap_or_else(|| today_str.clone());

    if let Err(e) = validate_date(&start_date) {
        crate::output::print_error_block(&e);
        std::process::exit(1);
    }
    if let Err(e) = validate_date(&end_date) {
        crate::output::print_error_block(&e);
        std::process::exit(1);
    }

    let use_tui = !args.json_output && !args.verbose && std::io::stdout().is_terminal();

    // 1. Initialize TUI SharedState if TUI is active
    let state = if use_tui {
        let initial_query = if args.location.is_empty() {
            "Otomatik Tespit".to_string()
        } else {
            args.location.join(" ")
        };
        Some(Arc::new(Mutex::new(ProcessState::new(initial_query))))
    } else {
        None
    };

    // 2. Setup Tracing Subscribers (central logging config)
    let log_path = get_log_path();
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path);

    use tracing_subscriber::prelude::*;

    let file_layer = file.ok().map(|f| {
        tracing_subscriber::fmt::layer()
            .with_writer(std::sync::Arc::new(f))
            .with_ansi(false)
            .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
    });

    let stderr_layer = if !use_tui {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("off"));
        Some(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_filter(filter),
        )
    } else {
        None
    };

    let tui_layer = state.as_ref().map(|st| {
        crate::tui::TuiLoggingLayer { state: st.clone() }
            .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
    });

    let _ = tracing_subscriber::registry()
        .with(file_layer)
        .with(stderr_layer)
        .with(tui_layer)
        .try_init();

    // 3. Create shared HTTP Client
    let client = Arc::new(reqwest::Client::new());

    // 4. Resolve Location Query (IP detection or CLI arg or user prompt)
    let mut resolved_location = None;
    let mut location_query = if args.location.is_empty() {
        None
    } else {
        Some(args.location.join(" "))
    };

    if location_query.is_none() {
        if args.verbose {
            println!("📡 Automatically detecting device location via IP...");
        }
        match crate::geocoding::resolve_ip_location(client.clone(), None, state.as_ref()).await {
            Ok(loc) => {
                if args.verbose {
                    println!(
                        "✅ IP location detection succeeded: {}, {}",
                        loc.name, loc.country
                    );
                }
                location_query = Some(loc.name.clone());
                resolved_location = Some(loc);
            }
            Err(e) => {
                if args.verbose {
                    println!("⚠ IP location detection failed: {}", e);
                }
                if std::io::stdin().is_terminal() {
                    eprint!("Device location could not be automatically detected. Please enter a location: ");
                    let mut input = String::new();
                    if std::io::stdin().read_line(&mut input).is_ok() {
                        let trimmed = input.trim().to_string();
                        if !trimmed.is_empty() {
                            location_query = Some(trimmed);
                        }
                    }
                }
                if location_query.is_none() {
                    crate::output::print_error_block("Device location could not be automatically detected, and no location was provided.");
                    std::process::exit(1);
                }
            }
        }
    }

    let location_query = location_query.unwrap();

    // Update query inside shared TUI state
    if let Some(ref st) = state {
        let mut guard = st.lock().unwrap();
        guard.query = location_query.clone();
    }

    // 5. Verbose Geocoding Cache check output (TUI is inactive)
    if args.verbose && !use_tui && resolved_location.is_none() {
        let query_key = location_query.to_lowercase().trim().to_string();
        let mut custom_path = std::env::var("WEATHER_OZ_CACHE_PATH")
            .ok()
            .map(std::path::PathBuf::from);
        if custom_path.is_none() {
            if let Ok(home) = std::env::var("HOME") {
                let mut path = std::path::PathBuf::from(home);
                path.push(".cache");
                path.push("weather_oz");
                path.push("geo_cache.json");
                custom_path = Some(path);
            }
        }

        let mut cache_hit = false;
        if let Some(path) = custom_path {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(cache) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(entry) = cache.get(&query_key) {
                            if let Some(timestamp) = entry.get("timestamp").and_then(|t| t.as_f64())
                            {
                                let now_secs = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs_f64();
                                if now_secs - timestamp < 30.0 * 24.0 * 60.0 * 60.0 {
                                    cache_hit = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        if cache_hit {
            println!("⚡ \x1b[1;32mCache Hit:\x1b[0m {}", location_query);
        } else {
            println!("🔍 \x1b[1;32mCache Miss:\x1b[0m {}", location_query);
        }
    }

    // 6. Run Orchestration
    let result = if use_tui {
        let state_clone = state.as_ref().unwrap().clone();
        let client_clone = client.clone();
        let args_clone = args.clone();
        let config_clone = config.clone();
        let location_query_clone = location_query.clone();
        let resolved_location_clone = resolved_location.clone();

        let task_handle = tokio::spawn(async move {
            crate::orchestrator::run_orchestrator(
                &args_clone,
                &config_clone,
                client_clone,
                location_query_clone,
                resolved_location_clone,
                Some(&state_clone),
            )
            .await
        });

        let mut stdout = std::io::stdout();
        let _ = crossterm::terminal::enable_raw_mode();
        let _ = crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen);
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let mut terminal = match ratatui::Terminal::new(backend) {
            Ok(t) => t,
            Err(e) => {
                let _ = crossterm::terminal::disable_raw_mode();
                crate::output::print_error_block(&e.to_string());
                std::process::exit(1);
            }
        };

        let mut exit_err = None;
        let mut final_result = None;

        loop {
            {
                let guard = state.as_ref().unwrap().lock().unwrap();
                let _ = terminal.draw(|f| {
                    crate::tui::renderer::draw_dashboard(f, &guard);
                });
            }

            if task_handle.is_finished() {
                match task_handle.await {
                    Ok(Ok(res)) => {
                        final_result = Some(res);
                    }
                    Ok(Err(e)) => {
                        exit_err = Some(e.to_string());
                    }
                    Err(e) => {
                        exit_err = Some(e.to_string());
                    }
                }
                break;
            }

            if let Ok(true) = crossterm::event::poll(std::time::Duration::from_millis(100)) {
                if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
                    if key.code == crossterm::event::KeyCode::Char('q')
                        || key.code == crossterm::event::KeyCode::Esc
                    {
                        exit_err = Some("User cancelled".to_string());
                        break;
                    }
                }
            }
        }

        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        );
        let _ = terminal.show_cursor();

        if let Some(err) = exit_err {
            crate::output::print_error_block(&err);
            std::process::exit(1);
        }

        final_result.unwrap()
    } else {
        match crate::orchestrator::run_orchestrator(
            &args,
            &config,
            client.clone(),
            location_query.clone(),
            resolved_location,
            None,
        )
        .await
        {
            Ok(res) => res,
            Err(e) => {
                crate::output::print_error_block(&e.to_string());
                std::process::exit(1);
            }
        }
    };

    // 7. Process Output
    if args.verbose && !use_tui {
        if result.winner_name == "Cache" && !result.is_offline {
            println!(
                "⚡ \x1b[1;32mWeather Cache Hit:\x1b[0m Loaded fresh weather data from cache."
            );
        } else if result.winner_name == "Cache" && result.is_offline {
            println!(
                "🔍 \x1b[1;33mWeather Cache Stale:\x1b[0m Cached data is older than 15 minutes."
            );
        } else {
            // It was a race! Print matching verbose outputs
            let today_obj = Utc::now().naive_utc().date();
            let s_date = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
            let mut eligible = vec!["Open-Meteo", "wttr.in"];
            if s_date >= today_obj {
                eligible.insert(1, "MET Norway");
            }
            println!(
                "🚀 \x1b[1;34mStarting parallel race for:\x1b[0m {}",
                eligible.join(", ")
            );

            println!("\n🏁 \x1b[1mThread Details:\x1b[0m");
            let mut keys: Vec<&String> = result.stats.keys().collect();
            keys.sort();
            for p_name in keys {
                if let Some(p_stat) = result.stats.get(p_name) {
                    if p_stat.success {
                        println!(
                            "  - {}: \x1b[32mSUCCESS\x1b[0m ({:.1}ms)",
                            p_name, p_stat.time_ms
                        );
                    } else {
                        println!(
                            "  - {}: \x1b[31mFAILED\x1b[0m ({:.1}ms) - {}",
                            p_name,
                            p_stat.time_ms,
                            p_stat.error.as_deref().unwrap_or("")
                        );
                    }
                }
            }
            let winner_time = result
                .stats
                .get(&result.winner_name)
                .map(|s| s.time_ms)
                .unwrap_or(0.0);
            println!(
                "\n🏆 \x1b[1;33mWinner:\x1b[0m \x1b[1m{}\x1b[0m ({:.1}ms)\n",
                result.winner_name, winner_time
            );
        }
    }

    let mode = args.mode.clone().unwrap_or_else(|| {
        if args.json_output {
            "json".to_string()
        } else {
            "default".to_string()
        }
    });

    if mode != "default" {
        let today_obj = Utc::now().naive_utc().date();
        let s = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
        let e = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").unwrap();

        let data_type = if e < today_obj {
            "historical"
        } else if s >= today_obj {
            "forecast"
        } else {
            "mixed"
        };

        crate::output::render_mode(
            &mode,
            &result.resolved_location,
            &result.weather_data.hourly,
            &start_date,
            &end_date,
            data_type,
            result.is_offline,
            result.cached_timestamp,
            &result.weather_data.provider_name,
        );
        return;
    }

    if result.is_offline {
        if let Some(ts) = result.cached_timestamp {
            crate::output::print_offline_warning(ts);
        }
    }

    print_location_info(&result.resolved_location);

    let today_obj = Utc::now().naive_utc().date();
    let s = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d").unwrap();
    let e = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d").unwrap();

    let data_type = if e < today_obj {
        "historical"
    } else if s >= today_obj {
        "forecast"
    } else {
        "mixed"
    };

    print_date_range_info(&start_date, &end_date, data_type);

    let max_rows = if args.all_hours { 168 } else { 24 };
    print_hourly_table(&result.weather_data.hourly, max_rows);
}
