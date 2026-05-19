# Yeni Nesil Weather CLI - Sprint & Task Planlaması

Mevcut sistemin (Python tabanlı, 9 kolonlu TUI, geocoding cache'li) pazar analiz raporunda (ANALIZ.MD) belirtilen "Yeni Nesil Açık Kaynak Hava Durumu CLI Aracı" vizyonuna ulaşması için gereken teknik görevler aşağıda "Sprint" yapısında projelendirilmiştir.

Bu görevler, profesyonel bir mühendis tarafından ele alınıp kodlanmaya hazır şekilde tasarlanmıştır.

---

## Task 1: Altyapı ve Çalışma Zamanı Dönüşümü (Python -> Statik Derlenen Dil)
**Hedef:** Uygulamanın çalışma zamanı (runtime) bağımlılıklarını ortadan kaldırmak, `node_modules` veya `pip` zorunluluğunu bitirmek, açılış hızını milisaniye seviyesine indirmek ve statik derlenmiş taşınabilir (portable) binary sunmak.

**Şu Anki Durum:**
Sistem Python (≥ 3.9) kullanılarak yazılmıştır. `ThreadPoolExecutor`, `Click` (CLI parametreleri) ve `Rich` (Terminal TUI) gibi bağımlılıklar içerir. Kurulum ve dağıtım için Python ortamı gereklidir.

**İstenen Durum:**
Sistemin sıfır konfigürasyon ve sıfır dış bağımlılık prensibiyle Go (Golang) veya Rust dillerinden birine geçirilmiş, doğrudan sistemde tek bir binary olarak çalışan, milisaniye açılışlı hali.

**Geçiş İçin Gereksinimler (Kod Yazmadan Yapılacaklar):**
1. Mevcut mimari yapının (Provider Interface, Geocoding Cache, Parallel Orchestrator) seçilen dilin konseptlerine (örn. Go Routines veya Rust Tokio) uyarlanarak eşzamanlılık haritasının çıkarılması.
2. `Click` ve `Rich` kütüphanelerinin dildeki karşılıklarının (örneğin Rust için `clap` ve `ratatui` veya `comfy-table`) seçilmesi ve çıktı sistematiğinin tasarlanması.
3. Statik binary çıktıları üretmek için CI/CD süreçlerinin (Cross-compilation) kurgulanması.

---

## Task 2: IP Tabanlı Sıfır-Config Otomatik Konum Tespiti
**Hedef:** Kullanıcı `weather` komutunu argümansız girdiğinde, cihazın IP adresinden bulunduğu konumu otomatik olarak tespit ederek hava durumunu anında ekrana basmak.

**Şu Anki Durum:**
`cli.py` içerisinde `location` argümanı (örn: `weather Istanbul`) zorunludur. Geocoding sistemi (`geocoding.py`) sadece yazılı metin üzerinden Open-Meteo API ile koordinat bulur. Cihaz konumu algılanmaz.

**İstenen Durum:**
Kullanıcı konum belirtmezse (seyahat halinde vb.), sistem bir GeoIP veya web servisi aracılığıyla (örn. ip-api.com) coğrafi konumu (enlem/boylam ve şehir) otomatik bulur ve orkestratöre iletir.

**Geçiş İçin Gereksinimler:**
1. CLI giriş noktasında `location` argümanının opsiyonel hale getirilmesi.
2. `geocoding.py` içerisine ağ üzerinden IP tespiti ve enlem/boylam dönüşümü yapacak yeni bir `resolve_ip_location()` metodunun eklenmesi.
3. GeoIP servisine ulaşılamadığında uygulamanın hata vermek yerine kullanıcıdan `location` bilgisini girmesini isteyen mantıksal bir "fallback" mekanizmasının tasarlanması.

---

## Task 3: Akıllı Veri Önbellekleme (Smart Caching) ve Çevrimdışı (Offline) Mod
**Hedef:** Her çalıştırmada gereksiz API kotalarını harcamamak, performansı artırmak ve internetin olmadığı durumlarda dahi (uçak modu vs.) son güncel veriyi göstermek.

**Şu Anki Durum:**
Sadece konum aramaları `geo_cache.json` dosyasına 30 günlük TTL ile kaydedilmektedir. Gerçek hava durumu verileri için hiçbir önbellek (cache) yoktur. Her `weather` çağrısı sağlayıcılara canlı HTTP isteği atar.

**İstenen Durum:**
Başarılı olan API isteği (kazanan sağlayıcının verisi) yerel bir dosyaya (SQLite veya JSON) 15 dakikalık TTL ile kaydedilir. İlk 15 dakika içindeki isteklerde anında bu dosya okunur. İnternet kesilirse veya ağ hatası alınırsa, süresi geçmiş olsa bile bu veri "Çevrimdışı Mod" uyarısı ile ekranda gösterilir.

**Geçiş İçin Gereksinimler:**
1. Veri kaydetme işlemleri için `weather_cache.py` adında yeni bir modülün veri şeması dizaynının yapılması.
2. `run_weather_race` çağrısı yapılmadan önce `weather_cache.py` üzerinden 15 dakikalık TTL kontrolünün yapılması, hit varsa yarışın (race) başlatılmadan atlanması.
3. Orkestratördeki tüm provider'ların başarısız olduğu (`ConnectionError` atılan) durumun yakalanması ve mevcut bir önbellek varsa terminalde üst bilgi olarak "⚡ Çevrimdışı Mod: Veri XX:XX saatine aittir" şeklinde basacak fallback yapısının kurgulanması.

---

## Task 4: Hava Kalitesi Endeksi (AQI) Çekirdek Entegrasyonu
**Hedef:** Sadece standart hava koşullarını değil; karbonmonoksit, partikül madde gibi çevresel riskleri ölçüp "Dışarı Çıkma Endeksi" oluşturarak rekabette fark yaratmak.

**Şu Anki Durum:**
`providers/models.py` içindeki `HourlyPoint` modelinde sadece sıcaklık, hissedilen, rüzgar, yağış ve bulut örtüsü yer alır. AQI ($PM_{2.5}$, $CO$, $NO_2$ vb.) verilerini karşılayacak veri modelleri bulunmamaktadır.

**İstenen Durum:**
Meteorolojik verinin yanına, API'lerin hava kirliliği (AQI) verilerinin de çekilip sisteme eklenmesi, kritik seviyelerde tabloda renkli uyarı (örn. Yüksek PM2.5 Kırmızı) gösterilmesi.

**Geçiş İçin Gereksinimler:**
1. `HourlyPoint` (veya yeni bir `AQIData` nesnesi) içine $CO$, $NO_2$, $O_3$, $SO_2$, $PM_{2.5}$ ve $PM_{10}$ float değerlerinin eklenerek şemanın genişletilmesi.
2. İlgili veri sağlayıcıların (örn. Open-Meteo Air Quality API) uç noktalarının çağrılacak şekilde `fetch` metotlarının ve normalize etme kurallarının revize edilmesi.
3. Bu 6 ana kirletici üzerinden astım/alerji hastaları için bir katsayı oluşturulup `output.py`'a uyarı statüsü (İyi, Orta, Tehlikeli) döndüren bir iş katmanının (business logic) oluşturulması.

---

## Task 5: Çoklu Sağlayıcı Hiyerarşisi (Tiering), API Key ve Yandex Entegrasyonu
**Hedef:** OpenWeatherMap, WeatherBIT gibi premium API'lerin sisteme dahil edilmesi, kullanıcının kendi API anahtarlarını girebilmesi ve bölgesel avantaj sağlayan (örn: Türkiye) özel sağlayıcıların eklenmesi.

**Şu Anki Durum:**
Sistem sadece Open-Meteo, MET Norway ve wttr.in kullanır. `orchestrator.py` hepsini aynı anda yarışa sokar. Kullanıcının API key tanımlayabileceği bir ayar dosyası mekanizması yoktur.

**İstenen Durum:**
Birincil (Sıfır-Config) katman aynen çalışırken, kullanıcının `~/.config/weather_cli/config.toml` gibi bir dosyaya API Key girdiği premium servisler otomatik devreye girer. Ayrıca Türkiye veya Doğu Avrupa'dan yapılan konumlarda Yandex Weather mikro-sokak tahmini için API yarışına katılır.

**Geçiş İçin Gereksinimler:**
1. Kullanıcı yapılandırmalarını (API Key, Varsayılan Lokasyon, Metrik/Imperial tercihi) okuyan bir `config_manager` modülünün oluşturulması.
2. `OpenWeatherMapProvider`, `WeatherBitProvider` ve `YandexWeatherProvider` sınıflarının (base'i implemente eden) taslaklarının oluşturulması.
3. `orchestrator.py`'da lokasyon verisindeki `.country` özelliğini okuyup, eğer ülke "Turkey" ise Yandex API'yi havuza dahil eden zeki bir filtreleme / tier (katman) orkestrasyon algoritmasının yazılması.

---

## Task 6: Dinamik Boyut Algılama, Güvenli Font Modu (Fallback) ve Görsel Modlar
**Hedef:** Dikey bölünmüş küçük terminallerde tablonun kırılmasını (row break) önlemek, Nerd Font desteklemeyen terminallerde soru işareti çıkmasını engellemek, Neofetch stili kompakt çıktılar sunmak.

**Şu Anki Durum:**
`output.py` varsayılan olarak her zaman yatay 9 kolonlu bir tablo basar (`max_rows` sınırlaması dışında). Emoji kullanımı (☀️, 🌧️) standarttır, font kontrolü yapılmaz, `$COLUMNS` boyutu göz ardı edilir.

**İstenen Durum:**
Pencere boyutu ufaldığında araç otomatik olarak `Kompakt` görünüme geçer. Terminalin Unicode desteği yoksa ASCII (`|`, `-`, `*`) kullanır. Kullanıcı argümanla `--mode inline` diyerek tek satır çıktı alabilir (tmux vs. için).

**Geçiş İçin Gereksinimler:**
1. `SIGWINCH` veya `shutil.get_terminal_size()` ile terminal genişliğini ölçüp, belirlenen eşik değerin altındaysa TUI yapısını otomatik değiştiren (Responsive Terminal Design) bir karar mekanizması.
2. Ortam değişkenlerinden (`$TERM`, `$LC_CTYPE`, `$LANG`) terminalin gelişmiş emoji veya Nerd Font yeteneğini tespit eden veya tespit edemiyorsa güvenli ASCII haritalaması sunan bir rendering filtresi.
3. `print_hourly_table` fonksiyonuna alternatif olarak `print_inline_widget()` ve `print_compact_neofetch_style()` adında yeni sunum fonksiyonlarının (arayüz modellerinin) sistem çıktı akışına eklenmesi.
