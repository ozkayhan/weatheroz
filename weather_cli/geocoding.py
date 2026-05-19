import json
import os
import time
import unicodedata
import urllib.error
import urllib.parse
import urllib.request

CACHE_PATH = os.path.expanduser("~/.cache/weather_cli/geo_cache.json")
CACHE_TTL_SECONDS = 30 * 24 * 60 * 60  # 30 days

_PREFIXES_TO_STRIP = {"eski", "yeni", "new", "old", "upper", "lower", "north", "south", "east", "west"}


def _ascii_fold(text: str) -> str:
    return unicodedata.normalize("NFKD", text).encode("ascii", "ignore").decode("ascii")


def _normalize_query(query: str) -> list[str]:
    candidates = []
    words = query.split()
    if words and words[0].lower() in _PREFIXES_TO_STRIP:
        stripped = " ".join(words[1:])
        if stripped:
            candidates.append(stripped)
    candidates.append(query)
    parts = [p.strip() for p in query.replace(",", " ").split() if p.strip()]
    for part in parts:
        if part.lower() not in _PREFIXES_TO_STRIP:
            candidates.append(part)
    seen = set()
    deduped = []
    for c in candidates:
        key = c.lower().strip()
        if key not in seen:
            seen.add(key)
            deduped.append(c)
    ascii_candidates = []
    for c in deduped:
        folded = _ascii_fold(c)
        if folded != c and folded:
            ascii_candidates.append(folded)
    return deduped + ascii_candidates


def _fetch_geocoding(query: str) -> list[dict]:
    encoded = urllib.parse.quote(query)
    url = (
        f"https://geocoding-api.open-meteo.com/v1/search"
        f"?name={encoded}&count=5&language=en&format=json"
    )
    try:
        with urllib.request.urlopen(url) as resp:
            data: dict = json.loads(resp.read().decode())
    except urllib.error.URLError as e:
        raise ConnectionError(f"Failed to reach geocoding service: {e}") from e
    return data.get("results", [])


def search_location(query: str) -> list[dict]:
    return _fetch_geocoding(query)


def _load_cache() -> dict:
    if not os.path.exists(CACHE_PATH):
        return {}
    try:
        with open(CACHE_PATH, "r", encoding="utf-8") as f:
            return json.load(f)
    except Exception:
        return {}


def _save_cache(cache: dict) -> None:
    try:
        os.makedirs(os.path.dirname(CACHE_PATH), exist_ok=True)
        with open(CACHE_PATH, "w", encoding="utf-8") as f:
            json.dump(cache, f, indent=2)
    except Exception:
        pass


def resolve_location(query: str, state=None) -> dict:
    import time
    query_key = query.lower().strip()
    
    if state:
        state.step_geocoding = "running"
        state.global_progress = 10
        state.last_log = f"'{query}' için önbellek sorgulanıyor..."
        time.sleep(0.15)

    cache = _load_cache()
    
    if query_key in cache:
        entry = cache[query_key]
        cached_time = entry.get("timestamp", 0)
        if time.time() - cached_time < CACHE_TTL_SECONDS:
            if state:
                state.cache_status = "hit"
                state.resolved_location = f"{entry['data']['name']}, {entry['data'].get('country', '')}"
                state.last_log = f"⚡ Önbellek İsabeti (Cache Hit): '{query}' konum bilgisi yüklendi!"
                state.global_progress = 40
                time.sleep(0.15)
                state.step_geocoding = "completed"
                state.global_progress = 50
                time.sleep(0.1)
            return entry["data"]

    if state:
        state.cache_status = "miss"
        state.last_log = f"🔍 Önbellek Iskaladı (Cache Miss): '{query}' için Geocoding API çağrılıyor..."
        state.global_progress = 25
        time.sleep(0.15)

    candidates = _normalize_query(query)
    for candidate in candidates:
        if state:
            state.last_log = f"📡 Geocoding API sorgulanıyor: '{candidate}'..."
            time.sleep(0.1)
        try:
            results = _fetch_geocoding(candidate)
        except Exception as e:
            if state:
                state.step_geocoding = "failed"
                state.last_log = f"❌ Geocoding Hatası: {str(e)}"
                time.sleep(0.15)
            raise e
            
        if results:
            best = results[0]
            resolved = {
                "name": best["name"],
                "latitude": round(best["latitude"], 4),
                "longitude": round(best["longitude"], 4),
                "country": best.get("country", ""),
                "admin1": best.get("admin1", ""),
            }
            
            # Save to cache
            cache[query_key] = {
                "data": resolved,
                "timestamp": time.time()
            }
            _save_cache(cache)
            
            if state:
                state.resolved_location = f"{resolved['name']}, {resolved.get('country', '')}"
                state.last_log = f"✅ Konum çözümlendi: {state.resolved_location}"
                state.global_progress = 45
                time.sleep(0.15)
                state.step_geocoding = "completed"
                state.global_progress = 50
                time.sleep(0.1)
            
            return resolved
            
    if state:
        state.step_geocoding = "failed"
        state.last_log = f"❌ Konum bulunamadı: '{query}'"
        time.sleep(0.15)
    raise ValueError(f"Location not found: {query}")

