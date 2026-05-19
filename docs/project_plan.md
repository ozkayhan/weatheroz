# Weather CLI: v0.0.1 -> v0.1 Teknik Gereksinimler ve Proje Planı

Bu doküman, **Weather CLI** projesini açık kaynak dünyasındaki en kararlı, en hızlı ve "AI-Native" uyumlu hava durumu aracı haline getirmek üzere kararlaştırdığımız tüm **Teknik Gereksinimleri** ve **Proje Planını** içermektedir.

---

## 1. Temel Vizyon & Mimari Yaklaşım

Projenin v0.1 sürümündeki ana odağı; harici API anahtarlarına (API Key) bağımlılığı sıfıra indirmek, ağ gecikmelerini paralel işleme (race) ve yerel önbellek mekanizmalarıyla aşmak ve son derece temiz, amaca yönelik bir çıktı sunmaktır.

```mermaid
graph TD
    User[Kullanıcı / AI Ajanı] --> CLI[cli.py]
    CLI --> GeoCache{geo_cache.json}
    
    GeoCache -- Cache Hit 0ms --> Orch[orchestrator.py]
    GeoCache -- Cache Miss --> GeoAPI[Geocoding API] --> Orch
    
    Orch -->|Truncate Lat/Lon to 4 Decimals| Pool[ThreadPoolExecutor]
    
    subgraph Parallel Race (First-One-Wins)
        Pool -->|Thread 1| OM[Open-Meteo Provider]
        Pool -->|Thread 2| MN[MET Norway Provider]
        Pool -->|Thread 3| WT[wttr.in Provider]
    end
    
    OM --> Standardizer[Data Standardizer]
    MN --> Standardizer
    WT --> Standardizer
    
    Standardizer --> Output[output.py]
```

---

## 2. Detaylı Teknik Gereksinimler (Technical Requirements)

### A. API Anahtarsız (Keyless) Sağlayıcı Zinciri
CLI aracının kurulum gerektirmeden çalışabilmesi için yalnızca API key istemeyen, cömert limitli global servisler kullanılacaktır:
1.  **Open-Meteo:** Ana sağlayıcı (Forecast & Archive/Historical verilerini destekler).
2.  **MET Norway (`api.met.no`):** 1. Yedek sağlayıcı (Yalnızca Forecast destekler. Özel `User-Agent: weather-cli/0.1.0 contact@example.com` başlığı ile sorgulanır).
3.  **wttr.in (`format=j1`):** 2. Yedek sağlayıcı (Forecast & Archive/Historical verilerini destekler, OpenStreetMap tabanlıdır).

### B. Paralel Yarış Orkestratörü (Parallel Race Orchestrator)
Ağ gecikmelerini sıfıra indirmek ve sağlayıcı çökmelerinden etkilenmemek için **"Happy Eyeballs" / Eşzamanlı Yarış** mimarisi uygulanacaktır:
*   **Paralel Tetikleme:** Sorgular `concurrent.futures.ThreadPoolExecutor` aracılığıyla tüm uygun sağlayıcılara eşzamanlı (aynı anda) gönderilecektir.
*   **İlk Gelen Kazanır (Race):** Sağlayıcılardan başarıyla normalize edilmiş JSON dönen **ilk sonuç** anında kabul edilerek ekrana basılacak; diğer yavaş veya başarısız thread'ler sessizce iptal edilecektir.
*   **Tarih Kontrolü (Historical Guard):**
    *   Eğer talep edilen tarih aralığı *geçmiş (historical)* verileri içeriyorsa, MET Norway bu veriyi desteklemediği için yarış dışı bırakılacak; sadece Open-Meteo ve wttr.in yarıştırılacaktır.
    *   Tahmin (forecast) isteklerinde 3 sağlayıcı birden yarışacaktır.

### C. Akıllı Konum Önbelleği (Geocoding JSON Cache)
*   Yerel diskte `~/.cache/weather_cli/geo_cache.json` dosyasında konum çözme sonuçları saklanacaktır.
*   **Akış:** Kullanıcı "istanbul" yazdığında önce bu dosyaya bakılacak; eğer kayıt varsa **0 ms** içinde koordinatlar alınacaktır. Yoksa Geocoding API'sine gidilip gelen sonuç önbelleğe yazılacaktır.
*   **Geçerlilik Süresi (TTL):** Konum koordinatları değişmeyeceği için önbellek süresi **30 gün** olarak belirlenmiştir.

### D. Hassas Koordinat Yuvarlama (Strict Coordinate Truncation)
*   Geocoding API'lerinden dönen hassas koordinatlar (örn: `41.0082384, 28.9783582`), sağlayıcılara gönderilmeden önce **virgülden sonra kesinlikle 4 basamağa** yuvarlanacaktır (`41.0082, 28.9784`).
*   **Gerekçe:** 11 metrelik hassasiyet hava durumu için mükemmeldir. Bu sayede hem kullanıcı gizliliği korunur hem de sağlayıcıların sunucu tarafındaki CDN/Önbellek (Cache Hit) oranı maksimize edilerek yanıtlar çok daha hızlı alınır.

