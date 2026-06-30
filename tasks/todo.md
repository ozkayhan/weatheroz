# Todo — Query Pipeline

## Phase 0 (hemen, bağımsız)
- [x] T0: Cache key bug fix — `src/weather_cache.rs` — **Haiku**

## Phase 1 (T0 ile paralel)
- [x] T1a: Timezone — `Cargo.toml` + `src/providers/base.rs` + `src/providers/openmeteo.rs` + `src/cli.rs` — **Haiku**
- [x] T1b: Field registry — `src/query/fields.rs` (yeni) — **Sonnet**

## Phase 2 (T1a+T1b bittikten sonra, paralel)
- [x] T2a: Time filter `--from-hour/--to-hour` — `src/query/time_filter.rs` — **Haiku**
- [x] T2b: Field select `--fields` — `src/query/field_select.rs` — **Haiku**
- [x] T2c: WHERE predicate `--where` — `src/query/predicate.rs` — **Sonnet**

## Phase 3 (T2 bittikten sonra, paralel)
- [x] T3a: Aggregation `--aggregate` — `src/query/aggregate.rs` — **Sonnet**
- [x] T3b: Sort + limit `--sort/--limit` — `src/query/sort.rs` — **Haiku**

## Phase 4 (T3 bittikten sonra)
- [x] T4: Output controls `--no-headers/--precision/--dry-run` — `src/output.rs` — **Haiku**

## Phase 5 (hepsi bittikten sonra)
- [x] T5: CLI wire + smoke test — `src/cli.rs` + `src/main.rs` — **Sonnet**

---
## Smoke Test Komutu
```bash
weatheroz "ankara çankaya" --days 20 --timezone Europe/Istanbul \
  --from-hour 22 --to-hour 24 --fields temp,humidity --no-headers
```
Beklenen: ~40 satır (20 gün × 2 saat), 2 kolon (time, temp, humidity), header yok.
