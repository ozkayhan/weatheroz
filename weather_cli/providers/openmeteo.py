import json
import urllib.error
import urllib.request
from datetime import date, timedelta
from typing import Any
from weather_cli.providers.base import BaseWeatherProvider
from weather_cli.providers.models import HourlyPoint, NormalizedWeatherData

FORECAST_BASE = "https://api.open-meteo.com/v1/forecast"
ARCHIVE_BASE = "https://archive-api.open-meteo.com/v1/archive"

HOURLY_VARS = (
    "temperature_2m,apparent_temperature,precipitation_probability,"
    "precipitation,relative_humidity_2m,wind_speed_10m,wind_direction_10m,"
    "cloud_cover,weather_code"
)


class OpenMeteoProvider(BaseWeatherProvider):
    @property
    def name(self) -> str:
        return "Open-Meteo"

    @property
    def provider_name(self) -> str:
        return "Open-Meteo"

    def _build_url(self, base: str, lat: float, lon: float, start: str, end: str) -> str:
        return (
            f"{base}?latitude={lat}&longitude={lon}"
            f"&start_date={start}&end_date={end}"
            f"&hourly={HOURLY_VARS}"
            f"&timezone=auto"
        )

    def _fetch_url(self, url: str) -> dict:
        try:
            with urllib.request.urlopen(url) as resp:
                return json.loads(resp.read().decode())
        except urllib.error.URLError as e:
            raise ConnectionError(f"Failed to reach Open-Meteo weather service: {e}") from e

    def _merge_responses(self, archive: dict, forecast: dict) -> dict:
        merged: dict[str, Any] = {}
        all_keys = set(archive.keys()) | set(forecast.keys())
        for key in all_keys:
            if key == "hourly":
                merged[key] = {}
                archive_section = archive.get(key, {})
                forecast_section = forecast.get(key, {})
                all_fields = set(archive_section.keys()) | set(forecast_section.keys())
                for field in all_fields:
                    a_vals = archive_section.get(field, [])
                    f_vals = forecast_section.get(field, [])
                    merged[key][field] = list(a_vals) + list(f_vals)
            else:
                merged[key] = archive.get(key) if key in archive else forecast.get(key)
        return merged

    def fetch(self, lat: float, lon: float, start_date: str, end_date: str) -> NormalizedWeatherData:
        today = date.today()
        s = date.fromisoformat(start_date)
        e = date.fromisoformat(end_date)
        forecast_limit = today + timedelta(days=16)

        if e < today:
            url = self._build_url(ARCHIVE_BASE, lat, lon, start_date, end_date)
            raw_data = self._fetch_url(url)
        elif s >= today and e <= forecast_limit:
            url = self._build_url(FORECAST_BASE, lat, lon, start_date, end_date)
            raw_data = self._fetch_url(url)
        else:
            archive_end = (today - timedelta(days=1)).isoformat()
            forecast_start = today.isoformat()
            archive_url = self._build_url(ARCHIVE_BASE, lat, lon, start_date, archive_end)
            forecast_url = self._build_url(FORECAST_BASE, lat, lon, forecast_start, end_date)
            archive_resp = self._fetch_url(archive_url)
            forecast_resp = self._fetch_url(forecast_url)
            raw_data = self._merge_responses(archive_resp, forecast_resp)

        return self._normalize(raw_data)

    def _normalize(self, raw: dict) -> NormalizedWeatherData:
        hourly_raw = raw.get("hourly", {})
        times = hourly_raw.get("time", [])
        temps = hourly_raw.get("temperature_2m", [])
        apparent_temps = hourly_raw.get("apparent_temperature", [])
        precip_probs = hourly_raw.get("precipitation_probability", [])
        precipitations = hourly_raw.get("precipitation", [])
        humidities = hourly_raw.get("relative_humidity_2m", [])
        wind_speeds = hourly_raw.get("wind_speed_10m", [])
        wind_dirs = hourly_raw.get("wind_direction_10m", [])
        clouds = hourly_raw.get("cloud_cover", [])
        codes = hourly_raw.get("weather_code", [])

        points = []
        for i in range(len(times)):
            point = HourlyPoint(
                time=times[i],
                temperature=float(temps[i]) if i < len(temps) and temps[i] is not None else 0.0,
                apparent_temperature=float(apparent_temps[i]) if i < len(apparent_temps) and apparent_temps[i] is not None else 0.0,
                precipitation_probability=float(precip_probs[i]) if i < len(precip_probs) and precip_probs[i] is not None else 0.0,
                precipitation=float(precipitations[i]) if i < len(precipitations) and precipitations[i] is not None else 0.0,
                humidity=float(humidities[i]) if i < len(humidities) and humidities[i] is not None else 0.0,
                wind_speed=float(wind_speeds[i]) if i < len(wind_speeds) and wind_speeds[i] is not None else 0.0,
                wind_direction=float(wind_dirs[i]) if i < len(wind_dirs) and wind_dirs[i] is not None else 0.0,
                cloud_cover=float(clouds[i]) if i < len(clouds) and clouds[i] is not None else 0.0,
                weather_code=int(codes[i]) if i < len(codes) and codes[i] is not None else 0,
            )
            points.append(point)

        return NormalizedWeatherData(
            provider_name=self.provider_name,
            hourly=points
        )
