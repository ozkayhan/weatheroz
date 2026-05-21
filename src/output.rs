use chrono::{NaiveDateTime, Datelike, Timelike};
use comfy_table::presets::NOTHING;
use comfy_table::Table;
use std::collections::HashMap;
use std::path::PathBuf;
use crate::geocoding::GeocodedLocation;
use crate::providers::models::HourlyPoint;

fn get_weather_info(weather_code: i32) -> (&'static str, &'static str) {
    match weather_code {
        0 => ("☀️", "Clear"),
        1 => ("🌤️", "Mainly clear"),
        2 => ("⛅", "Partly cloudy"),
        3 => ("☁️", "Overcast"),
        45 => ("🌫️", "Fog"),
        48 => ("🌫️", "Depositing rime fog"),
        51 => ("🌦️", "Light drizzle"),
        53 => ("🌦️", "Moderate drizzle"),
        55 => ("🌦️", "Dense drizzle"),
        56 => ("🌦️", "Light freezing drizzle"),
        57 => ("🌦️", "Dense freezing drizzle"),
        61 => ("🌧️", "Slight rain"),
        63 => ("🌧️", "Moderate rain"),
        65 => ("🌧️", "Heavy rain"),
        66 => ("🌧️", "Light freezing rain"),
        67 => ("🌧️", "Heavy freezing rain"),
        71 => ("🌨️", "Slight snow"),
        73 => ("🌨️", "Moderate snow"),
        75 => ("🌨️", "Heavy snow"),
        77 => ("🌨️", "Snow grains"),
        80 => ("🌧️", "Slight rain showers"),
        81 => ("🌧️", "Moderate rain showers"),
        82 => ("🌧️", "Violent rain showers"),
        85 => ("🌨️", "Slight snow showers"),
        86 => ("🌨️", "Heavy snow showers"),
        95 => ("⛈️", "Thunderstorm"),
        96 => ("⛈️", "Thunderstorm with slight hail"),
        99 => ("⛈️", "Thunderstorm with heavy hail"),
        _ => ("❓", "Unknown"),
    }
}

fn get_temp_color(temp: f64) -> String {
    let rounded = temp.round() as i32;
    if temp <= 5.0 {
        format!("\x1b[34m{}\x1b[0m", rounded)
    } else if temp <= 15.0 {
        format!("\x1b[32m{}\x1b[0m", rounded)
    } else {
        format!("\x1b[31m{}\x1b[0m", rounded)
    }
}

pub fn print_location_info(location: &GeocodedLocation) {
    let mut parts = Vec::new();
    if !location.name.is_empty() {
        parts.push(location.name.as_str());
    }
    if !location.admin1.is_empty() {
        parts.push(location.admin1.as_str());
    }
    if !location.country.is_empty() {
        parts.push(location.country.as_str());
    }
    let location_str = parts.join(", ");
    println!();
    println!("📍 \x1b[1;36m{}\x1b[0m", location_str);
    println!("   Coordinates: {:.4}°, {:.4}°", location.latitude, location.longitude);
    println!();
}

pub fn print_date_range_info(start_date: &str, end_date: &str, data_type: &str) {
    use chrono::NaiveDate;
    let start_dt = NaiveDate::parse_from_str(start_date, "%Y-%m-%d");
    let end_dt = NaiveDate::parse_from_str(end_date, "%Y-%m-%d");

    let date_str = match (start_dt, end_dt) {
        (Ok(s), Ok(e)) => {
            if s == e {
                s.format("%B %d, %Y").to_string()
            } else {
                format!("{} - {}", s.format("%B %d"), e.format("%B %d, %Y"))
            }
        }
        _ => format!("{} to {}", start_date, end_date),
    };

    let type_label = match data_type {
        "historical" => "Historical Data",
        "mixed" => "Historical + Forecast",
        _ => "Forecast",
    };

    println!("📅 \x1b[1m{}\x1b[0m ({})", date_str, type_label);
    println!();
}

pub fn print_offline_warning(timestamp: f64) {
    use chrono::{TimeZone, Local};
    let formatted = match Local.timestamp_opt(timestamp as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        _ => {
            use chrono::Utc;
            match Utc.timestamp_opt(timestamp as i64, 0) {
                chrono::LocalResult::Single(dt) => format!("{} UTC", dt.format("%Y-%m-%d %H:%M:%S")),
                _ => "unknown time".to_string(),
            }
        }
    };
    println!("⚡ \x1b[1;33mOffline Mode:\x1b[0m \x1b[1mCached data from {}\x1b[0m", formatted);
    println!();
}

