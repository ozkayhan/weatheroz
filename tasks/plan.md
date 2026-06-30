# Plan — Query Pipeline

`tasks/plan.md` repoda yoktu; bu dosya kod incelendikten sonra geriye dönük yazıldı.

## T0 — Cache key bug
`make_cache_key` `enrich`/`minute_resolution` bayraklarını anahtara katmıyordu: aynı
konum+tarih aralığı için `--enrich` açık/kapalı arasında geçiş yapan iki çağrı, TTL
penceresi içinde birbirinin (yanlış şekilli) verisini cache'ten alıyordu. Anahtara
`enrich`/`minute_resolution` eklendi; `get_cached_weather`/`save_cached_weather`
imzaları buna göre genişletildi, çağrı yerleri (`orchestrator.rs`, testler) güncellendi.

## T1a — Timezone
Open-Meteo API'si `timezone` parametresini `auto` yerine doğrudan herhangi bir IANA
tz string'i olarak kabul ediyor ve zaten yerelleştirilmiş saatler döndürüyor — bu
yüzden istemci tarafında dönüşüm kodu gerekmedi. `--timezone` flag'i eklendi,
`chrono-tz` ile (yalnızca doğrulama amacıyla) IANA string olarak kontrol ediliyor,
`FetchContext`'e `timezone: Option<&str>` alanı eklenip `OpenMeteoProvider::build_url`
bunu `timezone=auto` yerine kullanıyor. Diğer sağlayıcılar bu alanı görmezden gelir.

## T1b–T4 — Query pipeline
`src/query/` altında bağımsız, test edilebilir modüller:
- `fields.rs`: alan adı → `HourlyPoint` getter eşlemesi (string + opsiyonel numeric).
- `time_filter.rs`: `--from-hour/--to-hour` ile saat bazlı filtre.
- `field_select.rs`: `--fields` ile kolon projeksiyonu + `--precision`.
- `predicate.rs`: `--where "field<op>value"` filtresi.
- `aggregate.rs`: `--aggregate field:func` (min/max/avg/sum/count).
- `sort.rs`: `--sort [-]field` + `--limit`.
- `output.rs::print_query_rows`: tab-ayrılmış, `--no-headers` destekli basit tablo.

## T5 — CLI wiring
`cli.rs`'ye tüm yeni bayraklar eklendi. `main.rs`'de pipeline yalnızca `--fields`
veya `--aggregate` verildiğinde devreye girer (sırasıyla: hour filter → where →
aggregate-ve-çık / sort+limit → field select → yazdır). `--dry-run` ağ çağrısı
yapmadan çözümlenen sorgu planını basıp çıkar.

## Smoke test notu
`--days 20` ile Open-Meteo'nun forecast API'si "out of allowed range" (~14-15 gün)
hatası veriyor — bu sağlayıcının önceden var olan bir kısıtı, pipeline'la ilgisi yok.
wttr.in yedeğe düşüp 3 günlük veriyle 6 satır üretiyor; `--days 14` ile Open-Meteo
kazanıyor ve beklenen 28 satır (14 gün × 2 saat) doğru şekilde üretiliyor. Pipeline
mekaniği (saat filtresi, kolon seçimi, header'sız çıktı) doğrulandı.
