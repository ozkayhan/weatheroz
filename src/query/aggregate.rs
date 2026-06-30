use super::fields::find;
use crate::providers::models::HourlyPoint;

/// Computes a single aggregate from a `field:func` expression, e.g. `temp:avg`.
/// Supported functions: min, max, avg, sum, count.
pub fn aggregate(points: &[HourlyPoint], expr: &str) -> Result<(String, f64), String> {
    let (field, func) = expr
        .split_once(':')
        .ok_or_else(|| format!("Invalid --aggregate expression '{}', expected 'field:func'", expr))?;
    let def = find(field).ok_or_else(|| format!("Unknown field in --aggregate: '{}'", field))?;
    let values: Vec<f64> = points.iter().filter_map(|p| (def.get_num)(p)).collect();

    let result = match func {
        "min" => values.iter().cloned().fold(f64::INFINITY, f64::min),
        "max" => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        "sum" => values.iter().sum(),
        "avg" => {
            if values.is_empty() {
                0.0
            } else {
                values.iter().sum::<f64>() / values.len() as f64
            }
        }
        "count" => values.len() as f64,
        other => return Err(format!("Unknown aggregate function: '{}'", other)),
    };
    Ok((format!("{}:{}", field, func), result))
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
    fn computes_avg() {
        let points = vec![point(10.0), point(20.0)];
        let (label, value) = aggregate(&points, "temp:avg").unwrap();
        assert_eq!(label, "temp:avg");
        assert_eq!(value, 15.0);
    }
}
