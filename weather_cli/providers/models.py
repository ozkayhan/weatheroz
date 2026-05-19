from dataclasses import dataclass, field
from typing import List

@dataclass
class HourlyPoint:
    time: str                      # ISO format like "2026-05-19T14:00"
    temperature: float             # in °C
    apparent_temperature: float    # in °C (feels like)
    precipitation_probability: float  # in %
    precipitation: float           # in mm
    humidity: float                # in %
    wind_speed: float              # in km/h
    wind_direction: float          # in degrees (0-360)
    cloud_cover: float             # in %
    weather_code: int              # WMO weather code

@dataclass
class NormalizedWeatherData:
    provider_name: str
    hourly: List[HourlyPoint] = field(default_factory=list)
