# ANALIZ.MD

## Yeni Nesil Açık Kaynak Hava Durumu CLI Aracı Stratejik Planlama ve Rakip Analizi Dokümanı

Bu doküman, mevcut ekosistemdeki 20 hava durumu CLI ve GUI aracının teknik mimarilerini, veri kaynaklarını, dağıtım kanallarını, avantajlarını ve kısıtlamalarını analiz ederek, pazardaki tüm rakiplerinden daha üstün, yüksek performanslı ve optimize bir kullanıcı deneyimi sunacak yeni bir açık kaynak terminal uygulamasının mimari standartlarını belirlemek üzere hazırlanmıştır.

---

## 1. Mevcut Ekosistem Teknik Dağılım Matrisi

Aşağıdaki tablolar, incelenen 20 aracın programlama dilleri, veri sağlayıcı bağımlılıkları ve konfigürasyon gereksinimlerine göre dağılımını göstermektedir.

### 1.1. Geliştirme Dili ve Çalışma Zamanı Dağılımı

| Dil / Altyapı             | Araçlar                                                                       | Stratejik Değerlendirme                                                                                            |
| ------------------------- | ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| **Go (Golang)**           | `wego`, `stormy`, `weatherman`, `forecast-cli`, `yandex-weather-cli`          | Harici bağımlılık barındırmaz. Milisaniye seviyesinde açılış hızına sahiptir. Statik binary üretimi için uygundur. |
| **Rust**                  | `wthrr-the-weathercrab`, `weather-rs`                                         | Bellek güvenliği, yüksek paralellik ve ekstrem optimizasyon sağlar. TUI kütüphaneleri gelişmiştir.                 |
| **Python**                | `rainy`, `cli-weather`, `weather-forecast-cli`, `wttr.in` (sunucu altyapısı)  | Çalışma zamanı (runtime) bağımlılığı yüksektir. Pip ve interpreter kurulu olma şartı dağıtımı zorlaştırır.         |
| **Node.js / JS**          | `yr-cli`, `best-weather-cli`, `npm-weather-cli`, `weather-cli-project-nodejs` | Ağır bağımlılık ağacı (`node_modules`) oluşturur. Hızlı prompt entegrasyonlarında gecikme (latency) yaratır.       |
| **POSIX Shell**           | `ansiweather`                                                                 | Sadece `curl` ve `jq` bağımlılığına sahiptir. Taşınabilirdir ancak karmaşık TUI yapıları için yetersizdir.         |
| **Vala / GTK / Masaüstü** | `gnome-weather`, `meteo`, `meteorologist`                                     | Grafik arayüz odaklıdır. Sistem kaynak tüketimi yüksektir, script entegrasyonuna uygun değildir.                   |
| **Snap Bağımsız**         | `weather-cli`                                                                 | Sandbox içinde izole çalışır, disk boyutu büyüktür ve ilk açılışta gecikme yaşatır.                                |

### 1.2. Veri Sağlayıcı ve API Anahtarı Politikası

| Veri Sağlayıcı (API)   | Kullanan Araçlar                                                                                                                                                                                       | API Key Gereksinimi                                | Kapsam ve Limit Kısıtlamaları                                                                                     |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| **OpenWeatherMap**     | `wttr.in`, `wego`, `stormy`, `rainy`, `weather-rs`, `weatherman`, `weather-forecast-cli`, `best-weather-cli`, `npm-weather-cli`, `weather-cli-project-nodejs`, `meteorologist`, `ansiweather`, `meteo` | Zorunlu (Yerleşik key tükenirse kota hatası verir) | Geniş veri kümesi sunar ancak yoğun trafikte HTTP 429 (Rate Limit) veya 403 hataları üretir.                      |
| **Open-Meteo**         | `wego`, `stormy`, `wthrr-the-weathercrab`, `weather-rs`                                                                                                                                                | **Gerektirmez** (Kullanıma hazır)                  | Anında kurulum (out-of-the-box) deneyimi sağlar. Kararlı ve hızlıdır.                                             |
| **MET Norway (yr.no)** | `yr-cli`, `gnome-weather`                                                                                                                                                                              | **Gerektirmez**                                    | Avrupa ve Kuzey Yarımküre odaklı yüksek doğruluk oranı. Adil kullanım politikası (User-Agent zorunluluğu) vardır. |
| **Yandex Weather API** | `yandex-weather-cli`                                                                                                                                                                                   | Zorunlu                                            | Doğu Avrupa ve Türkiye için sokak/mahalle ölçeğinde mikro radar tahmini sağlar. Küresel kapsamı zayıftır.         |
| **WeatherBIT / Stack** | `cli-weather`, `weather-cli-project-nodejs`                                                                                                                                                            | Zorunlu                                            | Gelişmiş hava kalitesi (AQI) verileri içerir, ücretsiz plan limitleri dardır.                                     |

