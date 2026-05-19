import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import date
from typing import List, Tuple, Dict, Any
from weather_cli.providers.base import BaseWeatherProvider
from weather_cli.providers.models import NormalizedWeatherData


def run_weather_race(
    providers: List[BaseWeatherProvider],
    lat: float,
    lon: float,
    start_date: str,
    end_date: str
) -> Tuple[NormalizedWeatherData, Dict[str, Any], str]:
    """Execute concurrent requests to eligible weather providers.
    
    First successful response wins.
    If dates include past dates (historical query), MET Norway is excluded.
    """
    today = date.today()
    s = date.fromisoformat(start_date)
    
    # Historical check
    is_historical = s < today
    
    # Filter eligible providers
    eligible_providers = []
    for p in providers:
        if is_historical and p.name == "MET Norway":
            continue
        eligible_providers.append(p)
        
    if not eligible_providers:
        raise ValueError("No eligible weather providers found for the request.")

    stats: Dict[str, Any] = {}
    winner_data: NormalizedWeatherData = None
    winner_name: str = ""

    # Concurrently execute requests
    with ThreadPoolExecutor(max_workers=len(eligible_providers)) as executor:
        future_to_provider = {}
        for provider in eligible_providers:
            # We wrap the call to capture execution stats
            def run_task(p=provider):
                start_time = time.perf_counter()
                try:
                    res = p.fetch(lat, lon, start_date, end_date)
                    elapsed = (time.perf_counter() - start_time) * 1000.0
                    return p.name, res, elapsed, None
                except Exception as ex:
                    elapsed = (time.perf_counter() - start_time) * 1000.0
                    return p.name, None, elapsed, str(ex)

            future = executor.submit(run_task)
            future_to_provider[future] = provider

        # Process results as they complete
        for future in as_completed(future_to_provider):
            p_name, res, elapsed, error = future.result()
            
            stats[p_name] = {
                "time_ms": elapsed,
                "success": res is not None,
                "error": error
            }
            
            if res is not None and winner_data is None:
                # First one wins! Record winner and return.
                winner_data = res
                winner_name = p_name
                # Note: We can break out of the loop immediately.
                # Remaining threads will finish executing in the background.
                break

    # If we didn't find any successful result
    if winner_data is None:
        # Wait for all remaining futures to get a complete error log if needed,
        # but since as_completed ran through them, let's gather errors:
        errors = [f"{name}: {info['error']}" for name, info in stats.items() if info["error"]]
        error_msg = "All weather providers failed. Errors:\n" + "\n".join(errors)
        raise ConnectionError(error_msg)

    return winner_data, stats, winner_name
