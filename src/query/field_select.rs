use super::fields::find;
use crate::providers::models::HourlyPoint;

/// Projects `points` down to the requested `field_names`, returning `(header, rows)`.
/// Unknown field names are silently skipped — `--fields` validation happens at the CLI layer.
pub fn select_fields(
    points: &[HourlyPoint],
    field_names: &[String],
    precision: Option<usize>,
) -> (Vec<String>, Vec<Vec<String>>) {
    let defs: Vec<_> = field_names.iter().filter_map(|n| find(n)).collect();
    let header = defs.iter().map(|d| d.name.to_string()).collect();
    let rows = points
        .iter()
        .map(|p| {
            defs.iter()
                .map(|d| match (precision, (d.get_num)(p)) {
                    (Some(prec), Some(v)) => format!("{:.*}", prec, v),
                    _ => (d.get_str)(p),
                })
                .collect()
        })
        .collect();
    (header, rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point() -> HourlyPoint {
        HourlyPoint {
            time: "2026-05-19T22:00".to_string(),
            temperature: 18.456,
            apparent_temperature: 0.0,
            precipitation_probability: 0.0,
            precipitation: 0.0,
            humidity: 55.0,
            wind_speed: 0.0,
            wind_direction: 0.0,
            cloud_cover: 0.0,
            weather_code: 0,
            aqi: None,
            uv_index: None,
            is_day: None,
            visibility: None,
            soil_temperature: None,
            soil_moisture: None,
        }
    }

    #[test]
    fn selects_requested_columns() {
        let points = vec![point()];
        let (header, rows) = select_fields(
            &points,
            &["time".to_string(), "temp".to_string(), "humidity".to_string()],
            None,
        );
        assert_eq!(header, vec!["time", "temp", "humidity"]);
        assert_eq!(rows[0], vec!["2026-05-19T22:00", "18.5", "55.0"]);
    }

    #[test]
    fn applies_precision_override() {
        let points = vec![point()];
        let (_, rows) = select_fields(&points, &["temp".to_string()], Some(2));
        assert_eq!(rows[0], vec!["18.46"]);
    }
}
