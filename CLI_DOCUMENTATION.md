# Weather CLI — Complete Reference Documentation

> **Version:** 1.0.0  
> **Python:** ≥ 3.9  
> **Command:** `weather`  
> **APIs:** Open-Meteo (Forecast + Archive + Geocoding)

---

## Table of Contents

1. [Overview](#1-overview)
2. [Installation](#2-installation)
3. [Quick Start](#3-quick-start)
4. [Command Reference](#4-command-reference)
   - [Positional Argument](#41-positional-argument-location)
   - [Options](#42-options)
   - [Option Summary Table](#43-option-summary-table)
5. [Location Resolution Logic](#5-location-resolution-logic)
   - [Query Normalization](#51-query-normalization)
   - [Candidate Generation](#52-candidate-generation)
   - [Fallback Chain](#53-fallback-chain)
   - [Geocoding API Details](#54-geocoding-api-details)
6. [Weather Data Fetching Logic](#6-weather-data-fetching-logic)
   - [Date Classification](#61-date-classification)
   - [API Selection Rules](#62-api-selection-rules)
   - [Mixed-Mode Merging](#63-mixed-mode-merging)
   - [Forecast Window Limit](#64-forecast-window-limit)
   - [Requested Variables](#65-requested-variables)
7. [Output Formats](#7-output-formats)
   - [Formatted Output (Default)](#71-formatted-output-default)
   - [JSON Output](#72-json-output)
8. [Output Sections](#8-output-sections)
   - [Location Header](#81-location-header)
   - [Date Range Banner](#82-date-range-banner)
   - [Daily Summary Table](#83-daily-summary-table)
   - [Hourly Weather Table](#84-hourly-weather-table)
9. [Weather Code Reference (WMO)](#9-weather-code-reference-wmo)
10. [Visual Encoding Rules](#10-visual-encoding-rules)
    - [Temperature Colors](#101-temperature-colors)
    - [Wind Direction Mapping](#102-wind-direction-mapping)
    - [UV Index Colors](#103-uv-index-colors)
    - [Time Formatting](#104-time-formatting)
11. [Error Handling](#11-error-handling)
12. [Exit Codes](#12-exit-codes)
13. [Dependencies](#13-dependencies)
14. [Examples](#14-examples)
    - [Basic Usage](#141-basic-usage)
    - [Single Day Queries](#142-single-day-queries)
    - [Date Range Queries](#143-date-range-queries)
    - [Historical Data](#144-historical-data)
    - [Future Forecasts](#145-future-forecasts)
    - [Cross-Boundary (Mixed) Queries](#146-cross-boundary-mixed-queries)
    - [All Hours Flag](#147-all-hours-flag)
    - [JSON Output](#148-json-output)
    - [Combined Flags](#149-combined-flags)
    - [Location Variations](#1410-location-variations)
    - [Edge Cases & Error Scenarios](#1411-edge-cases--error-scenarios)
    - [Scripting & Automation](#1412-scripting--automation)
15. [API Rate Limits & Constraints](#15-api-rate-limits--constraints)
16. [Architecture Overview](#16-architecture-overview)

---

## 1. Overview

**Weather CLI** is a terminal-based weather tool that fetches and displays weather data for any location worldwide. It uses the **Open-Meteo** API ecosystem — combining **forecast**, **archive (historical)**, and **geocoding** services — to provide:

- **Flexible location input** — city, district, neighborhood, or any place name
- **Historical + forecast data** — seamlessly blends past and future weather
- **Rich terminal output** — color-coded tables with emojis, or raw JSON for scripting
- **Multi-day support** — query any date range within API limits

---

## 2. Installation

### From Source

```bash
# Clone or navigate to the project directory
cd weather

# Install dependencies
pip install -r requirements.txt

# Install the package in development mode
pip install -e .
```

### Direct Dependency Installation

```bash
pip install "click>=8.1.0" "rich>=13.0.0"
```

### Verify Installation

```bash
weather --help
```

---

## 3. Quick Start

```bash
# Get today's weather for Istanbul
weather Istanbul

# Get forecast for London next week
weather London --from-date 2026-05-20 --to-date 2026-05-27

# Get raw JSON for scripting
weather "New York" --json-output
```

---

## 4. Command Reference

### 4.1. Positional Argument: `location`

| Property    | Value                              |
|-------------|------------------------------------|
| **Required** | Yes                                |
| **Type**     | String                             |
| **Format**   | Any place name (city, district, etc.) |
| **Language** | English preferred (geocoding uses `language=en`) |

The location argument is **required** and must be provided as the first argument. It accepts any human-readable place name. The CLI normalizes and resolves it through the geocoding service.

### 4.2. Options

#### `--from-date` / `-f`

| Property    | Value                              |
|-------------|------------------------------------|
| **Type**    | String                             |
| **Format**  | `YYYY-MM-DD` (ISO 8601 date)       |
| **Default** | Today's date                       |
| **Required**| No                                 |

Sets the **start date** for the weather query. If omitted, defaults to today.

> **⚠️ FORMAT WARNING:** The date **must** be in `YYYY-MM-DD` format. Any other format (e.g., `MM/DD/YYYY`, `DD-MM-YYYY`, `2026/05/20`) will cause a validation error and the command will exit with code 1.

**Valid examples:** `2026-05-20`, `2025-12-31`, `2026-01-01`  
**Invalid examples:** `05-20-2026`, `20/05/2026`, `May 20 2026`, `2026.05.20`

#### `--to-date` / `-t`

| Property    | Value                              |
|-------------|------------------------------------|
| **Type**    | String                             |
| **Format**  | `YYYY-MM-DD` (ISO 8601 date)       |
| **Default** | Today's date                       |
| **Required**| No                                 |

Sets the **end date** for the weather query. If omitted, defaults to today.

> **⚠️ FORMAT WARNING:** Same strict `YYYY-MM-DD` requirement as `--from-date`.

> **⚠️ LOGIC WARNING:** If `--to-date` is **before** `--from-date`, the API will return empty data. The CLI does **not** validate this ordering — it passes the dates directly to the API. Always ensure `from_date <= to_date`.

#### `--all-hours` / `-a`

| Property    | Value                              |
|-------------|------------------------------------|
| **Type**    | Boolean flag                       |
| **Default** | `false`                            |
| **Required**| No                                 |

When set, displays **all available hourly rows** instead of the default first 24 hours.

> **ℹ️ NOTE:** Without this flag, the hourly table is capped at **24 rows**. With this flag, up to **168 rows** (7 days × 24 hours) are displayed. The actual maximum depends on the date range queried.

#### `--json-output` / `-j`

| Property    | Value                              |
|-------------|------------------------------------|
| **Type**    | Boolean flag                       |
| **Default** | `false`                            |
| **Required**| No                                 |

Outputs **raw JSON** from the Open-Meteo API instead of formatted Rich tables. Suppresses the location header and date range banner.

> **ℹ️ NOTE:** When this flag is used, **no Rich markup** is applied. The output is pure JSON, suitable for piping to `jq`, saving to files, or consuming in scripts.

### 4.3. Option Summary Table

| Flag          | Short | Type   | Default    | Description                        |
|---------------|-------|--------|------------|------------------------------------|
| `--from-date` | `-f`  | String | Today      | Start date (`YYYY-MM-DD`)          |
| `--to-date`   | `-t`  | String | Today      | End date (`YYYY-MM-DD`)            |
| `--all-hours` | `-a`  | Flag   | `false`    | Show all hourly rows (max 168)     |
| `--json-output`| `-j` | Flag   | `false`    | Output raw JSON instead of tables  |

---

## 5. Location Resolution Logic

### 5.1. Query Normalization

Before sending the location to the geocoding API, the CLI applies several normalization strategies to maximize match probability:

1. **ASCII Folding** — Unicode characters are normalized to ASCII equivalents using `NFKD` decomposition.  
   Example: `İstanbul` → `Istanbul`, `München` → `Munchen`

2. **Prefix Stripping** — Common directional/temporal prefixes are stripped from the beginning of queries:  
   `eski`, `yeni`, `new`, `old`, `upper`, `lower`, `north`, `south`, `east`, `west`  
   Example: `New York` → `York` (as a fallback candidate)

3. **Tokenization** — Comma-separated queries are split into individual tokens, each tried as a separate candidate.  
   Example: `Istanbul, Turkey` → tries `Istanbul`, `Turkey`

### 5.2. Candidate Generation

The normalization process generates an **ordered list of candidate queries**, tried sequentially:

```
Input: "Eski Istanbul, Turkey"

Candidates (in order):
1. "Istanbul, Turkey"     (prefix "Eski" stripped)
2. "Eski Istanbul, Turkey" (original)
3. "Istanbul"              (first token)
4. "Turkey"                (second token)
5. "Istanbul, Turkey"      (ASCII-folded if different)
```

The **first candidate that returns results** from the geocoding API is used. This means more specific queries are tried before generic ones.

### 5.3. Fallback Chain

```
User Input
    │
    ▼
┌─────────────────────┐
│ Normalize Query     │
│ (strip, fold, split)│
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐     No results
│ Try Candidate #1    │────────────┐
└────────┬────────────┘            │
         │ Yes                     ▼
         ▼                 ┌─────────────────────┐
┌─────────────────────┐    │ Try Candidate #2    │
│ Return best result  │    └────────┬────────────┘
│ (first API match)   │             │
└─────────────────────┘             │
                                    │ (repeat for all candidates)
                                    │
                                    ▼
                          ┌─────────────────────┐
                          │ Raise ValueError:   │
                          │ "Location not found"│
                          └─────────────────────┘
```

### 5.4. Geocoding API Details

- **Endpoint:** `https://geocoding-api.open-meteo.com/v1/search`
- **Parameters:**
  - `name` — URL-encoded place name
  - `count=5` — return up to 5 results
  - `language=en` — English results
  - `format=json` — JSON response
- **Selection:** The **first result** (`results[0]`) is always chosen as the best match.
- **Returned fields:** `name`, `latitude`, `longitude`, `country`, `admin1`

> **⚠️ WARNING:** The geocoding API uses `language=en`. Non-English place names may not match as expected. Use English names for best results.

> **⚠️ WARNING:** Only the **first** geocoding result is used. If you query an ambiguous name like "Springfield", you get the first match returned by the API, which may not be the intended location. Use more specific queries (e.g., "Springfield, Illinois") to disambiguate.

---

## 6. Weather Data Fetching Logic

### 6.1. Date Classification

The CLI classifies the requested date range into one of three types:

| Type         | Condition                                    | Label in Output            |
|--------------|----------------------------------------------|----------------------------|
| `historical` | `end_date < today`                           | "Historical Data"          |
| `forecast`   | `start_date >= today`                        | "Forecast"                 |
| `mixed`      | `start_date < today` AND `end_date >= today` | "Historical + Forecast"    |

### 6.2. API Selection Rules

Based on the classification, the CLI selects which Open-Meteo API to call:

```
Is end_date < today?
    │
    ├── YES → Use ARCHIVE API only
    │         (https://archive-api.open-meteo.com/v1/archive)
    │
    └── NO → Is start_date >= today AND end_date <= today+16?
                 │
                 ├── YES → Use FORECAST API only
                 │         (https://api.open-meteo.com/v1/forecast)
                 │
                 └── NO → Use BOTH APIs and MERGE
                          Archive: start_date → today-1
                          Forecast: today → end_date
```

### 6.3. Mixed-Mode Merging

When a query spans both historical and forecast dates, the CLI:

1. Calls the **Archive API** for dates from `start_date` to `today - 1`
2. Calls the **Forecast API** for dates from `today` to `end_date`
3. **Merges** the responses by concatenating time-series arrays:

```python
merged[key][field] = archive_values + forecast_values
```

The merge is performed for both `hourly` and `daily` sections. Top-level metadata (like `latitude`, `longitude`, `timezone`) is taken from whichever response contains it.

> **⚠️ WARNING:** In mixed mode, two separate API calls are made. If either call fails, the entire request fails. There is no partial data fallback.

### 6.4. Forecast Window Limit

The Open-Meteo Forecast API provides a maximum of **16 days** into the future. If `end_date` exceeds `today + 16 days`, the forecast API will return data only up to its maximum range — the CLI does **not** truncate or warn about this.

> **⚠️ WARNING:** Queries with `end_date > today + 16 days` will silently return less data than requested. The CLI does not validate this upper bound.

### 6.5. Requested Variables

#### Hourly Variables (12)

| Variable                  | Unit   | Description                    |
|---------------------------|--------|--------------------------------|
| `temperature_2m`          | °C     | Air temperature at 2m          |
| `relative_humidity_2m`    | %      | Relative humidity at 2m        |
| `dew_point_2m`            | °C     | Dew point at 2m                |
| `apparent_temperature`    | °C     | Feels-like temperature         |
| `precipitation_probability`| %     | Probability of precipitation   |
| `precipitation`           | mm     | Precipitation amount           |
| `weather_code`            | WMO    | Weather condition code         |
| `surface_pressure`        | hPa    | Atmospheric pressure           |
| `cloud_cover`             | %      | Total cloud cover              |
| `wind_speed_10m`          | km/h   | Wind speed at 10m              |
| `wind_direction_10m`      | °      | Wind direction (0-360°)        |
| `wind_gusts_10m`          | km/h   | Wind gusts at 10m              |

#### Daily Variables (11)

| Variable                          | Unit   | Description                    |
|-----------------------------------|--------|--------------------------------|
| `temperature_2m_max`              | °C     | Daily maximum temperature      |
| `temperature_2m_min`              | °C     | Daily minimum temperature      |
| `apparent_temperature_max`        | °C     | Daily max feels-like           |
| `apparent_temperature_min`        | °C     | Daily min feels-like           |
| `precipitation_sum`               | mm     | Daily total precipitation      |
| `precipitation_probability_max`   | %      | Daily max precip probability   |
| `wind_speed_10m_max`              | km/h   | Daily max wind speed           |
| `wind_direction_10m_dominant`     | °      | Daily dominant wind direction  |
| `sunrise`                         | ISO    | Sunrise time                   |
| `sunset`                          | ISO    | Sunset time                    |
| `uv_index_max`                    | index  | Daily max UV index             |

> **⚠️ NOTE:** All temperatures are in **Celsius (°C)**. All wind speeds are in **km/h**. All precipitation is in **millimeters (mm)**. The CLI does **not** support unit conversion.

---

## 7. Output Formats

### 7.1. Formatted Output (Default)

The default output consists of four sections rendered in sequence:

1. **Location Header** — resolved name, country, coordinates
2. **Date Range Banner** — formatted dates + data type label
3. **Daily Summary Table** — one row per day
4. **Hourly Weather Table** — one row per hour (capped at 24 by default)

### 7.2. JSON Output

When `--json-output` / `-j` is used:

- Only the **raw API JSON** is printed
- No location header, no date banner, no tables
- Output is indented with 2 spaces (`indent=2`)
- `markup=False` prevents Rich from interpreting JSON brackets as markup

> **ℹ️ TIP:** Pipe JSON to `jq` for further processing:
> ```bash
> weather London --json-output | jq '.daily.temperature_2m_max'
> ```

---

## 8. Output Sections

### 8.1. Location Header

```
📍 Istanbul, Turkey
   Coordinates: 41.0138°, 28.9497°
```

**Format:** `{name}, {admin1}, {country}` — components are omitted if empty.  
**Coordinates:** Displayed to 4 decimal places.

### 8.2. Date Range Banner

```
📅 May 20 - May 27, 2026 (Forecast)
```

**Single day:** `📅 May 20, 2026 (Forecast)`  
**Multi-day:** `📅 May 20 - May 27, 2026 (Forecast)`  
**Historical:** `📅 January 01 - January 07, 2026 (Historical Data)`  
**Mixed:** `📅 May 15 - May 27, 2026 (Historical + Forecast)`

### 8.3. Daily Summary Table

| Column       | Width | Format                              |
|--------------|-------|-------------------------------------|
| Date         | 12    | `YYYY-MM-DD`                        |
| Min (°C)     | 7     | Rounded integer, **color-coded**    |
| Max (°C)     | 7     | Rounded integer, **color-coded**    |
| Precip (mm)  | 8     | 1 decimal (`0.0` if none)           |
| Max Wind (km/h) | 10 | Rounded integer                     |
| Sunrise      | 8     | `HH:MM`                             |
| Sunset       | 8     | `HH:MM`                             |
| UV Index     | 8     | Integer, **color-coded**            |

### 8.4. Hourly Weather Table

| Column       | Width | Format                              |
|--------------|-------|-------------------------------------|
| Time         | 10    | `HH:MM` or `DD HH:MM` (multi-day)   |
| Temp (°C)    | 6     | Rounded integer, **color-coded**    |
| Feels (°C)   | 7     | Rounded integer                     |
| Precip Prob (%) | 10 | Rounded integer                     |
| Precip (mm)  | 7     | 1 decimal (`-` if 0 or null)        |
| Humidity (%) | 8     | Rounded integer                     |
| Wind (km/h)  | 8     | `{speed} {direction}` (e.g., `15 NE`)|
| Gusts (km/h) | 8     | Rounded integer                     |
| Cloud (%)    | 7     | Rounded integer                     |
| Weather      | 18    | `{emoji} {description}`             |
| Pressure (hPa) | 9   | Rounded integer                     |
| Dew Pt (°C)  | 7     | Rounded integer                     |

> **ℹ️ NOTE:** When the hourly data exceeds the display limit, a footer message appears:
> ```
> Showing first 24 of 168 hours. Use --all-hours to see all.
> ```

---

## 9. Weather Code Reference (WMO)

The following table maps **WMO weather codes** to their emoji and description. These codes are returned by the Open-Meteo API and displayed in the hourly table.

| Code | Emoji | Description              | Category     |
|------|-------|--------------------------|--------------|
| 0    | ☀️    | Clear                    | Clear        |
| 1    | 🌤️    | Mainly clear             | Clear        |
| 2    | ⛅    | Partly cloudy            | Cloudy       |
| 3    | ☁️    | Overcast                 | Cloudy       |
| 45   | 🌫️    | Fog                      | Fog          |
| 48   | 🌫️    | Depositing rime fog      | Fog          |
| 51   | 🌦️    | Light drizzle            | Drizzle      |
| 53   | 🌦️    | Moderate drizzle         | Drizzle      |
| 55   | 🌦️    | Dense drizzle            | Drizzle      |
| 56   | 🌦️    | Light freezing drizzle   | Freezing     |
| 57   | 🌦️    | Dense freezing drizzle   | Freezing     |
| 61   | 🌧️    | Slight rain              | Rain         |
| 63   | 🌧️    | Moderate rain            | Rain         |
| 65   | 🌧️    | Heavy rain               | Rain         |
| 66   | 🌧️    | Light freezing rain      | Freezing     |
| 67   | 🌧️    | Heavy freezing rain      | Freezing     |
| 71   | 🌨️    | Slight snow              | Snow         |
| 73   | 🌨️    | Moderate snow            | Snow         |
| 75   | 🌨️    | Heavy snow               | Snow         |
| 77   | 🌨️    | Snow grains              | Snow         |
| 80   | 🌧️    | Slight rain showers      | Showers      |
| 81   | 🌧️    | Moderate rain showers    | Showers      |
| 82   | 🌧️    | Violent rain showers     | Showers      |
| 85   | 🌨️    | Slight snow showers      | Snow Showers |
| 86   | 🌨️    | Heavy snow showers       | Snow Showers |
| 95   | ⛈️    | Thunderstorm             | Thunderstorm |
| 96   | ⛈️    | Thunderstorm + slight hail | Thunderstorm |
| 99   | ⛈️    | Thunderstorm + heavy hail | Thunderstorm |

> **⚠️ WARNING:** Any weather code **not** in this table will display as `❓ Unknown`. The Open-Meteo API may return codes not listed here in edge cases.

---

## 10. Visual Encoding Rules

### 10.1. Temperature Colors

Temperatures in the hourly table, daily min, and daily max columns are color-coded:

| Condition      | Color  | Example        |
|----------------|--------|----------------|
| `temp ≤ 5°C`   | 🔵 Blue   | `3` (blue)     |
| `5°C < temp ≤ 15°C` | 🟢 Green | `12` (green)   |
| `temp > 15°C`  | 🔴 Red    | `28` (red)     |

> **ℹ️ NOTE:** Temperatures are **rounded to the nearest integer** before display.

### 10.2. Wind Direction Mapping

Wind direction in degrees (0-360°) is converted to compass directions:

| Degrees Range    | Direction |
|------------------|-----------|
| 337.5° - 22.5°   | N         |
| 22.5° - 67.5°    | NE        |
| 67.5° - 112.5°   | E         |
| 112.5° - 157.5°  | SE        |
| 157.5° - 202.5°  | S         |
| 202.5° - 247.5°  | SW        |
| 247.5° - 292.5°  | W         |
| 292.5° - 337.5°  | NW        |

**Formula:** `directions[round(degrees / 45) % 8]`

### 10.3. UV Index Colors

UV Index in the daily summary is color-coded:

| UV Index | Color      | Risk Level    |
|----------|------------|---------------|
| 0-4      | 🟢 Green   | Low           |
| 5-7      | 🟡 Yellow  | Moderate-High |
| 8+       | 🔴 Red     | Very High+    |

### 10.4. Time Formatting

| Context        | Format        | Example       |
|----------------|---------------|---------------|
| Single day     | `HH:MM`       | `14:00`       |
| Multi-day      | `DD HH:MM`    | `21 14:00`    |

> **ℹ️ NOTE:** All times are in **UTC**. The API returns times in UTC (`Z` suffix), and the CLI displays them as-is without timezone conversion.

---

## 11. Error Handling

| Error Type        | Trigger                              | Message Format                        | Exit Code |
|-------------------|--------------------------------------|---------------------------------------|-----------|
| `ValueError`      | Location not found                   | `Error: Location not found: {query}`  | 1         |
| `ValueError`      | Invalid date format                  | `Error: Date must be in YYYY-MM-DD format` | 1    |
| `ConnectionError` | Geocoding API unreachable            | `Error: Failed to reach geocoding service: {detail}` | 1 |
| `ConnectionError` | Weather API unreachable              | `Error: Failed to reach weather service: {detail}` | 1     |
| Generic `Exception` | Any other error                    | `Error: {message}`                    | 1         |

> **⚠️ WARNING:** All errors cause an immediate `SystemExit(1)`. No partial output is displayed on error.

> **⚠️ WARNING:** The CLI does **not** validate date ordering (`from_date <= to_date`). If `from_date > to_date`, the API returns empty data and the tables will be empty — no error is raised.

---

## 12. Exit Codes

| Code | Meaning            |
|------|--------------------|
| 0    | Success            |
| 1    | Any error occurred |

---

## 13. Dependencies

| Package   | Minimum Version | Purpose                    |
|-----------|-----------------|----------------------------|
| `click`   | ≥ 8.1.0         | CLI framework              |
| `rich`    | ≥ 13.0.0        | Rich terminal formatting   |

Both are pure Python — no compiled extensions required.

---

## 14. Examples

### 14.1. Basic Usage

```bash
# Today's weather for a city
weather Istanbul

# Today's weather for a US city
weather "New York"

# Today's weather for a European city
weather London

# Today's weather for an Asian city
weather Tokyo
```

### 14.2. Single Day Queries

```bash
# Explicitly query today (same as no flags)
weather Istanbul --from-date 2026-05-19 --to-date 2026-05-19

# Query a specific past date (historical)
weather Istanbul --from-date 2026-01-01 --to-date 2026-01-01

# Query a specific future date (forecast)
weather Istanbul --from-date 2026-06-15 --to-date 2026-06-15
```

### 14.3. Date Range Queries

```bash
# 7-day forecast starting today
weather Istanbul --from-date 2026-05-19 --to-date 2026-05-25

# 3-day historical window
weather Paris --from-date 2026-03-01 --to-date 2026-03-03

# Full week historical
weather Berlin --from-date 2026-04-01 --to-date 2026-04-07

# Month-long forecast (capped at 16 days by API)
weather Tokyo --from-date 2026-05-19 --to-date 2026-06-19
```

### 14.4. Historical Data

```bash
# Weather on a specific past date
weather "San Francisco" --from-date 2025-12-25 --to-date 2025-12-25

# Historical week
weather Moscow --from-date 2025-11-01 --to-date 2025-11-07

# Historical month (archive API supports years of data)
weather Sydney --from-date 2025-06-01 --to-date 2025-06-30

# Year-ago comparison
weather Istanbul --from-date 2025-05-19 --to-date 2025-05-25
```

> **⚠️ NOTE:** The Archive API supports historical data going back many years. However, very old dates may have limited variable availability.

### 14.5. Future Forecasts

```bash
# Tomorrow's forecast
weather Istanbul --from-date 2026-05-20 --to-date 2026-05-20

# Next week's forecast
weather London --from-date 2026-05-26 --to-date 2026-06-01

# Maximum forecast range (16 days)
weather Paris --from-date 2026-05-19 --to-date 2026-06-04

# Weekend forecast
weather Barcelona --from-date 2026-05-23 --to-date 2026-05-24
```

> **⚠️ WARNING:** The forecast API only provides **16 days** ahead. Queries beyond `today + 16` will return incomplete data silently.

### 14.6. Cross-Boundary (Mixed) Queries

```bash
# Spanning yesterday through tomorrow
weather Istanbul --from-date 2026-05-18 --to-date 2026-05-20

# Past week through next week (archive + forecast merge)
weather London --from-date 2026-05-12 --to-date 2026-05-26

# Starting 3 days ago, ending 3 days from now
weather "New York" --from-date 2026-05-16 --to-date 2026-05-22
```

> **ℹ️ NOTE:** Mixed queries trigger **two API calls** (archive + forecast) and merge the results. The output banner will show `(Historical + Forecast)`.

### 14.7. All Hours Flag

```bash
# Show all 24 hours for today (explicit)
weather Istanbul --all-hours

# Show all hours for a 3-day range (up to 72 rows)
weather London --from-date 2026-05-19 --to-date 2026-05-21 --all-hours

# Show all hours for a week (up to 168 rows)
weather Paris --from-date 2026-05-19 --to-date 2026-05-25 --all-hours

# Default behavior (first 24 hours only)
weather Istanbul
```

> **ℹ️ NOTE:** Without `--all-hours`, the hourly table shows only the **first 24 rows**. A footer message indicates how many more hours are available.

### 14.8. JSON Output

```bash
# Raw JSON for today
weather Istanbul --json-output

# Raw JSON for a date range
weather London --from-date 2026-05-19 --to-date 2026-05-25 --json-output

# JSON piped to jq for specific data
weather Istanbul --json-output | jq '.daily.temperature_2m_max'

# JSON saved to file
weather Paris --json-output > paris_weather.json

# JSON with all hours
weather Tokyo --all-hours --json-output
```

**Sample JSON structure:**
```json
{
  "latitude": 41.0138,
  "longitude": 28.9497,
  "generationtime_ms": 0.5,
  "utc_offset_seconds": 0,
  "timezone": "UTC",
  "hourly": {
    "time": ["2026-05-19T00:00", "2026-05-19T01:00", ...],
    "temperature_2m": [15.2, 14.8, ...],
    "relative_humidity_2m": [72, 75, ...],
    "weather_code": [0, 1, ...],
    ...
  },
  "daily": {
    "time": ["2026-05-19", "2026-05-20", ...],
    "temperature_2m_max": [22.5, 24.1, ...],
    "temperature_2m_min": [12.3, 13.0, ...],
    ...
  }
}
```

### 14.9. Combined Flags

```bash
# Short flags: date range + all hours
weather Istanbul -f 2026-05-19 -t 2026-05-21 -a

# Short flags: JSON + date range
weather London -f 2026-05-20 -t 2026-05-27 -j

# All flags combined
weather Paris -f 2026-05-15 -t 2026-05-25 -a -j

# Long + short mix
weather Tokyo --from-date 2026-06-01 -t 2026-06-07 --all-hours
```

### 14.10. Location Variations

```bash
# Simple city name
weather Istanbul

# City with country disambiguation
weather "Springfield, Illinois"
weather "Cambridge, UK"

# District / neighborhood
weather Kadikoy
weather Brooklyn

# Non-ASCII characters (auto-folded)
weather München
weather Zürich
weather São Paulo

# Comma-separated (tokenized)
weather "Istanbul, Turkey"
weather "Paris, France"

# With directional prefix (prefix stripped as fallback)
weather "New York"        # tries "York" as fallback
weather "Eski Ankara"     # tries "Ankara" as fallback
weather "South London"    # tries "London" as fallback
```

### 14.11. Edge Cases & Error Scenarios

```bash
# INVALID: Wrong date format → Error exit
weather Istanbul --from-date 05/19/2026
# Error: Date must be in YYYY-MM-DD format

# INVALID: Another wrong format → Error exit
weather Istanbul --from-date 2026/05/19
# Error: Date must be in YYYY-MM-DD format

# INVALID: Text date → Error exit
weather Istanbul --from-date "May 19, 2026"
# Error: Date must be in YYYY-MM-DD format

# WARNING: Reversed dates → Empty tables (no error raised)
weather Istanbul --from-date 2026-05-25 --to-date 2026-05-19
# Output: Location shown, but tables are empty

# WARNING: Unknown location → Error exit
weather "Xyzzzzz"
# Error: Location not found: Xyzzzzz

# WARNING: Beyond forecast range → Silent truncation
weather Istanbul --from-date 2026-05-19 --to-date 2026-12-31
# Output: Only data up to ~June 4 returned (16-day limit)

# WARNING: Network failure → Error exit
# (When API is unreachable)
# Error: Failed to reach weather service: <URLError ...>
```

### 14.12. Scripting & Automation

```bash
# Extract max temperatures for a week
weather Istanbul --from-date 2026-05-19 --to-date 2026-05-25 --json-output \
  | jq -r '.daily.time[] as $t | .daily.temperature_2m_max[] as $max | "\($t): \($max)°C"'

# Check if it will rain tomorrow
weather London --from-date 2026-05-20 --to-date 2026-05-20 --json-output \
  | jq '.hourly.precipitation | map(select(. > 0)) | length > 0'

# Get current weather code
weather Istanbul --json-output \
  | jq '.hourly.weather_code[0]'

# Batch query multiple cities (shell loop)
for city in "Istanbul" "London" "Paris" "Tokyo"; do
  echo "=== $city ==="
  weather "$city" --json-output | jq '{city: .latitude, lon: .longitude}'
done

# Save daily summary to CSV
weather Istanbul --from-date 2026-05-19 --to-date 2026-05-25 --json-output \
  | jq -r '
    .daily.time as $times |
    .daily.temperature_2m_max as $maxs |
    .daily.temperature_2m_min as $mins |
    "date,max_temp,min_temp",
    range($times | length) | "\($times[.]),\($maxs[.]),\($mins[.])"
  ' > weather_summary.csv
```

---

## 15. API Rate Limits & Constraints

| API            | URL Base                                      | Constraint                        |
|----------------|-----------------------------------------------|-----------------------------------|
| Forecast       | `https://api.open-meteo.com/v1/forecast`      | Max 16 days into the future       |
| Archive        | `https://archive-api.open-meteo.com/v1/archive`| Historical data (varies by location) |
| Geocoding      | `https://geocoding-api.open-meteo.com/v1/search` | Max 5 results per query         |

> **⚠️ NOTE:** Open-Meteo is free for non-commercial use with reasonable request volumes. The CLI does **not** implement rate limiting or caching. Rapid repeated calls may be throttled by the API.

> **⚠️ NOTE:** All times returned by the APIs are in **UTC**. The CLI does not perform timezone conversion. Sunrise/sunset times and hourly timestamps are displayed in UTC.

---

## 16. Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                        CLI Layer                         │
│                    (weather_cli/cli.py)                   │
│                                                          │
│  weather() — Click command entry point                   │
│  ├── Validates date format (YYYY-MM-DD)                  │
│  ├── Resolves location via geocoding                     │
│  ├── Fetches weather data                                │
│  ├── Classifies data type (historical/forecast/mixed)    │
│  └── Renders output (tables or JSON)                     │
└───────────────────────┬─────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│  Geocoding    │ │    Weather    │ │    Output     │
│ (geocoding.py)│ │ (weather.py)  │ │ (output.py)   │
│               │ │               │ │               │
│ normalize     │ │ fetch_weather │ │ print_location│
│ candidates    │ │               │ │ print_daily   │
│ fetch results │ │ archive/      │ │ print_hourly  │
│ return best   │ │ forecast/     │ │ color coding  │
│               │ │ mixed merge   │ │ weather codes │
└───────────────┘ └───────────────┘ └───────────────┘
```

### Module Responsibilities

| Module          | Responsibility                              |
|-----------------|---------------------------------------------|
| `cli.py`        | Click command definition, argument parsing, orchestration |
| `geocoding.py`  | Query normalization, candidate generation, geocoding API calls |
| `weather.py`    | API URL construction, archive/forecast routing, response merging |
| `output.py`     | Rich table rendering, weather code mapping, visual encoding |

---

*Documentation generated for weather-cli v1.0.0. All behavior described reflects the actual source code implementation.*
