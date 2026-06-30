use super::fields::find;
use crate::providers::models::HourlyPoint;

/// Sorts by a `field` (ascending) or `-field` (descending) expression, then truncates to `limit`.
pub fn sort_and_limit(
    mut points: Vec<HourlyPoint>,
    sort_expr: Option<&str>,
    limit: Option<usize>,
) -> Result<Vec<HourlyPoint>, String> {
    if let Some(expr) = sort_expr {
        let (field, desc) = match expr.strip_prefix('-') {
            Some(f) => (f, true),
            None => (expr, false),
        };
        let def = find(field).ok_or_else(|| format!("Unknown field in --sort: '{}'", field))?;
        points.sort_by(|a, b| {
            let av = (def.get_num)(a).unwrap_or(f64::NAN);
            let bv = (def.get_num)(b).unwrap_or(f64::NAN);
            av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal)
        });
        if desc {
            points.reverse();
        }
    }
    if let Some(n) = limit {
        points.truncate(n);
    }
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(temp: f64) -> HourlyPoint {
        HourlyPoint {
            time: "2026-05-19T22:00".to_string(),
            temperature: temp,
            apparent_temperature: 0.0,
            precipitation_probability: 0.0,
            precipitation: 0.0,
            humidity: 0.0,
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
    fn sorts_descending_and_limits() {
        let points = vec![point(10.0), point(30.0), point(20.0)];
        let result = sort_and_limit(points, Some("-temp"), Some(2)).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].temperature, 30.0);
        assert_eq!(result[1].temperature, 20.0);
    }
}
