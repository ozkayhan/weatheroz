import json
from unittest import mock
import pytest
from weather_cli.providers.models import HourlyPoint, NormalizedWeatherData
from weather_cli.providers.openmeteo import OpenMeteoProvider
from weather_cli.providers.metnorway import MetNorwayProvider
from weather_cli.providers.wttr import WttrProvider

# Mock data for Open-Meteo
OPENMETEO_MOCK_RESPONSE = {
    "latitude": 41.0082,
    "longitude": 28.9784,
    "hourly": {
        "time": ["2026-05-19T00:00", "2026-05-19T01:00"],
        "temperature_2m": [15.2, 14.8],
        "apparent_temperature": [14.0, 13.5],
        "precipitation_probability": [10, 20],
        "precipitation": [0.0, 0.2],
        "relative_humidity_2m": [70, 75],
        "wind_speed_10m": [12.0, 11.5],
        "wind_direction_10m": [45, 50],
        "cloud_cover": [20, 40],
        "weather_code": [0, 1]
    }
}

# Mock data for MET Norway Locationforecast
METNORWAY_MOCK_RESPONSE = {
    "properties": {
        "timeseries": [
            {
                "time": "2026-05-19T00:00:00Z",
                "data": {
                    "instant": {
                        "details": {
                            "air_temperature": 15.2,
                            "relative_humidity": 70.0,
                            "wind_speed": 3.33, # 3.33 m/s = ~12.0 km/h (conversion: speed * 3.6)
                            "wind_from_direction": 45.0,
                            "cloud_area_fraction": 20.0
                        }
                    },
                    "next_1_hours": {
                        "summary": {
                            "symbol_code": "clearsky_day"
                        },
                        "details": {
                            "precipitation_amount": 0.0,
                            "probability_of_precipitation": 10.0
                        }
                    }
                }
            },
            {
                "time": "2026-05-19T01:00:00Z",
                "data": {
                    "instant": {
                        "details": {
                            "air_temperature": 14.8,
                            "relative_humidity": 75.0,
                            "wind_speed": 3.19, # ~11.5 km/h
                            "wind_from_direction": 50.0,
                            "cloud_area_fraction": 40.0
                        }
                    },
                    "next_1_hours": {
                        "summary": {
                            "symbol_code": "fair_day"
                        },
                        "details": {
                            "precipitation_amount": 0.2,
                            "probability_of_precipitation": 20.0
                        }
                    }
                }
            }
        ]
    }
}

# Mock data for wttr.in
WTTR_MOCK_RESPONSE = {
    "weather": [
        {
            "date": "2026-05-19",
            "hourly": [
                {
                    "time": "0",
                    "tempC": "15",
                    "FeelsLikeC": "14",
                    "chanceofrain": "10",
                    "precipMM": "0.0",
                    "humidity": "70",
                    "windspeedKmph": "12",
                    "winddirDegree": "45",
                    "cloudcover": "20",
                    "weatherCode": "113" # WWO 113 Sunny -> WMO 0
                },
                {
                    "time": "300",
                    "tempC": "14",
                    "FeelsLikeC": "13",
                    "chanceofrain": "20",
                    "precipMM": "0.2",
                    "humidity": "75",
                    "windspeedKmph": "11",
                    "winddirDegree": "50",
                    "cloudcover": "40",
                    "weatherCode": "116" # WWO 116 Partly Cloudy -> WMO 2
                }
            ]
        }
    ]
}

def test_openmeteo_provider():
    provider = OpenMeteoProvider()
    with mock.patch("urllib.request.urlopen") as mock_url:
        mock_resp = mock.MagicMock()
        mock_resp.read.return_value = json.dumps(OPENMETEO_MOCK_RESPONSE).encode("utf-8")
        mock_resp.__enter__.return_value = mock_resp
        mock_url.return_value = mock_resp
        
        data = provider.fetch(41.0082, 28.9784, "2026-05-19", "2026-05-19")
        
        assert isinstance(data, NormalizedWeatherData)
        assert data.provider_name == "Open-Meteo"
        assert len(data.hourly) == 2
        assert data.hourly[0].temperature == 15.2
        assert data.hourly[0].apparent_temperature == 14.0
        assert data.hourly[0].precipitation_probability == 10.0
        assert data.hourly[0].precipitation == 0.0
        assert data.hourly[0].humidity == 70.0
        assert data.hourly[0].wind_speed == 12.0
        assert data.hourly[0].wind_direction == 45.0
        assert data.hourly[0].cloud_cover == 20.0
        assert data.hourly[0].weather_code == 0

def test_metnorway_provider():
    provider = MetNorwayProvider()
    with mock.patch("urllib.request.urlopen") as mock_url:
        mock_resp = mock.MagicMock()
        mock_resp.read.return_value = json.dumps(METNORWAY_MOCK_RESPONSE).encode("utf-8")
        mock_resp.__enter__.return_value = mock_resp
        mock_url.return_value = mock_resp
        
        data = provider.fetch(41.0082, 28.9784, "2026-05-19", "2026-05-19")
        
        assert isinstance(data, NormalizedWeatherData)
        assert data.provider_name == "MET Norway"
        assert len(data.hourly) == 2
        
        h1 = data.hourly[0]
        assert h1.temperature == 15.2
        assert h1.apparent_temperature == 15.2 # met.no doesn't have apparent_temp, falls back to temp
        assert h1.precipitation_probability == 10.0
        assert h1.precipitation == 0.0
        assert h1.humidity == 70.0
        assert abs(h1.wind_speed - 11.988) < 0.1 # 3.33 m/s * 3.6 = 11.988 km/h
        assert h1.wind_direction == 45.0
        assert h1.cloud_cover == 20.0
        assert h1.weather_code == 0 # mapped from clearsky_day

def test_wttr_provider():
    provider = WttrProvider()
    with mock.patch("urllib.request.urlopen") as mock_url:
        mock_resp = mock.MagicMock()
        mock_resp.read.return_value = json.dumps(WTTR_MOCK_RESPONSE).encode("utf-8")
        mock_resp.__enter__.return_value = mock_resp
        mock_url.return_value = mock_resp
        
        data = provider.fetch(41.0082, 28.9784, "2026-05-19", "2026-05-19")
        
        assert isinstance(data, NormalizedWeatherData)
        assert data.provider_name == "wttr.in"
        # Since wttr returns 2 points, they should be mapped correctly
        assert len(data.hourly) > 0
        h1 = data.hourly[0]
        assert h1.temperature == 15.0
        assert h1.apparent_temperature == 14.0
        assert h1.precipitation_probability == 10.0
        assert h1.precipitation == 0.0
        assert h1.humidity == 70.0
        assert h1.wind_speed == 12.0
        assert h1.wind_direction == 45.0
        assert h1.cloud_cover == 20.0
        assert h1.weather_code == 0 # mapped from WWO code 113
