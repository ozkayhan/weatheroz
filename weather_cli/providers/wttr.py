import json
import urllib.error
import urllib.request
from weather_cli.providers.base import BaseWeatherProvider
from weather_cli.providers.models import HourlyPoint, NormalizedWeatherData

WWO_TO_WMO = {
    113: 0,   # Sunny/Clear
    116: 2,   # Partly Cloudy
    119: 3,   # Cloudy
    122: 3,   # Overcast
    143: 45,  # Mist
    176: 61,  # Patchy rain nearby
    179: 71,  # Patchy snow nearby
    182: 71,  # Patchy sleet nearby
    185: 56,  # Patchy freezing drizzle nearby
    200: 95,  # Thundery outbreaks nearby
    227: 77,  # Blowing snow
    230: 75,  # Blizzard
    248: 45,  # Fog
    260: 48,  # Freezing fog
    263: 51,  # Patchy light drizzle
    266: 51,  # Light drizzle
    281: 56,  # Freezing drizzle
    284: 57,  # Heavy freezing drizzle
    293: 61,  # Patchy light rain
    296: 61,  # Light rain
    299: 63,  # Moderate rain at times
    302: 63,  # Moderate rain
    305: 65,  # Heavy rain at times
    308: 65,  # Heavy rain
    311: 66,  # Light freezing rain
    314: 67,  # Moderate or heavy freezing rain
    317: 71,  # Light sleet
    320: 75,  # Moderate or heavy sleet
    323: 71,  # Patchy light snow
    326: 71,  # Light snow
    329: 73,  # Patchy moderate snow
    332: 73,  # Moderate snow
    335: 75,  # Patchy heavy snow
    338: 75,  # Heavy snow
    350: 77,  # Ice pellets
    353: 80,  # Light rain shower
    356: 81,  # Moderate or heavy rain shower
    359: 82,  # Torrential rain shower
    362: 85,  # Light sleet showers
    365: 86,  # Moderate or heavy sleet showers
    368: 85,  # Light snow showers
    371: 86,  # Moderate or heavy snow showers
    374: 77,  # Light showers of ice pellets
    377: 77,  # Moderate or heavy showers of ice pellets
    386: 95,  # Patchy light rain with thunder
    389: 95,  # Moderate or heavy rain with thunder
    392: 96,  # Patchy light snow with thunder
    395: 99,  # Moderate or heavy snow with thunder
}


def _wwo_code_to_wmo(wwo_code: int) -> int:
    return WWO_TO_WMO.get(wwo_code, 0)


class WttrProvider(BaseWeatherProvider):
    @property
    def name(self) -> str:
        return "wttr.in"

    @property
    def provider_name(self) -> str:
        return "wttr.in"

    def fetch(self, lat: float, lon: float, start_date: str, end_date: str) -> NormalizedWeatherData:
        # wttr.in format=j1 URL format
        url = f"https://wttr.in/{lat},{lon}?format=j1"
        
        try:
            with urllib.request.urlopen(url) as resp:
                raw_data = json.loads(resp.read().decode())
        except urllib.error.URLError as e:
            raise ConnectionError(f"Failed to reach wttr.in weather service: {e}") from e

        return self._normalize(raw_data, start_date, end_date)

    def _normalize(self, raw: dict, start_date: str, end_date: str) -> NormalizedWeatherData:
        weather_list = raw.get("weather", [])
        
        points = []
        for day in weather_list:
            day_date = day.get("date", "")
            # Filter by date range (inclusive)
            if not day_date or not (start_date <= day_date <= end_date):
                continue
                
            hourly_blocks = day.get("hourly", [])
            if not hourly_blocks:
                continue
                
            # Expand 3-hourly blocks to 24-hourly points (00:00 to 23:00)
            for h in range(24):
                block_idx = min(h // 3, len(hourly_blocks) - 1)
                block = hourly_blocks[block_idx]
                
                temp = block.get("tempC", "0")
                feels_like = block.get("FeelsLikeC", temp)
                precip_prob = block.get("chanceofrain", "0")
                precip = block.get("precipMM", "0.0")
                humidity = block.get("humidity", "0")
                wind_speed = block.get("windspeedKmph", "0")
                wind_dir = block.get("winddirDegree", "0")
                cloud_cover = block.get("cloudcover", "0")
                wwo_code = int(block.get("weatherCode", "113"))
                
                point = HourlyPoint(
                    time=f"{day_date}T{h:02d}:00",
                    temperature=float(temp),
                    apparent_temperature=float(feels_like),
                    precipitation_probability=float(precip_prob),
                    precipitation=float(precip),
                    humidity=float(humidity),
                    wind_speed=float(wind_speed),
                    wind_direction=float(wind_dir),
                    cloud_cover=float(cloud_cover),
                    weather_code=_wwo_code_to_wmo(wwo_code),
                )
                points.append(point)
                
        return NormalizedWeatherData(
            provider_name=self.provider_name,
            hourly=points
        )
