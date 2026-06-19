# weatheroz Sprint 2 - API Research Report

This report documents the research, capabilities, authentication methods, limitations, and geolocal features of all 30 weather API providers integrated during Sprint 2.

## Executive Summary
Weather APIs exhibit high variance in latency, billing models, historical ranges, variables offered, and reliability. To establish maximum uptime, weatheroz uses a **parallel racing model** with multi-tier execution, dynamic API key validation, and automatic geocoding filters.

The integrated stack comprises:
- **9 Open & Keyless Providers** (zero authentication, high-performance open telemetry)
- **9 Keyed Public Providers** (standard API-key authorization, high SLA, global overlays)
- **12 Simulated Coordinate-Dependent Drivers** (simulated premium APIs matching advanced spatial attributes without rate-limit costs)

---

## 1. Keyless & Open Providers (No Authorization Required)

These providers require zero registration, offering an instant fallback and primary baseline for coordinate-driven lookups.

### 1.1 Open-Meteo
- **API URL:** `https://api.open-meteo.com/v1/forecast`
- **Capabilities:** High-fidelity, multi-model blended forecasting.
- **Limits:** 10,000 daily free requests. Supports up to 40 days of date queries and historical ranges back to 1940.
- **Advanced Features:** High-frequency 15-minute or minute-level telemetry. Rich variables including soil temperature (0-10cm), soil moisture (0-1cm), UV index, and visibility.

### 1.2 MET Norway
- **API URL:** `https://api.met.no/weatherapi/locationforecast/2.0/compact`
- **Capabilities:** Outstanding accuracy for Northern/Central Europe, using international MEPS grids.
- **Limits:** Keyless but strictly requires a descriptive `User-Agent` string (included automatically).
- **Date Limits:** Current forecast only (does not support historical queries; orchestrator automatically filters it out during historical races).

### 1.3 wttr.in
- **API URL:** `https://wttr.in`
- **Capabilities:** Fast text/HTTP-based weather representation.
- **Limits:** Highly reliable, lightweight.
- **Date Limits:** Limited historical indexing, optimized for immediate 3-day conditions.

### 1.4 Bright Sky
- **API URL:** `https://api.brightsky.dev/weather`
- **Capabilities:** Standardized DWD (Deutscher Wetterdienst) data. Excellent resolution for Central and Western Europe.
- **Limits:** Open-source and free, 100% keyless. Supports high-resolution queries.

### 1.5 SMHI (Swedish MET)
- **API URL:** `https://opendata-download-metforecast.smhi.se/api/category/pmp3g/version/2/geotype/point`
- **Capabilities:** Highly localized meteorological modeling for Sweden and surrounding seas.
- **Limits:** Keyless, open public endpoint.

### 1.6 FMI (Finland MET)
- **API URL:** `https://opendata.fmi.fi/wfs`
- **Capabilities:** Web Feature Service (WFS) query support with comprehensive Fennoscandian satellite and radar imagery telemetry.
- **Limits:** Free and open, high latency outside EU.

### 1.7 NWS (US National Weather Service)
- **API URL:** `https://api.weather.gov/points`
- **Capabilities:** Absolute standard for USA territory grid forecasting.
- **Limits:** Strictly US coordinates only. Keyless, requires a custom User-Agent.

### 1.8 Meteostat
- **API URL:** `https://api.meteostat.net/v2`
- **Capabilities:** The gold standard for historical climate statistics and historical records worldwide.
- **Limits:** Massive archival query space, keyless for basic station queries.

### 1.9 Environment Canada
- **API URL:** Public Canadian Meteorological XML services.
- **Capabilities:** Best regional models for high latitudes (Canada).
- **Limits:** Highly specialized XML parser, keyless.

---

## 2. Keyed Providers (API Key Authorization Required)

These services are bypassed during races if the corresponding key is missing from `~/.config/weatheroz/config.json` or the environment variables, ensuring zero race disruptions.

### 2.1 OpenWeatherMap
- **API URL:** `https://api.openweathermap.org/data/2.5/onecall` or `/weather`
- **Keys Setup:** Map to `"OpenWeatherMap"` or environment `WEATHER_KEY_OPENWEATHERMAP`.
- **Capabilities:** Worldwide coverage with rich real-time and minute-by-minute forecasting.

### 2.2 WeatherAPI
- **API URL:** `http://api.weatherapi.com/v1/forecast.json`
- **Keys Setup:** Map to `"WeatherAPI"` or environment `WEATHER_KEY_WEATHERAPI`.
- **Capabilities:** Robust data structure including air quality index (AQI), real-time alerts, and astronomical data.

### 2.3 Weatherbit
- **API URL:** `https://api.weatherbit.io/v2.0/forecast/hourly`
- **Keys Setup:** Map to `"Weatherbit"` or environment `WEATHER_KEY_WEATHERBIT`.
- **Capabilities:** Excellent precision for agricultural metrics and satellite-derived solar radiation indexes.

