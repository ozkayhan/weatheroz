import json
import urllib.error
import urllib.request
from weather_cli.providers.base import BaseWeatherProvider
from weather_cli.providers.models import HourlyPoint, NormalizedWeatherData

SYMBOL_TO_WMO = {
    "clearsky": 0,
    "fair": 1,
    "partlycloudy": 2,
    "cloudy": 3,
    "fog": 45,
    "lightrainshowers": 80,
    "rainshowers": 81,
    "heavyrainshowers": 82,
    "lightrainshowersandthunder": 95,
    "rainshowersandthunder": 95,
    "heavyrainshowersandthunder": 95,
    "lightsleetshowers": 85,
    "sleetshowers": 86,
    "heavysleetshowers": 86,
    "lightsleetshowersandthunder": 95,
    "sleetshowersandthunder": 95,
    "heavysleetshowersandthunder": 95,
    "lightsnowshowers": 85,
    "snowshowers": 86,
    "heavysnowshowers": 86,
    "lightsnowshowersandthunder": 95,
    "snowshowersandthunder": 95,
    "heavysnowshowersandthunder": 95,
    "lightrain": 61,
    "rain": 63,
    "heavyrain": 65,
    "lightrainandthunder": 95,
    "rainandthunder": 95,
    "heavyrainandthunder": 95,
    "lightsleet": 71,
    "sleet": 73,
    "heavysleet": 75,
    "lightsleetandthunder": 95,
    "sleetandthunder": 95,
    "heavysleetandthunder": 95,
    "lightsnow": 71,
    "snow": 73,
    "heavysnow": 75,
    "lightsnowandthunder": 95,
    "snowandthunder": 95,
    "heavysnowandthunder": 95,
}


def _symbol_code_to_wmo(symbol: str) -> int:
    if not symbol:
        return 0
    base = symbol.split("_")[0].lower()
    return SYMBOL_TO_WMO.get(base, 0)


class MetNorwayProvider(BaseWeatherProvider):
    @property
    def name(self) -> str:
        return "MET Norway"

    @property
    def provider_name(self) -> str:
        return "MET Norway"

    def fetch(self, lat: float, lon: float, start_date: str, end_date: str) -> NormalizedWeatherData:
        # met.no compact URL format
        url = f"https://api.met.no/weatherapi/locationforecast/2.0/compact?lat={lat}&lon={lon}"
        
        req = urllib.request.Request(
            url,
            headers={
                "User-Agent": "weather-cli/0.1.0 contact@example.com"
            }
        )
        
        try:
            with urllib.request.urlopen(req) as resp:
                raw_data = json.loads(resp.read().decode())
        except urllib.error.URLError as e:
            raise ConnectionError(f"Failed to reach MET Norway weather service: {e}") from e

        return self._normalize(raw_data, start_date, end_date)

    def _normalize(self, raw: dict, start_date: str, end_date: str) -> NormalizedWeatherData:
        timeseries = raw.get("properties", {}).get("timeseries", [])
        
        points = []
        for entry in timeseries:
            time_str = entry.get("time", "")
            # Filter by date range (inclusive)
            # time_str is ISO format "YYYY-MM-DDTHH:MM:SSZ"
            if not time_str or not (start_date <= time_str[:10] <= end_date):
                continue
                
            data = entry.get("data", {})
            instant_details = data.get("instant", {}).get("details", {})
            next_1 = data.get("next_1_hours", {})
            next_1_details = next_1.get("details", {})
            next_1_summary = next_1.get("summary", {})
            
            temp = instant_details.get("air_temperature", 0.0)
            humidity = instant_details.get("relative_humidity", 0.0)
            
            # Conversion: MET Norway wind_speed is in m/s, convert to km/h
            wind_speed_ms = instant_details.get("wind_speed", 0.0)
            wind_speed_kmh = wind_speed_ms * 3.6
            
            wind_dir = instant_details.get("wind_from_direction", 0.0)
            cloud_cover = instant_details.get("cloud_area_fraction", 0.0)
            
            precip = next_1_details.get("precipitation_amount", 0.0)
            precip_prob = next_1_details.get("probability_of_precipitation", 0.0)
            
            symbol = next_1_summary.get("symbol_code", "")
            wmo_code = _symbol_code_to_wmo(symbol)
            
            point = HourlyPoint(
                time=time_str.replace("Z", ""), # standardise representation
                temperature=float(temp),
                apparent_temperature=float(temp), # met.no doesn't have apparent temp in compact, fallback to temp
                precipitation_probability=float(precip_prob),
                precipitation=float(precip),
                humidity=float(humidity),
                wind_speed=round(wind_speed_kmh, 2),
                wind_direction=float(wind_dir),
                cloud_cover=float(cloud_cover),
                weather_code=wmo_code,
            )
            points.append(point)
            
        return NormalizedWeatherData(
            provider_name=self.provider_name,
            hourly=points
        )