---

## 2. Sektörel Standartlar ve Ortak Güçlü Yönler (Kazanılacak Cepheler)

İnşa edilecek yeni CLI aracının, rakiplerinin başarı yakaladığı aşağıdaki temel özellikleri asgari standart olarak kabul etmesi ve bünyesinde barındırması şarttır:

- **Sıfır Konfigürasyon ile Anında Çalışma (Zero-Config UX):** `wttr.in`, `wthrr` ve `stormy` araçlarında olduğu gibi, kullanıcı uygulamayı çalıştırdığı anda herhangi bir API anahtarı üretmekle veya üyelikle uğraşmamalıdır. Sistem, varsayılan olarak Open-Meteo veya MET Norway gibi açık kaynaklı veri sağlayıcıları üzerinden doğrudan çalışmaya başlamalıdır.
- **IP Tabanlı Otomatik Konum Tespiti:** `wttr.in`, `wthrr` ve `weather-rs` projelerinde uygulanan, GeoIP veritabanları veya web servisleri üzerinden kullanıcının coğrafi konumunu (şehir/ülke) otomatik algılama mekanizması entegre edilmelidir. Seyahat eden kullanıcı manuel müdahale yapmadan doğru veriyi alabilmelidir.
- **Çoklu Görsel Mod Seçeneği:**
- _Inline Mod:_ `wttr.in?format=3` ve `ansiweather` benzeri, `tmux`, `polybar`, `i3status` veya kabuk promptlarına (`bash`, `zsh`) widget olarak gömülmeye uygun tek satırlık çıktı formatı.
- _Kompakt Mod (Neofetch-Style):_ `stormy` ve `rainy` araçlarının öncülük ettiği, sol tarafta ASCII grafik ikonu, sağ tarafta ise temel metriklerin yer aldığı minimalist düzen.
- _Zengin TUI Modu:_ `wthrr` ve `wego` örneklerindeki gibi, terminal pencerelerini dolduran çizgi grafikleri, saatlik/haftalık dikey tahmin kolonları ve gelişmiş tablolar.

---

## 3. Ortak Zayıf Yönler ve Sektörel Açıklar (Saldırılacak Cepheler)

Mevcut 20 aracın tamamı incelendiğinde, kullanıcı deneyimini baltalayan ve yeni üründe kesinlikle çözülmesi gereken 5 kritik kronik problem tespit edilmiştir:

