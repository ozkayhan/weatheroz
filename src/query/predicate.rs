use super::fields::find;
use crate::providers::models::HourlyPoint;

const OPS: &[&str] = &[">=", "<=", "!=", "==", "=", ">", "<"];

fn parse_expr(expr: &str) -> Result<(String, String, String), String> {
    for op in OPS {
        if let Some(idx) = expr.find(op) {
            let field = expr[..idx].trim().to_string();
            let value = expr[idx + op.len()..].trim().to_string();
            if field.is_empty() || value.is_empty() {
                break;
            }
            return Ok((field, op.to_string(), value));
        }
    }
    Err(format!(
        "Invalid --where expression '{}', expected '<field> <op> <value>' (op: = != > < >= <=)",
        expr
    ))
}

/// Filters `points` by a simple `field <op> value` expression, e.g. `temp>20` or `humidity<=50`.
pub fn parse_and_filter(points: &[HourlyPoint], expr: &str) -> Result<Vec<HourlyPoint>, String> {
    let (field, op, value) = parse_expr(expr)?;
    let def = find(&field).ok_or_else(|| format!("Unknown field in --where: '{}'", field))?;

    let filtered = points
        .iter()
        .filter(|p| match (def.get_num)(p) {
            Some(num) => match value.parse::<f64>() {
                Ok(target) => match op.as_str() {
                    ">" => num > target,
                    "<" => num < target,
                    ">=" => num >= target,
                    "<=" => num <= target,
                    "=" | "==" => (num - target).abs() < f64::EPSILON,
                    "!=" => (num - target).abs() >= f64::EPSILON,
                    _ => false,
                },
                Err(_) => false,
            },
            None => {
                let s = (def.get_str)(p);
                match op.as_str() {
                    "=" | "==" => s == value,
                    "!=" => s != value,
                    _ => false,
                }
            }
        })
        .cloned()
        .collect();
    Ok(filtered)
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
    fn filters_numeric_gt() {
        let points = vec![point(15.0), point(25.0)];
        let filtered = parse_and_filter(&points, "temp>20").unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].temperature, 25.0);
    }

    #[test]
    fn rejects_unknown_field() {
        assert!(parse_and_filter(&[], "bogus>1").is_err());
    }
}
