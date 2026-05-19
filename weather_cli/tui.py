"""TUI Dashboard module for displaying weather CLI processes interactively."""

from rich.layout import Layout
from rich.panel import Panel
from rich.text import Text
from rich.table import Table
from rich.console import Group

class ProcessState:
    """Shared state object for tracking the progress of the weather fetching process."""
    def __init__(self, query: str):
        self.query = query
        self.resolved_location = None
        self.cache_status = "searching"  # "hit", "miss", "searching"
        self.global_progress = 0
        
        # Steps status: "pending", "running", "completed", "failed"
        self.step_parsing = "completed"
        self.step_geocoding = "pending"
        self.step_race = "pending"
        self.step_blending = "pending"
        
        # Providers status: name -> {"status": "pending"|"running"|"completed"|"failed", "time": float|None, "error": str|None}
        self.providers = {
            "Open-Meteo": {"status": "pending", "time": None, "error": None},
            "MET Norway": {"status": "pending", "time": None, "error": None},
            "wttr.in": {"status": "pending", "time": None, "error": None},
        }
        self.last_log = "Süreç başlatılıyor..."

def build_dashboard_layout(state: ProcessState) -> Layout:
    """Builds a beautiful rich.Layout using the current ProcessState."""
    layout = Layout()
    layout.split_column(
        Layout(name="header", size=3),
        Layout(name="body", size=8),
        Layout(name="footer", size=3)
    )
    
    # 1. Header with styling
    layout["header"].update(
        Panel(
            Text("☁️  WEATHER RUNNER v0.1.0 — Canlı Süreç Konsolu", style="bold bright_cyan", justify="center"),
            border_style="cyan"
        )
    )
    
    # 2. Body split
    layout["body"].split_row(
        Layout(name="steps", ratio=1),
        Layout(name="race", ratio=1)
    )
    
    # Left Side: Checklist & Details
    steps_group = []
    
    def get_step_text(name: str, status: str) -> Text:
        if status == "completed":
            return Text(f"  ✅ {name}", style="green")
        elif status == "running":
            return Text(f"  ⠋ {name}", style="bold yellow")
        elif status == "failed":
            return Text(f"  ❌ {name}", style="bold red")
        else:
            return Text(f"  🕒 {name}", style="dim")
            
    steps_group.append(get_step_text("Argümanlar Ayrıştırıldı", state.step_parsing))
    steps_group.append(get_step_text("Konum Çözümleniyor", state.step_geocoding))
    steps_group.append(get_step_text("API Sağlayıcı Yarışı", state.step_race))
    steps_group.append(get_step_text("Veri Harmanlama & Analiz", state.step_blending))
    
    # Beautiful manual progress bar
    filled = int(state.global_progress / 5)
    bar = "▰" * filled + "▱" * (20 - filled)
    
    bar_style = "bold blue"
    if state.global_progress >= 90:
        bar_style = "bold green"
    elif state.global_progress >= 50:
        bar_style = "bold yellow"
        
    progress_text = Text(f"\n📊 İlerleme: {bar} {state.global_progress}%\n", style=bar_style)
    
    loc_text = Text()
    if state.resolved_location:
        loc_text = Text(f"📍 Konum: {state.resolved_location}", style="bold green")
    else:
        loc_text = Text(f"📍 Arama: \"{state.query}\"", style="bold yellow")
        
    layout["steps"].update(
        Panel(
            Group(loc_text, progress_text, *steps_group),
            title="📋 İşlem Adımları",
            border_style="cyan"
        )
    )
    
    # Right Side: Provider Race Table
    table = Table(show_header=True, header_style="bold magenta", box=None, expand=True)
    table.add_column("Sağlayıcı", justify="left")
    table.add_column("Durum", justify="center")
    table.add_column("Süre", justify="right")
    
    for prov_name, prov_info in state.providers.items():
        status_str = prov_info["status"]
        time_ms = prov_info["time"]
        
        status_text = Text()
        time_text = Text()
        
        if status_str == "completed":
            status_text = Text("🏆 Başarılı", style="bold green")
            time_text = Text(f"{time_ms:.1f}ms" if time_ms else "-", style="green")
        elif status_str == "running":
            status_text = Text("🏎️ Yarışıyor...", style="bold yellow")
            time_text = Text("-", style="dim")
        elif status_str == "failed":
            status_text = Text("❌ Başarısız", style="bold red")
            time_text = Text(f"{time_ms:.1f}ms" if time_ms else "-", style="red")
        else:
            status_text = Text("🕒 Beklemede", style="dim")
            time_text = Text("-", style="dim")
            
        table.add_row(prov_name, status_text, time_text)
        
    layout["race"].update(
        Panel(
            table,
            title="🏁 API Sağlayıcı Yarışı",
            border_style="cyan"
        )
    )
    
    # 3. Footer with System Log
    log_style = "dim white"
    if "hata" in state.last_log.lower() or "failed" in state.last_log.lower() or "limit" in state.last_log.lower():
        log_style = "bold red"
    elif "kazandı" in state.last_log.lower() or "başarıyla" in state.last_log.lower() or "hit" in state.last_log.lower():
        log_style = "bold green"
        
    layout["footer"].update(
        Panel(
            Text(state.last_log, style=log_style, overflow="ellipsis"),
            title="⚡ Son Aktivite",
            border_style="cyan"
        )
    )
    
    return layout