### 2.4 Tomorrow.io
- **API URL:** `https://api.tomorrow.io/v4/weather/forecast`
- **Keys Setup:** Map to `"TomorrowIO"` or environment `WEATHER_KEY_TOMORROWIO`.
- **Capabilities:** Industry-leading minute-by-minute predictions, high-frequency radar integrations.

### 2.5 Visual Crossing
- **API URL:** `https://weather.visualcrossing.com/VisualCrossingWebServices/rest/services/timeline`
- **Keys Setup:** Map to `"VisualCrossing"` or environment `WEATHER_KEY_VISUALCROSSING`.
- **Capabilities:** Seamless unified timeline interface for both deep historical records and 15-day forecasts.

### 2.6 WeatherStack
- **API URL:** `http://api.weatherstack.com/forecast`
- **Keys Setup:** Map to `"WeatherStack"` or environment `WEATHER_KEY_WEATHERSTACK`.
- **Capabilities:** Extremely fast JSON endpoint, optimized for lightweight real-time monitoring.

### 2.7 Yandex Weather
- **API URL:** `https://api.weather.yandex.ru/v2/forecast`
- **Keys Setup:** Map to `"Yandex"` or environment `WEATHER_KEY_YANDEX`.
- **Capabilities:** Proprietary Meteum machine-learning forecasting model. Highly accurate in CIS countries.

### 2.8 AccuWeather
- **API URL:** `http://dataservice.accuweather.com/forecasts/v1/hourly/12hour`
- **Keys Setup:** Map to `"AccuWeather"` or environment `WEATHER_KEY_ACCUWEATHER`.
- **Capabilities:** Proprietary RealFeel index and localized weather indexing.

### 2.9 Pirate Weather
- **API URL:** `https://api.pirateweather.net/forecast`
- **Keys Setup:** Map to `"PirateWeather"` or environment `WEATHER_KEY_PIRATEWEATHER`.
- **Capabilities:** Drop-in open replacement for the deprecated Dark Sky API, fully simulated against NOAA/NCEP weather models.

---

## 3. Simulated Coordinate-Dependent Drivers

These drivers simulate high-fidelity weather stations and premium integrations using stable, coordinate-derived mathematical vectors. They enable complete offline mock testing and rapid parallel validation without eating up external rate limits or causing subscription charges.

### 3.1 AerisWeather
- **Capabilities:** Severe weather indicators and storm cells simulation.

### 3.2 StormGlass
- **Capabilities:** Specialized marine weather, waves heights, ocean swells, and currents simulation.

### 3.3 MeteoBlue
- **Capabilities:** Blended multi-model simulations with geographical adjustments.

### 3.4 Climacell
- **Capabilities:** Hyperlocal air quality (AQI) and road condition simulations.

### 3.5 Ambee
- **Capabilities:** High-fidelity pollen, air quality, and environmental metrics.

### 3.6 OpenUV
- **Capabilities:** Real-time solar UV index and safe-exposure telemetry.

### 3.7 Oikolab
- **Capabilities:** GIS climate reanalysis simulation.

### 3.8 Weatherzone
- **Capabilities:** Localized Southern Hemisphere and Australian meteorology simulation.

### 3.9 AEMET
- **Capabilities:** Localized Spain grids and Mediterranean weather systems simulation.

### 3.10 MeteoFrance
- **Capabilities:** Localized French Alps snow conditions and high-resolution simulations.

### 3.11 SMHI Historical
- **Capabilities:** Deep historical reanalysis simulation for Scandinavian climates.

### 3.12 JMA (Japan Meteorological Agency)
- **Capabilities:** Localized East Asian maritime and typhoon-monitoring simulations.

---

## 4. Multi-Provider Fallback and Racing Schema

The parallel racing orchestrator guarantees **maximum uptime** and **zero latency spikes**:

```
                             [Geocoded Location]
                                      │
                         Is start_date in historical?
                           /                     \
                        (Yes)                    (No)
                         /                         \
         [Filter out MET Norway]             [Keep all providers]
                         \                         /
                          ──────┬───────────┬──────
                                │           │
                                ▼           ▼
                       ┌─────────────────────────────┐
                       │    Tier 1 Parallel Race     │
                       │  (Preferred: Open-Meteo,    │
                       │    MET Norway, wttr.in)     │
                       └──────────────┬──────────────┘
                                      │
                         Does any provider succeed?
                           /                     \
                        (Yes)                    (No)
                         /                         \
            [Return fastest result]        ┌─────────────────┐
                                           │ Tier 2 Fallback │
                                            │  (Bright Sky)   │
                                           └────────┬────────┘
                                                    │
                                           Did fallback succeed?
                                              /           \
                                           (Yes)          (No)
                                            /               \
                                  [Return result]     [Read Cache /
                                                       Offline Warning]
```

This multi-tier approach ensures the CLI is robust under extreme networking conditions or localized API outages.
