import json
from unittest import mock
import pytest
from click.testing import CliRunner
from weather_cli.cli import weather

@pytest.fixture
def mock_geocoding_and_orchestrator(monkeypatch):
    monkeypatch.setenv("COLUMNS", "120")
    # Mock resolved location
    resolved = {
        "name": "Istanbul",
        "latitude": 41.0082,
        "longitude": 28.9784,
        "country": "Turkey",
        "admin1": "Istanbul"
    }
    # Mock weather data
    from weather_cli.providers.models import NormalizedWeatherData, HourlyPoint
    hourly = [
        HourlyPoint(
            time="2026-05-19T00:00",
            temperature=15.2,
            apparent_temperature=14.0,
            precipitation_probability=10.0,
            precipitation=0.0,
            humidity=70.0,
            wind_speed=12.0,
            wind_direction=45.0,
            cloud_cover=20.0,
            weather_code=0
        )
    ]
    weather_data = NormalizedWeatherData(provider_name="Open-Meteo", hourly=hourly)
    
    # Mock stats
    stats = {
        "Open-Meteo": {"time_ms": 150.0, "success": True, "error": None},
        "MET Norway": {"time_ms": 200.0, "success": True, "error": None},
        "wttr.in": {"time_ms": 250.0, "success": True, "error": None}
    }
    
    with mock.patch("weather_cli.cli.resolve_location", return_value=resolved) as mock_geo, \
         mock.patch("weather_cli.cli.run_weather_race", return_value=(weather_data, stats, "Open-Meteo")) as mock_race:
        yield mock_geo, mock_race, weather_data

def test_cli_normal_output(mock_geocoding_and_orchestrator):
    runner = CliRunner()
    result = runner.invoke(weather, ["istanbul"])
    
    assert result.exit_code == 0
    # Clean output should contain coordinates and location
    assert "Istanbul, Turkey" in result.output
    assert "41.0082" in result.output
    
    # Should contain hourly weather table headers/content
    assert "Hourly Weather" in result.output
    assert "Temp" in result.output
    assert "Feels" in result.output
    assert "Precip" in result.output
    assert "Weather" in result.output
    
    # Daily Summary should be COMPLETELY removed
    assert "Daily Summary" not in result.output

def test_cli_verbose_output(mock_geocoding_and_orchestrator):
    runner = CliRunner()
    result = runner.invoke(weather, ["istanbul", "--verbose"])
    
    assert result.exit_code == 0
    # Verbose logging should contain parallel race information, winners, and cache hits
    assert "Starting parallel race" in result.output
    assert "Winner: Open-Meteo" in result.output
    assert "Open-Meteo (150.0ms)" in result.output

def test_cli_json_output(mock_geocoding_and_orchestrator):
    runner = CliRunner()
    result = runner.invoke(weather, ["istanbul", "--json-output"])
    
    assert result.exit_code == 0
    
    # Standard JSON check
    parsed = json.loads(result.output)
    assert parsed["provider_name"] == "Open-Meteo"
    assert len(parsed["hourly"]) == 1
    assert parsed["hourly"][0]["temperature"] == 15.2
