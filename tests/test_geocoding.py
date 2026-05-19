import json
import time
from unittest import mock
import pytest
from weather_cli.geocoding import resolve_location, _normalize_query

# We will patch the cache path to a temporary one for testing
@pytest.fixture
def temp_cache(tmp_path):
    cache_file = tmp_path / "geo_cache.json"
    with mock.patch("weather_cli.geocoding.CACHE_PATH", str(cache_file)):
        yield cache_file

def test_coordinate_truncation(temp_cache):
    # Mocking _fetch_geocoding to return highly precise coordinates
    mock_result = [{
        "name": "Istanbul",
        "latitude": 41.0082384,
        "longitude": 28.9783582,
        "country": "Turkey",
        "admin1": "Istanbul"
    }]
    
    with mock.patch("weather_cli.geocoding._fetch_geocoding", return_value=mock_result) as mock_fetch:
        resolved = resolve_location("istanbul")
        
        # Verify coordinates are rounded to 4 decimals
        assert resolved["latitude"] == 41.0082
        assert resolved["longitude"] == 28.9784
        mock_fetch.assert_called_once()

def test_geocoding_cache_hit_and_ttl(temp_cache):
    # Setup initial cache content
    cached_data = {
        "istanbul": {
            "data": {
                "name": "Istanbul",
                "latitude": 41.0082,
                "longitude": 28.9784,
                "country": "Turkey",
                "admin1": "Istanbul"
            },
            "timestamp": time.time()
        }
    }
    with open(temp_cache, "w") as f:
        json.dump(cached_data, f)

    # Calling resolve_location should hit the cache and NOT call the external geocoding API
    with mock.patch("weather_cli.geocoding._fetch_geocoding") as mock_fetch:
        resolved = resolve_location("istanbul")
        assert resolved["latitude"] == 41.0082
        assert resolved["longitude"] == 28.9784
        mock_fetch.assert_not_called()

def test_geocoding_cache_miss_ttl_expired(temp_cache):
    # Setup expired cache content (e.g., 31 days ago)
    expired_time = time.time() - (31 * 24 * 60 * 60)
    cached_data = {
        "istanbul": {
            "data": {
                "name": "Istanbul",
                "latitude": 41.0082,
                "longitude": 28.9784,
                "country": "Turkey",
                "admin1": "Istanbul"
            },
            "timestamp": expired_time
        }
    }
    with open(temp_cache, "w") as f:
        json.dump(cached_data, f)

    # Geocoding API mock for when it falls back
    mock_result = [{
        "name": "Istanbul",
        "latitude": 41.0082384,
        "longitude": 28.9783582,
        "country": "Turkey",
        "admin1": "Istanbul"
    }]

    # Calling resolve_location should miss the cache due to TTL and call _fetch_geocoding
    with mock.patch("weather_cli.geocoding._fetch_geocoding", return_value=mock_result) as mock_fetch:
        resolved = resolve_location("istanbul")
        assert resolved["latitude"] == 41.0082
        assert resolved["longitude"] == 28.9784
        mock_fetch.assert_called()