1. **Tek Bir Sağlayıcıya veya API Anahtarına Bağımlılık:** Araçların büyük kısmı yalnızca OpenWeatherMap API'sine güvenmektedir. Bu durum, ortak API anahtarı kota sınırına ulaştığında tüm kullanıcı tabanının sistem dışı kalmasına (HTTP 429) neden olmaktadır. Çözüm, soyutlanmış bir çoklu sağlayıcı (multi-provider proxy) mimarisi kurmaktır.
2. **Yazı Tipi (Font) ve Karakter Kırılmaları:** `wthrr-the-weathercrab` başta olmak üzere, modern zengin semboller ve rüzgar yön okları kullanan araçlar, sisteminde Nerd Fonts veya genişletilmiş Unicode desteği olmayan kullanıcılarda kare (``) veya bozuk karakterler göstermektedir. Çözüm, sistem font yeteneklerini otomatik tespit eden fallback (standart ASCII karakterlerine geri dönme) mekanizmasıdır.
3. **Çalışma Zamanı (Runtime) Sürüm Karmaşası ve Bağımlılık Yükü:** Node.js tabanlı (`yr-cli`, `npm-weather-cli`) ve Python tabanlı (`cli-weather`, `rainy`) araçlar, son kullanıcının makinesinde belirli interpreter sürümlerinin kurulu olmasını gerektirir. Zamanla güncellenmeyen kütüphaneler güvenlik açığı (Snyk raporlarında görüldüğü üzere) oluşturur veya çöker. Çözüm, statik derlenen ve bağımlılık içermeyen bir dil kullanımıdır.
4. **Terminal Pencere Genişliği Uyumsuzluğu:** `wego` ve bazı TUI araçları, terminal penceresi küçültüldüğünde veya dikey split (bölünmüş ekran) modunda çalıştırıldığında satır kaymaları yaşamakta ve arayüz okunamaz hale gelmektedir. Çözüm, dinamik terminal boyutu takibi (`SIGWINCH` sinyal yönetimi) ve responsive TUI tasarımıdır.
5. **Ağ Kesintilerinde Tam Çökme (Önbellek Eksikliği):** Araçların ezici çoğunluğu her çalıştırıldığında canlı ağ isteği atar. İnternet kesildiğinde veya uçaktayken çalışamazlar. `wego` ve `wthrr` kısmi önbellek sunsa da yetersizdir. Çözüm, diske senkronize yerel veri saklama altyapısıdır.

---

## 4. Yeni Nesil CLI Mimari ve Özellik Standartları Kılavuzu

Yeni inşa edilecek hava durumu CLI aracının, yukarıdaki analizlerden yola çıkarak rakiplerini geride bırakması için uygulaması gereken teknik ve operasyonel standartlar aşağıda listelenmiştir.

### 4.1. Çekirdek Teknik Mimari Standartları

- **Geliştirme Dili Seçimi:** Uygulama, sıfır harici çalışma zamanı bağımlılığı, milisaniyeler düzeyinde soğuk başlatma (cold-start) süresi, yüksek bellek optimizasyonu ve responsive TUI kütüphaneleri (örn: `ratatui`) nedeniyle **Rust** veya statik binary üretim hızı ve çoklu platform (cross-compilation) kolaylığı nedeniyle **Go** diliyle yazılmalıdır. Python veya Node.js kesinlikle kullanılmamalıdır.
- **Soyutlanmış Dinamik Veri Katmanı (Multi-Provider Orchestration):** Tek bir API kaynağına bağımlı kalınmamalıdır. Çekirdek mimari, bir "Sağlayıcı Arabirimi" (Provider Interface) üzerinden çalışmalıdır.
- _Birincil Katman (Sıfır-Config):_ Open-Meteo ve MET Norway. API key istemez.
- _İkincil Katman (Gelişmiş Veri):_ Kullanıcı kendi OpenWeatherMap veya WeatherBIT keyini girerse sistem otomatik olarak daha detaylı tarihsel analiz moduna geçmelidir.
- _Bölgesel Mod:_ Konum Doğu Avrupa veya Türkiye olarak algılanırsa, opsiyonel olarak Yandex Weather API entegrasyonu tetiklenerek sokak bazlı mikro tahmin aktif edilmelidir.

- **Akıllı Hata Yönetimi ve Otomatik Fallback (Failover):** Eğer birincil veri sağlayıcı ağ hatası (HTTP 500, 503) veya kota aşımı (HTTP 429) verirse, CLI bunu kullanıcıya hissettirmeden 200 milisaniye içinde ikincil sağlayıcıya yönlendirmeli ve veriyi oradan çekmelidir.

### 4.2. Gelişmiş Özellik Seti (Rakiplerde Olmayan / Eksik Olan)

