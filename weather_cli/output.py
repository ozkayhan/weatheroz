"""Rich terminal output formatting module for weather CLI."""

from datetime import datetime
from typing import List
from rich.console import Console
from rich.table import Table
from rich.text import Text
from weather_cli.providers.models import HourlyPoint

# Weather code to emoji and description mapping
WEATHER_CODES = {
    0: ("☀️", "Clear"),
    1: ("🌤️", "Mainly clear"),
    2: ("⛅", "Partly cloudy"),
    3: ("☁️", "Overcast"),
    45: ("🌫️", "Fog"),
    48: ("🌫️", "Depositing rime fog"),
    51: ("🌦️", "Light drizzle"),
    53: ("🌦️", "Moderate drizzle"),
    55: ("🌦️", "Dense drizzle"),
    56: ("🌦️", "Light freezing drizzle"),
    57: ("🌦️", "Dense freezing drizzle"),
    61: ("🌧️", "Slight rain"),
    63: ("🌧️", "Moderate rain"),
    65: ("🌧️", "Heavy rain"),
    66: ("🌧️", "Light freezing rain"),
    67: ("🌧️", "Heavy freezing rain"),
    71: ("🌨️", "Slight snow"),
    73: ("🌨️", "Moderate snow"),
    75: ("🌨️", "Heavy snow"),
    77: ("🌨️", "Snow grains"),
    80: ("🌧️", "Slight rain showers"),
    81: ("🌧️", "Moderate rain showers"),
    82: ("🌧️", "Violent rain showers"),
    85: ("🌨️", "Slight snow showers"),
    86: ("🌨️", "Heavy snow showers"),
    95: ("⛈️", "Thunderstorm"),
    96: ("⛈️", "Thunderstorm with slight hail"),
    99: ("⛈️", "Thunderstorm with heavy hail"),
}


def _get_weather_info(weather_code: int) -> tuple[str, str]:
    """Get emoji and description for a weather code."""
    return WEATHER_CODES.get(weather_code, ("❓", "Unknown"))


def _get_wind_direction(degrees: float) -> str:
    """Convert wind degrees to compass direction."""
    directions = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"]
    index = round(degrees / 45) % 8
    return directions[index]


def _get_temp_color(temp: float) -> Text:
    """Get colored text for temperature."""
    if temp <= 5:
        return Text(str(round(temp)), style="blue")
    elif temp <= 15:
        return Text(str(round(temp)), style="green")
    else:
        return Text(str(round(temp)), style="red")


def _format_time(time_str: str, is_multi_day: bool = False) -> str:
    """Format time string to HH:MM or DD HH:MM."""
    cleaned = time_str.replace("Z", "")
    if len(cleaned) == 13:  # handle "2026-05-19T14"
        cleaned += ":00"
    dt = datetime.fromisoformat(cleaned)
    if is_multi_day:
        return f"{dt.day:02d} {dt.hour:02d}:{dt.minute:02d}"
    return f"{dt.hour:02d}:{dt.minute:02d}"


def print_location_info(location: dict) -> None:
    """Display resolved location name, country, admin region, coordinates."""
    console = Console()

    name = location.get("name", "Unknown")
    country = location.get("country", "")
    admin1 = location.get("admin1", "")

    location_parts = [name]
    if admin1:
        location_parts.append(admin1)
    if country:
        location_parts.append(country)
    location_str = ", ".join(location_parts)

    lat = location.get("latitude", 0.0)
    lon = location.get("longitude", 0.0)

    console.print()
    console.print(f"📍 [bold cyan]{location_str}[/bold cyan]")
    console.print(f"   Coordinates: {lat:.4f}°, {lon:.4f}°")
    console.print()


def print_date_range_info(start_date: str, end_date: str, data_type: str = "forecast") -> None:
    """Display date range and data type information."""
    console = Console()

    try:
        start_dt = datetime.strptime(start_date, "%Y-%m-%d")
        end_dt = datetime.strptime(end_date, "%Y-%m-%d")
    except ValueError:
        console.print(f"[yellow]Date range: {start_date} to {end_date}[/yellow]")
        return

    if start_dt == end_dt:
        date_str = start_dt.strftime("%B %d, %Y")
    else:
        date_str = f"{start_dt.strftime('%B %d')} - {end_dt.strftime('%B %d, %Y')}"

    if data_type == "historical":
        type_label = "Historical Data"
    elif data_type == "mixed":
        type_label = "Historical + Forecast"
    else:
        type_label = "Forecast"

    console.print(f"📅 [bold]{date_str}[/bold] ({type_label})")
    console.print()


def print_hourly_table(hourly_points: List[HourlyPoint], max_rows: int = 24) -> None:
    """Display beautiful rich.Table showing simplified hourly weather (exactly 9 columns)."""
    console = Console()

    if not hourly_points:
        console.print("[dim]No hourly weather data available.[/dim]")
        return

    # Determine if multi-day
    is_multi_day = False
    if len(hourly_points) > 1:
        try:
            first_date = datetime.fromisoformat(hourly_points[0].time.replace("Z", "")).date()
            last_date = datetime.fromisoformat(hourly_points[-1].time.replace("Z", "")).date()
            is_multi_day = first_date != last_date
        except Exception:
            pass

    display_count = min(max_rows, len(hourly_points))
    has_more = len(hourly_points) > max_rows

    # Create table
    table = Table(
        title="Hourly Weather",
        show_header=True,
        header_style="bold cyan",
        box=None,
    )

    # 9 columns as specified
    table.add_column("Time", justify="center", width=10)
    table.add_column("Temp\n(°C)", justify="right", width=6)
    table.add_column("Feels\n(°C)", justify="right", width=8)
    table.add_column("Precip\nProb (%)", justify="right", width=10)
    table.add_column("Precip\n(mm)", justify="right", width=7)
    table.add_column("Humidity\n(%)", justify="right", width=8)
    table.add_column("Wind", justify="center", width=12)
    table.add_column("Cloud\n(%)", justify="right", width=7)
    table.add_column("Weather", justify="left", width=18)

    for i in range(display_count):
        pt = hourly_points[i]
        
        time_str = _format_time(pt.time, is_multi_day)
        temp_str = _get_temp_color(pt.temperature)
        feel_str = str(round(pt.apparent_temperature))
        pp_str = str(round(pt.precipitation_probability))
        precip_str = f"{pt.precipitation:.1f}" if pt.precipitation > 0.0 else "-"
        hum_str = str(round(pt.humidity))
        
        # Wind: combine speed and direction
        wind_dir_compass = _get_wind_direction(pt.wind_direction)
        wind_str = f"{round(pt.wind_speed)} {wind_dir_compass}"
        
        cloud_str = str(round(pt.cloud_cover))
        
        emoji, desc = _get_weather_info(pt.weather_code)
        weather_str = f"{emoji} {desc}"

        table.add_row(
            time_str,
            temp_str,
            feel_str,
            pp_str,
            precip_str,
            hum_str,
            wind_str,
            cloud_str,
            weather_str,
        )

    console.print(table)

    if has_more:
        console.print()
        console.print(f"[dim]Showing first {max_rows} of {len(hourly_points)} hours. Use --all-hours to see all.[/dim]")