import time
from unittest import mock
import pytest
from datetime import date
from weather_cli.providers.models import NormalizedWeatherData
from weather_cli.providers.orchestrator import run_weather_race
from weather_cli.providers.base import BaseWeatherProvider

# Define dummy providers for testing
class DummyProvider(BaseWeatherProvider):
    def __init__(self, name, delay, should_succeed=True, data=None):
        self._name = name
        self.delay = delay
        self.should_succeed = should_succeed
        self.data = data or NormalizedWeatherData(provider_name=name, hourly=[])

    @property
    def name(self) -> str:
        return self._name

    def fetch(self, lat: float, lon: float, start_date: str, end_date: str) -> NormalizedWeatherData:
        time.sleep(self.delay)
        if not self.should_succeed:
            raise ConnectionError(f"{self.name} failed")
        return self.data

def test_race_fastest_wins():
    provider_fast = DummyProvider("Fast", 0.01)
    provider_slow = DummyProvider("Slow", 0.1)
    
    providers = [provider_fast, provider_slow]
    
    winner_data, stats, winner_name = run_weather_race(providers, 41.0082, 28.9784, "2026-05-19", "2026-05-19")
    
    assert winner_name == "Fast"
    assert winner_data.provider_name == "Fast"
    assert stats["Fast"]["success"] is True
    assert stats["Fast"]["time_ms"] > 0

def test_race_handles_failure_falls_back():
    provider_fail = DummyProvider("FailFast", 0.01, should_succeed=False)
    provider_ok = DummyProvider("OkSlow", 0.05)
    
    providers = [provider_fail, provider_ok]
    
    winner_data, stats, winner_name = run_weather_race(providers, 41.0082, 28.9784, "2026-05-19", "2026-05-19")
    
    assert winner_name == "OkSlow"
    assert stats["FailFast"]["success"] is False
    assert stats["FailFast"]["error"] is not None
    assert stats["OkSlow"]["success"] is True

def test_race_historical_guard_excludes_metnorway():
    # Setup openmeteo, wttr and metnorway
    provider_om = DummyProvider("Open-Meteo", 0.01)
    provider_wttr = DummyProvider("wttr.in", 0.01)
    provider_mn = DummyProvider("MET Norway", 0.01)
    
    providers = [provider_om, provider_wttr, provider_mn]
    
    # We will query a past date
    past_date = "2020-01-01"
    
    # Let's mock date.today to always be 2026-05-19 for consistency
    with mock.patch("weather_cli.providers.orchestrator.date") as mock_date:
        mock_date.today.return_value = date(2026, 5, 19)
        mock_date.fromisoformat.side_effect = lambda x: date.fromisoformat(x)
        
        winner_data, stats, winner_name = run_weather_race(providers, 41.0082, 28.9784, past_date, past_date)
        
        # MET Norway should be skipped
        assert "MET Norway" not in stats
        assert len(stats) >= 1
        assert any(k in stats for k in ["Open-Meteo", "wttr.in"])

def test_race_all_fail():
    p1 = DummyProvider("P1", 0.01, should_succeed=False)
    p2 = DummyProvider("P2", 0.01, should_succeed=False)
    
    providers = [p1, p2]
    
    with pytest.raises(ConnectionError) as exc_info:
        run_weather_race(providers, 41.0082, 28.9784, "2026-05-19", "2026-05-19")
        
    assert "All weather providers failed" in str(exc_info.value)