pub fn print_hourly_table(hourly_points: &[HourlyPoint], max_rows: usize) {
    if hourly_points.is_empty() {
        println!("\x1b[2mNo hourly weather data available.\x1b[0m");
        return;
    }

    let mut is_multi_day = false;
    if hourly_points.len() > 1 {
        let first_date = &hourly_points[0].time[0..10];
        let last_date = &hourly_points[hourly_points.len() - 1].time[0..10];
        is_multi_day = first_date != last_date;
    }

    let display_count = std::cmp::min(max_rows, hourly_points.len());
    let has_more = hourly_points.len() > max_rows;

    let mut table = Table::new();
    table.load_preset(NOTHING);
    table.set_header(vec![
        "Time",
        "Temp\n(°C)",
        "Feels\n(°C)",
        "Precip\nProb (%)",
        "Precip\n(mm)",
        "Humidity\n(%)",
        "Wind",
        "Cloud\n(%)",
        "Weather",
    ]);

    for pt in hourly_points.iter().take(display_count) {
        let cleaned = pt.time.replace('Z', "");
        let mut final_str = cleaned.clone();
        if cleaned.len() == 13 {
            final_str.push_str(":00");
        }
        let parsed = NaiveDateTime::parse_from_str(&final_str, "%Y-%m-%dT%H:%M").ok();
        let formatted_t = match parsed {
            Some(dt) => {
                if is_multi_day {
                    format!("{:02} {:02}:{:02}", dt.date().day(), dt.time().hour(), dt.time().minute())
                } else {
                    format!("{:02}:{:02}", dt.time().hour(), dt.time().minute())
                }
            }
            None => pt.time.clone(),
        };

        let temp_str = get_temp_color(pt.temperature);
        let feel_str = pt.apparent_temperature.round().to_string();
        let pp_str = pt.precipitation_probability.round().to_string();
        let precip_str = if pt.precipitation > 0.0 {
            format!("{:.1}", pt.precipitation)
        } else {
            "-".to_string()
        };
        let hum_str = pt.humidity.round().to_string();

        let directions = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
        let dir_idx = ((pt.wind_direction / 45.0).round() as usize) % 8;
        let wind_dir_compass = directions[dir_idx];
        let wind_str = format!("{} {}", pt.wind_speed.round(), wind_dir_compass);

        let cloud_str = pt.cloud_cover.round().to_string();
        let (emoji, desc) = get_weather_info(pt.weather_code);
        let weather_str = format!("{} {}", emoji, desc);

        table.add_row(vec![
            formatted_t,
            temp_str,
            feel_str,
            pp_str,
            precip_str,
            hum_str,
            wind_str,
            cloud_str,
            weather_str,
        ]);
    }

    println!("{}", table);

    if has_more {
        println!();
        println!("\x1b[2mShowing first {} of {} hours. Use --all-hours to see all.\x1b[0m", max_rows, hourly_points.len());
    }
}

// ---------------------------------------------------------
// NEW VISUAL MODES FORMATTING LOGIC
// ---------------------------------------------------------