### E. Saatlik Tahmin Odaklılığı & Sadeleştirilmiş Çıktı
*   **Günlük özet (Daily Summary) tablosu tamamen kaldırılmıştır.**
*   CLI, saat başı (hourly) verileri içeren **tek bir sadeleştirilmiş tablo** basacaktır.
*   Tabloda gösterilecek ve ortak şemaya (`NormalizedWeatherData`) dahil edilecek alanlar:
    1.  **Zaman (Time):** HH:MM veya gün geçişlerinde DD HH:MM formatında.
    2.  **Sıcaklık (Temp, °C):** Ondalık yuvarlanmış değer.
    3.  **Hissedilen Sıcaklık (Feels Like, °C):** Rüzgar ve neme bağlı hissedilen sıcaklık.
    4.  **Yağış Olasılığı (Precip Prob, %):** Yağış ihtimali.
    5.  **Yağış Miktarı (Precip Amount, mm):** Beklenen yağış.
    6.  **Nem (Humidity, %):** Bağıl nem oranı.
    7.  **Rüzgar (Wind):** Hız ve yönün birleşimi (Örn: `15 km/h NE`).
    8.  **Bulutluluk Oranı (Cloud, %):** Bulut kaplılık yüzdesi.
    9.  **Hava Durumu (Weather):** Eşlenmiş standart WMO kodu üzerinden Emoji + Kısa Açıklama (Örn: `🌧️ Heavy Rain`).

### F. CLI Çıktı Modları (Output Modes)
*   **Normal Mod (Standart):** Çalışma esnasında hiçbir arka plan logu, spinner veya checklist gösterilmez. Doğrudan çözümlenen konum bilgisi ve ardından nihai saatlik hava durumu tablosu basılır.
*   **Verbose Modu (`--verbose` / `-v`):**
    *   Önbellekten okuma bilgileri (`⚡ Cache Hit: istanbul`).
    *   Hangi sağlayıcıların yarışa sokulduğu (`🚀 Starting parallel race for Open-Meteo, MET Norway, wttr.in`).
    *   Thread durumları, bitiş süreleri ve yarışı kazanan sağlayıcı (`🏆 Winner: MET Norway (210ms)`).
    *   Başarısız olan thread'lerin hata detayları.
*   **JSON Modu (`--json-output` / `-j`):** Renklendirmeler ve tablolar tamamen atlanarak, standart şemadaki saatlik veriler saf JSON olarak `stdout`'a basılır. AI ajanları için kusursuz parse imkanı sunar.

---

## 3. Proje Planı ve İş Paketleri (Project Plan)

Projenin v0.1 sürümüne geçişi, her adımın doğrulanabilir olduğu **5 ana iş paketi** halinde yürütülecektir:

### 📦 İş Paketi 1: Altyapı ve Veri Şeması (`providers/base.py` & cache)
*   [ ] Ortak veri modeli sınıflarının (`NormalizedWeatherData`, `HourlyPoint`) `dataclass` olarak tanımlanması.
*   [ ] Soyut `BaseWeatherProvider` arayüzünün oluşturulması.
*   [ ] Yerel disk tabanlı konum önbellek (`geo_cache.json`) mekanizmasının `geocoding.py` içine entegre edilmesi.
*   [ ] Koordinat yuvarlama (truncation) fonksiyonunun yazılması.

### 📦 İş Paketi 2: Sağlayıcı Modüllerinin Kodlanması (`providers/`)
*   [ ] **OpenMeteoProvider:** Mevcut urllib istek mantığının base sınıfa göre yeniden yazılması ve WMO kodlarının aktarılması.
*   [ ] **MetNorwayProvider:** `api.met.no` GeoJSON compact formatının çekilmesi ve standardizasyonu. `symbol_code` değerlerinin WMO kodlarına eşlenmesi.
*   [ ] **WttrProvider:** `wttr.in?format=j1` JSON formatının çekilmesi ve standardizasyonu. WWO kodlarının WMO kodlarına eşlenmesi.

### 📦 İş Paketi 3: Paralel Yarış Orkestratörü (`providers/orchestrator.py`)
*   [ ] `ThreadPoolExecutor` kullanılarak paralel istek mantığının kurulması.
*   [ ] İlk gelen başarılı yanıtı alma ve diğerlerini sonlandırma mantığının (`as_completed`) kodlanması.
*   [ ] Tarih aralığı kontrolüyle MET Norway'i geçmiş zaman sorgularında yarış dışı bırakma filtresinin yazılması.

### 📦 İş Paketi 4: Çıktı ve CLI Entegrasyonu (`cli.py` & `output.py`)
*   [ ] `cli.py` dosyasına `--verbose / -v` parametresinin eklenmesi.
*   [ ] `output.py` dosyasındaki tabloların yeni basitleştirilmiş kolon yapısına (9 kolon) göre güncellenmesi.
*   [ ] Günlük özet (daily summary) tablolarının tamamen koddan kaldırılması.
*   [ ] Normal ve Verbose modlarındaki loglama akışlarının ayrıştırılması.

### 📦 İş Paketi 5: Doğrulama ve Testler
*   [ ] CLI komutlarının normal modda sorunsuz çalıştığının doğrulanması.
*   [ ] `--verbose` bayrağı ile paralel yarışın ve önbellek mekanizmasının canlı izlenmesi.
*   [ ] `--json-output` ile AI-Native uyumluluğunun kontrol edilmesi.