- **Çevre ve Sağlık Analizi Entegrasyonu (Gelişmiş AQI):** Yalnızca `cli-weather` ve `weatherman` araçlarında kısmen bulunan hava kirliliği raporlaması standart hale getirilmelidir. Sıcaklık verisinin hemen yanında karbonmonoksit ($CO$), azotdioksit ($NO_2$), ozon ($O_3$), kükürtdioksit ($SO_2$) ve özellikle ince partikül maddeler ($PM_{2.5}$ ve $PM_{10}$) gösterilerek astım, alerji veya sporcu kullanıcılar için dışarı çıkma uygunluk endeksi hesaplanmalıdır.
- **Dinamik Akıllı Önbellekleme (Smart Caching & Offline Mode):** Her çalıştırmada API'ye istek atmak yerine, son başarılı sorgu yerel bir hafif veritabanına (örn: SQLite veya optimize edilmiş JSON dosyası) TTL (Time-To-Live) süresi 15 dakika olacak şekilde kaydedilmelidir. 15 dakika içindeki mükerrer çalıştırmalarda veri doğrudan diskten okunarak 1 milisaniyede ekrana basılmalı ve API kotaları korunmalıdır. Ağ bağlantısı tamamen koptuğunda, araç hata verip kapanmak yerine en son önbelleğe alınan veriyi "Çevrimdışı Mod" uyarısıyla sunmalıdır.
- **Sensör Tabanlı Font ve Ekran Algılama:** _ Uygulama, terminalin Nerd Fonts / Unicode yeteneğini test etmelidir. Eğer terminal desteklemiyorsa, görsel ikonlar otomatik olarak güvenli ASCII karakter moduna (`|`, `-`, `_`) dönüştürülmelidir.
- Terminal genişliği `$COLUMNS` değişkeni üzerinden anlık izlenmeli, dikey split ekranlarda arayüz kendiliğinden kompakt moda (tek sütun) geçerek görsel kırılmaları engellemedir.

### 4.3. Dağıtım ve Sürdürülebilirlik Stratejisi

- **Kapsamlı Paket Yöneticisi Dağıtımı:** Dağıtım stratejisi, tek bir kanala (`snap` veya `npm`) sıkışmamalıdır. CI/CD süreçleri (GitHub Actions) otomatikleştirilerek araç her sürümde eş zamanlı olarak şu kanallarda yayınlanmalıdır:
- **macOS:** Homebrew (Core Formula olarak, Cask bağımlılığı olmadan yerel binary).
- **Linux (Debian/Ubuntu):** Resmi `.deb` paketi ve PPA deposu.
- **Linux (Arch):** Resmi AUR paketi (`yay` entegrasyonu).
- **Evrensel / Sistem Bağımsız:** `cargo install` veya `go install` altyapısı ve hafif, izole olmayan yerel binary sürümleri.

- **Kurulumsuz Web İstemci Katmanı (Curl Wrapper):** `wttr.in` modelinin başarısı kurulum gerektirmemesidir. Geliştirilecek CLI aracının aynısı uzak bir sunucuda da host edilmelidir. Kullanıcı kendi yerel makinesine aracı kurmak istemediğinde `curl evrensel-hava-durumu.ci` komutuyla terminal ekranına yerel binary ile tamamen aynı görsel kalitede ve renkte (ANSI escape kodlarıyla biçimlendirilmiş) çıktıyı alabilmelidir.

---

## 5. Sonuç ve Geliştirme Ekibine Özet Talimat

ANALIZ.MD dokümanında belirtilen parametreler doğrultusunda inşa edilecek yeni CLI aracı; **Rust veya Go** tabanlı mimarisiyle `yr-cli` ve `cli-weather`'ın bağımlılık sorunlarını yok edecek, **otomatik fallback mekanizmasıyla** `wttr.in`'in rate-limit çöküşlerini egale edecek, **akıllı font/ekran algılamasıyla** `wthrr`'ın karakter kırılmalarını çözecek ve **entegre AQI (Hava Kalitesi) motoruyla** standart bir meteoroloji aracından çok daha derin analitik veri sunacaktır. Ekip, mimariyi oluştururken doğrudan "Sıfır-Konfigürasyon" ve "Çoklu Sağlayıcı Uç Noktası" yapılandırması üzerine odaklanmalıdır.