fn get_large_char(c: char, line: usize) -> &'static str {
    match (c, line) {
        ('0', 0) => "  ███  ", ('0', 1) => " ██ ██ ", ('0', 2) => "██   ██", ('0', 3) => " ██ ██ ", ('0', 4) => "  ███  ",
        ('1', 0) => "   ██  ", ('1', 1) => "  ███  ", ('1', 2) => "   ██  ", ('1', 3) => "   ██  ", ('1', 4) => " █████ ",
        ('2', 0) => "  ███  ", ('2', 1) => " ██ ██ ", ('2', 2) => "   ██  ", ('2', 3) => "  ██   ", ('2', 4) => " █████ ",
        ('3', 0) => " ████  ", ('3', 1) => "    ██ ", ('3', 2) => "  ███  ", ('3', 3) => "    ██ ", ('3', 4) => " ████  ",
        ('4', 0) => " ██  ██", ('4', 1) => " ██  ██", ('4', 2) => " ██████", ('4', 3) => "     ██", ('4', 4) => "     ██",
        ('5', 0) => " █████ ", ('5', 1) => " ██    ", ('5', 2) => " ████  ", ('5', 3) => "    ██ ", ('5', 4) => " ████  ",
        ('6', 0) => "  ███  ", ('6', 1) => " ██    ", ('6', 2) => " ████  ", ('6', 3) => " ██ ██ ", ('6', 4) => "  ███  ",
        ('7', 0) => " █████ ", ('7', 1) => "    ██ ", ('7', 2) => "   ██  ", ('7', 3) => "  ██   ", ('7', 4) => " ██    ",
        ('8', 0) => "  ███  ", ('8', 1) => " ██ ██ ", ('8', 2) => "  ███  ", ('8', 3) => " ██ ██ ", ('8', 4) => "  ███  ",
        ('9', 0) => "  ███  ", ('9', 1) => " ██ ██ ", ('9', 2) => "  ████ ", ('9', 3) => "    ██ ", ('9', 4) => "  ███  ",
        ('-', 0) => "       ", ('-', 1) => "       ", ('-', 2) => " █████ ", ('-', 3) => "       ", ('-', 4) => "       ",
        ('.', 0) => "       ", ('.', 1) => "       ", ('.', 2) => "       ", ('.', 3) => "  ██   ", ('.', 4) => "  ██   ",
        ('C', 0) => "  ████ ", ('C', 1) => " ██    ", ('C', 2) => " ██    ", ('C', 3) => " ██    ", ('C', 4) => "  ████ ",
        ('°', 0) => "  ██   ", ('°', 1) => " ██ ██ ", ('°', 2) => "  ██   ", ('°', 3) => "       ", ('°', 4) => "       ",
        _ => "       ",
    }
}

fn get_ascii_art(weather_code: i32) -> Vec<&'static str> {
    match weather_code {
        0 | 1 => vec![
            "   \\   /   ",
            "    .-.    ",
            " -- ( ) -- ",
            "    '-'    ",
            "   /   \\   ",
        ],
        2 | 3 => vec![
            "   \\  /    ",
            "  _ / /_   ",
            " (   _  )  ",
            "  '-' '-'  ",
            "           ",
        ],
        51..=67 | 80..=82 => vec![
            "    .-.    ",
            "   (   )   ",
            "  (_) (_)  ",
            "   / / /   ",
            "  / / /    ",
        ],
        71..=77 | 85..=86 => vec![
            "    .-.    ",
            "   (   )   ",
            "  (_) (_)  ",
            "  * * * *  ",
            " * * * *   ",
        ],
        95..=99 => vec![
            "    .-.    ",
            "   (   )   ",
            "  (_) (_)  ",
            "   ⚡  ⚡    ",
            "    ⚡  ⚡   ",
        ],
        _ => vec![
            "    .-.    ",
            "   ( ? )   ",
            "    '-'    ",
            "           ",
            "           ",
        ],
    }
}

