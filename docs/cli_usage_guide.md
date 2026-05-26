# Weatheroz - Usage Guide

A highly polished, ultra-performant command-line weather client featuring parallel multi-provider racing, dynamic caching, consensus-based weather blending, extreme data enrichment, and 10 custom visual output modes.

---

## 🚀 Installation & Build

Build the project using standard cargo toolchain:
```bash
cargo build --release
```
The optimized binary will be generated at `./target/release/weatheroz`.

---

## 🛠️ CLI Flags & Arguments

The Weatheroz exposes robust, intuitive options for managing visual presentation, forecast parameters, cache rules, and coordinates:

| Flag / Option | Description | Defaults |
|---|---|---|
| `<query>` | The name of the city, address, or location query to resolve (geocoded automatically). | *Required if no config default* |
| `-m`, `--mode` | Visual presentation mode: `default`, `compact`, `inline`, `json`, `emoji`, `ascii-banner`, `sparkline`, `bordered-card`, `markdown`, `html-preview`. | `default` |
| `-e`, `--enrich` | Fetches enriched meteorological variables: soil temperature (0-10cm), soil moisture (0-1cm), UV index, visibility, and AQI. | `false` |
| `-d`, `--days` | Custom number of forecast days to fetch (supports ranges up to 40 days). | `7` |
| `--minute` | Request high-frequency, minute-by-minute (or 15-minute resolution) updates. | `false` |
| `--cache-ttl` | Custom cache validation TTL in minutes. Set to `0` to completely bypass cache lookups. | `15` |
| `--verbose` | Output detailed orchestration logging, API latency metrics, and geocoding logs. | `false` |
| `-f`, `--from-date` | Start date for historical or custom weather queries (`YYYY-MM-DD`). | *Today* |
| `-t`, `--to-date` | End date for custom weather queries (`YYYY-MM-DD`). | *Today* |

---

## ⚡ Parallel Weather Racing & Consensus Blending

At the core of the Weatheroz is a state-of-the-art asynchronous engine designed to maximize uptime, performance, and accuracy:

*   **Parallel Multi-Provider Racing:** The client fires asynchronous HTTP requests to all configured weather providers in parallel. As soon as the first successful response is received, the loop finishes to guarantee lightning-fast response times.
*   **100ms Consensus Window:** After the first successful response is received, the client waits for a tiny 100ms grace period to allow other fast providers to complete their fetches. Uncompleted slow or hanging requests are immediately dropped to prevent terminal lag.
*   **Mathematical Data Blending:** Successful responses received within the consensus window are blended using advanced meteorological mapping:
    *   *Numerical Attributes* (Temperature, apparent temperature, humidity, wind speed, wind direction, cloud cover, and rain intensities) are mathematically averaged.
    *   *Condition Codes* are resolved using majority-voting (mode) consensus logic to filter out anomalous provider errors.
*   **API Key Pre-Filtering:** The engine automatically detects which providers require API keys and cross-references them with your configuration. Providers lacking keys are dynamically filtered out *before* starting the race, preventing useless HTTP errors.
*   **Automatic Fallback Groups:** If the entire primary race group fails, the orchestrator instantly falls back to secondary fallback providers.

---

## 🎨 The 10 Visual Output Modes

### 1. `default` (Standard Table)
- **Usage:** `./weatheroz Istanbul` (or `-m default`)
- **Aesthetic:** Clean, standard-width tabular grid listing temperature, wind speed, apparent temperature, humidity, precipitation probability, cloudiness, and condition.

### 2. `compact` (Neofetch Style ASCII Art Card)
- **Usage:** `./weatheroz Istanbul -m compact`
- **Aesthetic:** Features custom-drawn weather glyphs (sun, clouds, rain clouds, storm clouds) placed side-by-side with formatted metadata. Designed for system dashboards or neofetch-style custom terminal splash screens.

