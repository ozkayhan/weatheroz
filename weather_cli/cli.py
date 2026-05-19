"""Click-based CLI interface for weather CLI tool."""

import dataclasses
import json
import os
import time
from datetime import date, datetime

import click
from rich.console import Console

from weather_cli.geocoding import resolve_location
from weather_cli.output import (
    print_date_range_info,
    print_hourly_table,
    print_location_info,
)
from weather_cli.providers import OpenMeteoProvider, MetNorwayProvider, WttrProvider
from weather_cli.providers.orchestrator import run_weather_race


console = Console()


def validate_date(ctx, param, value):
    if value is None:
        return value
    try:
        datetime.strptime(value, "%Y-%m-%d")
        return value
    except ValueError:
        raise click.BadParameter("Date must be in YYYY-MM-DD format")


@click.command()
@click.argument("location", required=True)
@click.option(
    "--from-date",
    "-f",
    type=str,
    default=None,
    callback=validate_date,
    help="Start date in YYYY-MM-DD format (default: today)",
)
@click.option(
    "--to-date",
    "-t",
    type=str,
    default=None,
    callback=validate_date,
    help="End date in YYYY-MM-DD format (default: today)",
)
@click.option(
    "--all-hours",
    "-a",
    is_flag=True,
    default=False,
    help="Show all hours instead of first 24",
)
@click.option(
    "--json-output",
    "-j",
    is_flag=True,
    default=False,
    help="Output raw JSON instead of formatted tables",
)
@click.option(
    "--verbose",
    "-v",
    is_flag=True,
    default=False,
    help="Show detailed performance log, caching, and provider race details",
)
def weather(location, from_date, to_date, all_hours, json_output, verbose):
    """Weather CLI tool - Get weather information for a location.

    LOCATION: City, district, neighborhood, or any place name.
    """
    today_str = date.today().strftime("%Y-%m-%d")
    start_date = from_date or today_str
    end_date = to_date or today_str

    try:
        # 1. Geocoding Caching Log in Verbose Mode
        if verbose:
            from weather_cli.geocoding import CACHE_PATH, CACHE_TTL_SECONDS
            query_key = location.lower().strip()
            cache_hit = False
            if os.path.exists(CACHE_PATH):
                try:
                    with open(CACHE_PATH, "r", encoding="utf-8") as f:
                        cache = json.load(f)
                        if query_key in cache:
                            entry = cache[query_key]
                            if time.time() - entry.get("timestamp", 0) < CACHE_TTL_SECONDS:
                                cache_hit = True
                except Exception:
                    pass
            if cache_hit:
                console.print(f"⚡ [bold green]Cache Hit:[/bold green] {location}")
            else:
                console.print(f"🔍 [bold yellow]Cache Miss:[/bold yellow] {location}")

        resolved = resolve_location(location)

        # 2. Providers list setup
        providers = [
            OpenMeteoProvider(),
            MetNorwayProvider(),
            WttrProvider(),
        ]

        # In verbose mode, check which providers are eligible for the race
        today = date.today()
        s_date = date.fromisoformat(start_date)
        is_historical = s_date < today
        eligible_provider_names = [
            p.name for p in providers if not (is_historical and p.name == "MET Norway")
        ]

        if verbose:
            console.print(
                f"🚀 [bold blue]Starting parallel race for:[/bold blue] {', '.join(eligible_provider_names)}"
            )

        # 3. Parallel Race Execution
        weather_data, stats, winner_name = run_weather_race(
            providers=providers,
            lat=resolved["latitude"],
            lon=resolved["longitude"],
            start_date=start_date,
            end_date=end_date,
        )

        # 4. Verbose race stats logging
        if verbose:
            console.print("\n🏁 [bold]Thread Details:[/bold]")
            for p_name, p_stat in stats.items():
                if p_stat["success"]:
                    console.print(f"  - {p_name}: [green]SUCCESS[/green] ({p_stat['time_ms']:.1f}ms)")
                else:
                    console.print(
                        f"  - {p_name}: [red]FAILED[/red] ({p_stat['time_ms']:.1f}ms) - {p_stat['error']}"
                    )
            winner_time = stats[winner_name]["time_ms"]
            console.print(f"\n🏆 [bold gold1]Winner:[/bold gold1] [bold]{winner_name}[/bold] ({winner_time:.1f}ms)\n")

        # 5. JSON Output Mode
        if json_output:
            # Add resolved location to JSON
            result_dict = dataclasses.asdict(weather_data)
            result_dict["resolved_location"] = resolved
            console.print(json.dumps(result_dict, indent=2), markup=False)
            return

        # 6. Normal Output Mode
        print_location_info(resolved)

        today_obj = date.today()
        s = date.fromisoformat(start_date)
        e = date.fromisoformat(end_date)
        if e < today_obj:
            data_type = "historical"
        elif s >= today_obj:
            data_type = "forecast"
        else:
            data_type = "mixed"

        print_date_range_info(start_date, end_date, data_type)

        max_rows = 168 if all_hours else 24
        print_hourly_table(weather_data.hourly, max_rows=max_rows)

    except (ValueError, ConnectionError) as e:
        console.print(f"[bold red]Error:[/bold red] {str(e)}")
        raise SystemExit(1)
    except Exception as e:
        console.print(f"[bold red]Error:[/bold red] {str(e)}")
        raise SystemExit(1)


def main():
    """Entry point for the CLI."""
    weather()


if __name__ == "__main__":
    main()
