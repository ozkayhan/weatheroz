from abc import ABC, abstractmethod
from weather_cli.providers.models import NormalizedWeatherData

class BaseWeatherProvider(ABC):
    @property
    @abstractmethod
    def name(self) -> str:
        """Name of the weather provider."""
        pass

    @abstractmethod
    def fetch(self, lat: float, lon: float, start_date: str, end_date: str) -> NormalizedWeatherData:
        """Fetch and normalize weather data for a given location and date range.

        Args:
            lat: Latitude (rounded to 4 decimal places)
            lon: Longitude (rounded to 4 decimal places)
            start_date: Start date (YYYY-MM-DD)
            end_date: End date (YYYY-MM-DD)

        Returns:
            NormalizedWeatherData dataclass instance.
        """
        pass