### 3. `inline` (Status Bar Integration)
- **Usage:** `./weatheroz Istanbul -m inline`
- **Aesthetic:** A single string line with zero trailing linebreaks, optimized for integration with terminal status bars (e.g., tmux, polybar, i3blocks):
  `📍 Istanbul: 🌡️ 16.3°C (Feels 17.3°C) | ⛈️ Thunderstorm | 💧 96% | 💨 6.8 km/h SW`

### 4. `json` (Automated Schemas)
- **Usage:** `./weatheroz Istanbul -m json`
- **Aesthetic:** Fully structured JSON schema containing all spatial parameters, resolved coordinates, and full hourly arrays. Ideal for piping into jq or python automation scripts.

### 5. `emoji` (Rich Graphic Visualizer)
- **Usage:** `./weatheroz Istanbul -m emoji`
- **Aesthetic:** A highly vibrant, colorful terminal card utilizing custom visual panels to separate and highlight each individual meteorological metric.

### 6. `ascii-banner` (Giant Banner Display)
- **Usage:** `./weatheroz Istanbul -m ascii-banner`
- **Aesthetic:** Renders a giant, custom-padded multi-line ASCII block numbers display representing the resolved temperature, visible from feet away in a workspace terminal.

### 7. `sparkline` (Chronological Trend Indicator)
- **Usage:** `./weatheroz Istanbul -m sparkline`
- **Aesthetic:** Uses fine Unicode 1/8th-height bar characters (` ▂▃▄▅▆▇█`) to draw a 24-hour chronological temperature trend line directly inside the terminal.

### 8. `bordered-card` (Clean UI Container)
- **Usage:** `./weatheroz Istanbul -m bordered-card`
- **Aesthetic:** An elegant, double-lined terminal enclosure boxing the weather forecast details into a structured graphic component.

### 9. `markdown` (Render-Ready Markdown)
- **Usage:** `./weatheroz Istanbul -m markdown`
- **Aesthetic:** Outputs highly semantic GitHub-flavored markdown. Perfect for redirecting into logs, readme files, or piping into static site generators.

### 10. `html-preview` (Interactive Premium Dashboard)
- **Usage:** `./weatheroz Istanbul -m html-preview`
- **Aesthetic:** Generates a stunning glassmorphic web page at `~/.cache/weatheroz/preview.html` and **automatically launches it** in the user's default browser on macOS. Features gradient dark-mode styling, subtle transitions, and premium card layouts.

---

## ⚙️ Configuration File Setup

The Weatheroz looks for a global JSON configuration file at:
`~/.config/weatheroz/config.json`

### Template `config.json`
```json
{
  "api_keys": {
    "OpenWeatherMap": "YOUR_OPENWEATHER_KEY",
    "WeatherAPI": "YOUR_WEATHERAPI_KEY",
    "TomorrowIO": "YOUR_TOMORROWIO_KEY",
    "VisualCrossing": "YOUR_VISUALCROSSING_KEY"
  },
  "race_providers": [
    "Open-Meteo",
    "MET Norway",
    "wttr.in"
  ],
  "fallback_providers": [
    "Bright Sky"
  ],
  "cache_ttl_minutes": 15,
  "forecast_days": 7,
  "minute_updates": false,
  "enrich_data": true
```

### Environment Overrides
To keep API secrets out of config files or to run the Weatheroz inside CI/CD pipelines, you can set the following environment variables:
- `WEATHER_KEY_OPENWEATHERMAP`
- `WEATHER_KEY_WEATHERAPI`
- `WEATHER_KEY_WEATHERBIT`
- `WEATHER_KEY_TOMORROWIO`
- `WEATHER_KEY_VISUALCROSSING`
- `WEATHER_KEY_WEATHERSTACK`
- `WEATHER_KEY_YANDEX`
- `WEATHER_KEY_ACCUWEATHER`
- `WEATHER_KEY_PIRATEWEATHER`