pub fn render_mode(
    mode: &str,
    location: &GeocodedLocation,
    hourly_points: &[HourlyPoint],
    start_date: &str,
    end_date: &str,
    data_type: &str,
    is_offline: bool,
    offline_time: Option<f64>,
    provider_name: &str,
) {
    if is_offline {
        if let Some(t) = offline_time {
            print_offline_warning(t);
        }
    }

    if hourly_points.is_empty() {
        println!("No hourly weather data available.");
        return;
    }

    let current = &hourly_points[0];
    let (emoji, desc) = get_weather_info(current.weather_code);

    let directions = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    let dir_idx = ((current.wind_direction / 45.0).round() as usize) % 8;
    let wind_compass = directions[dir_idx];

    match mode.to_lowercase().as_str() {
        "compact" => {
            let art = get_ascii_art(current.weather_code);
            println!();
            println!("📍 \x1b[1;36m{}, {}\x1b[0m", location.name, location.country);
            println!("📅 {}", start_date);
            println!();
            println!("  {}   Temp:       \x1b[1;31m{:.1}°C\x1b[0m (Feels like: {:.1}°C)", art[0], current.temperature, current.apparent_temperature);
            println!("  {}   Condition:  {} {}", art[1], emoji, desc);
            println!("  {}   Wind:       {:.1} km/h {}", art[2], current.wind_speed, wind_compass);
            println!("  {}   Humidity:   {:.0}%", art[3], current.humidity);
            println!("  {}   Cloudiness: {:.0}%", art[4], current.cloud_cover);
            println!();
        }

        "inline" => {
            println!(
                "📍 {}: 🌡️ {:.1}°C (Feels {:.1}°C) | {} {} | 💧 {:.0}% | 💨 {:.1} km/h {} | ☁️ {:.0}%",
                location.name, current.temperature, current.apparent_temperature, emoji, desc, current.humidity, current.wind_speed, wind_compass, current.cloud_cover
            );
        }

        "json" => {
            let mut wrapper = HashMap::new();
            wrapper.insert("provider_name", serde_json::to_value(provider_name).unwrap());
            wrapper.insert("hourly", serde_json::to_value(hourly_points).unwrap());
            wrapper.insert("resolved_location", serde_json::to_value(location).unwrap());
            wrapper.insert("location", serde_json::to_value(location).unwrap());
            if is_offline {
                wrapper.insert("offline", serde_json::to_value(true).unwrap());
                if let Some(ts) = offline_time {
                    wrapper.insert("cached_timestamp", serde_json::to_value(ts).unwrap());
                }
            }
            println!("{}", serde_json::to_string_pretty(&wrapper).unwrap());
        }

        "emoji" => {
            println!();
            println!("🌍 \x1b[1mWEATHER DETAILS\x1b[0m 🌍");
            println!("━━━━━━━━━━━━━━━━━━━━━━");
            println!("📍 Location:    {} ({})", location.name, location.country);
            println!("📅 Date:        {}", start_date);
            println!("🌡️ Temp:        {:.1}°C", current.temperature);
            println!("🔥 Feels Like:  {:.1}°C", current.apparent_temperature);
            println!("🌤️ Condition:   {} {}", emoji, desc);
            println!("💦 Humidity:    {:.0}%", current.humidity);
            println!("💨 Wind:        {:.1} km/h direction {}° ({})", current.wind_speed, current.wind_direction, wind_compass);
            println!("☁️ Cloudiness:  {:.0}%", current.cloud_cover);
            if let Some(uv) = current.uv_index {
                println!("☀️ UV Index:    {:.1}", uv);
            }
            if let Some(vis) = current.visibility {
                println!("👁️ Visibility:  {:.1} km", vis);
            }
            if let Some(st) = current.soil_temperature {
                println!("🌱 Soil Temp:   {:.1}°C", st);
            }
            println!("━━━━━━━━━━━━━━━━━━━━━━");
        }

        "ascii-banner" => {
            let temp_str = format!("{:.0}°C", current.temperature);
            println!();
            println!("📍 {}, {}", location.name, location.country);
            println!("Condition: {} {}", emoji, desc);
            println!();
            for line in 0..5 {
                let mut line_str = String::new();
                for c in temp_str.chars() {
                    line_str.push_str(get_large_char(c, line));
                }
                println!("{}", line_str);
            }
            println!();
        }

        "sparkline" => {
            let count = std::cmp::min(24, hourly_points.len());
            let day_temps: Vec<f64> = hourly_points.iter().take(count).map(|p| p.temperature).collect();
            let spark = get_sparkline(&day_temps);
            let min = day_temps.iter().copied().fold(f64::INFINITY, f64::min);
            let max = day_temps.iter().copied().fold(f64::NEG_INFINITY, f64::max);

            println!();
            println!("📍 {}, {}", location.name, location.country);
            println!("24h Temperature Trend:");
            println!("  {}", spark);
            println!("  [Min: {:.1}°C, Max: {:.1}°C]", min, max);
            println!();
        }

        "bordered-card" => {
            let header = format!("║  Weather Card: {}, {}  ║", location.name, location.country);
            let width = header.chars().count() - 2;
            let top_border = format!("╔{}╗", "═".repeat(width));
            let bot_border = format!("╚{}╝", "═".repeat(width));
            let divider = format!("╠{}╣", "═".repeat(width));

            println!();
            println!("{}", top_border);
            println!("{}", header);
            println!("{}", divider);
            println!("║  Date:      {:width$}  ║", start_date, width = width - 12);
            println!("║  Temp:      {:.1}°C ({:.1}°C) {:width$}  ║", current.temperature, current.apparent_temperature, "", width = width - 25);
            println!("║  Condition: {} {} {:width$}  ║", emoji, desc, "", width = width - 15 - desc.len());
            println!("║  Wind:      {:.1} km/h {} {:width$}  ║", current.wind_speed, wind_compass, "", width = width - 18 - wind_compass.len());
            println!("║  Humidity:  {:.0}% {:width$}  ║", current.humidity, "", width = width - 15);
            println!("{}", bot_border);
            println!();
        }

        "markdown" => {
            println!();
            println!("# Weather Report for {}, {}", location.name, location.country);
            println!("- **Date**: {}", start_date);
            println!("- **Temperature**: {:.1}°C (Feels like: {:.1}°C)", current.temperature, current.apparent_temperature);
            println!("- **Condition**: {} {}", emoji, desc);
            println!("- **Wind**: {:.1} km/h direction {} ({})", current.wind_speed, current.wind_direction, wind_compass);
            println!("- **Humidity**: {:.0}%", current.humidity);
            println!("- **Cloud Cover**: {:.0}%", current.cloud_cover);
            if let Some(uv) = current.uv_index {
                println!("- **UV Index**: {:.1}", uv);
            }
            if let Some(vis) = current.visibility {
                println!("- **Visibility**: {:.1} km", vis);
            }
            println!();
        }

        "html-preview" => {
            let html_content = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Weather Dashboard - {name}</title>
    <style>
        body {{
            background: linear-gradient(135deg, #0f172a, #1e1b4b, #311042);
            color: #f8fafc;
            font-family: 'Outfit', sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            overflow: hidden;
        }}
        .dashboard {{
            background: rgba(255, 255, 255, 0.05);
            backdrop-filter: blur(20px);
            -webkit-backdrop-filter: blur(20px);
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 24px;
            padding: 40px;
            width: 450px;
            box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
            text-align: center;
            animation: fadeIn 1s ease-out;
        }}
        @keyframes fadeIn {{
            from {{ opacity: 0; transform: translateY(20px); }}
            to {{ opacity: 1; transform: translateY(0); }}
        }}
        h1 {{
            font-size: 2.2rem;
            margin: 0 0 5px 0;
            font-weight: 700;
            letter-spacing: -0.5px;
        }}
        .country {{
            color: #a78bfa;
            font-size: 1.1rem;
            text-transform: uppercase;
            letter-spacing: 2px;
            margin-bottom: 25px;
        }}
        .main-weather {{
            margin: 30px 0;
        }}
        .temp-val {{
            font-size: 6rem;
            font-weight: 800;
            line-height: 1;
            background: linear-gradient(to right, #ffffff, #e2e8f0);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }}
        .weather-desc {{
            font-size: 1.5rem;
            margin-top: 10px;
            color: #cbd5e1;
            font-weight: 500;
        }}
        .emoji-big {{
            font-size: 4rem;
            margin-bottom: 10px;
        }}
        .grid {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin-top: 30px;
        }}
        .metric-card {{
            background: rgba(255, 255, 255, 0.03);
            border: 1px solid rgba(255, 255, 255, 0.05);
            padding: 15px;
            border-radius: 16px;
            transition: all 0.3s ease;
        }}
        .metric-card:hover {{
            background: rgba(255, 255, 255, 0.08);
            transform: translateY(-5px);
        }}
        .metric-label {{
            font-size: 0.85rem;
            color: #94a3b8;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 5px;
        }}
        .metric-value {{
            font-size: 1.25rem;
            font-weight: 600;
            color: #f1f5f9;
        }}
    </style>
</head>
<body>
    <div class="dashboard">
        <h1>{name}</h1>
        <div class="country">{country}</div>
        <div class="main-weather">
            <div class="emoji-big">{emoji}</div>
            <div class="temp-val">{temp:.1}°C</div>
            <div class="weather-desc">{desc}</div>
        </div>
        <div class="grid">
            <div class="metric-card">
                <div class="metric-label">Apparent Temp</div>
                <div class="metric-value">{app:.1}°C</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Wind Speed</div>
                <div class="metric-value">{wind:.1} km/h ({wind_compass})</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Humidity</div>
                <div class="metric-value">{humidity:.0}%</div>
            </div>
            <div class="metric-card">
                <div class="metric-label">Cloud Cover</div>
                <div class="metric-value">{clouds:.0}%</div>
            </div>
        </div>
    </div>
</body>
</html>"#,
                name = location.name,
                country = location.country,
                emoji = emoji,
                temp = current.temperature,
                desc = desc,
                app = current.apparent_temperature,
                wind = current.wind_speed,
                wind_compass = wind_compass,
                humidity = current.humidity,
                clouds = current.cloud_cover
            );

            // Locate cache directory to save HTML
            let mut path = if let Ok(home) = std::env::var("HOME") {
                PathBuf::from(home)
            } else {
                PathBuf::from(".")
            };
            path.push(".cache");
            path.push("weather_oz");
            let _ = std::fs::create_dir_all(&path);
            path.push("preview.html");

            if std::fs::write(&path, html_content).is_ok() {
                println!();
                println!("🌐 \x1b[1;32mHTML Dashboard saved successfully!\x1b[0m");
                println!("   Path: \x1b[4mfile://{}\x1b[0m", path.to_string_lossy());
                println!();
                // Trigger auto-open using macOS open command
                let _ = std::process::Command::new("open")
                    .arg(path.to_string_lossy().to_string())
                    .spawn();
            } else {
                println!("Failed to write HTML preview file.");
            }
        }

        _ => {
            // Default comfy-table layout
            print_location_info(location);
            print_date_range_info(start_date, end_date, data_type);
            print_hourly_table(hourly_points, 24);
        }
    }
}

fn get_sparkline(temps: &[f64]) -> String {
    if temps.is_empty() {
        return String::new();
    }
    let min = temps.iter().copied().fold(f64::INFINITY, f64::min);
    let max = temps.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = max - min;
    let sparks = [' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let mut out = String::new();
    for &t in temps {
        let idx = if range == 0.0 {
            0
        } else {
            let val = ((t - min) / range * 7.0).round() as usize;
            std::cmp::min(val, 7)
        };
        out.push(sparks[idx]);
    }
    out
}

pub fn print_error_block(error_msg: &str) {
    let mut suggestion = "Please double check your command arguments or run 'weather --help' for details.".to_string();
    
    // Automatically match common errors to provide exceptionally helpful recommendations
    let lower = error_msg.to_lowercase();
    if lower.contains("date must be in yyyy-mm-dd format") || lower.contains("date") {
        suggestion = "Use the correct ISO format. Example: '--from-date 2026-05-21'".to_string();
    } else if lower.contains("otomatik tespit edilemedi") || lower.contains("location query could not be resolved") || lower.contains("cihaz konumu") || lower.contains("no geocoding results") || lower.contains("girilmedi") {
        suggestion = "The automatic geocoding system could not identify your location. Provide a city name directly. Example: 'weather Istanbul'".to_string();
    } else if lower.contains("offline") || lower.contains("dns error") || lower.contains("connect") || lower.contains("timeout") || lower.contains("network") {
        suggestion = "An internet connection error occurred. Check your network or run without network-dependent flags.".to_string();
    } else if lower.contains("api key") || lower.contains("unauthorized") {
        suggestion = "An API key issue was encountered. Verify your configuration in '~/.config/weather_oz/config.json' or set the appropriate environment variables.".to_string();
    }

    let err_line = format!("🚨 ERROR: {}", error_msg);
    let sugg_line = format!("💡 SUGGESTION: {}", suggestion);

    // Dynamic width calculation based on character length
    let max_len = std::cmp::max(err_line.chars().count(), sugg_line.chars().count());
    let border_width = std::cmp::max(max_len, 60);

    let horizontal_border = "═".repeat(border_width + 2);
    
    println!();
    println!("\x1b[1;31m╔{}╗\x1b[0m", horizontal_border);
    println!("\x1b[1;31m║\x1b[0m  \x1b[1;37m{:<width$}\x1b[0m  \x1b[1;31m║\x1b[0m", err_line, width = border_width);
    println!("\x1b[1;31m╠{}╣\x1b[0m", horizontal_border);
    println!("\x1b[1;31m║\x1b[0m  \x1b[1;32m{:<width$}\x1b[0m  \x1b[1;31m║\x1b[0m", sugg_line, width = border_width);
    println!("\x1b[1;31m╚{}╝\x1b[0m", horizontal_border);
    println!();
}

