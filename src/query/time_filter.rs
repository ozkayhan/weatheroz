use crate::providers::models::HourlyPoint;

fn hour_of(time: &str) -> Option<u32> {
    time.split('T').nth(1)?.split(':').next()?.parse().ok()
}

/// Keeps only points whose hour-of-day falls in `[from_hour, to_hour)`. Both bounds are
/// optional; an absent bound defaults to the full day (0..24).
pub fn filter_by_hour(
    points: &[HourlyPoint],
    from_hour: Option<u32>,
    to_hour: Option<u32>,
) -> Vec<HourlyPoint> {
    if from_hour.is_none() && to_hour.is_none() {
        return points.to_vec();
    }
    let from = from_hour.unwrap_or(0);
    let to = to_hour.unwrap_or(24);
    points
        .iter()
        .filter(|p| hour_of(&p.time).map(|h| h >= from && h < to).unwrap_or(false))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(time: &str) -> HourlyPoint {
        HourlyPoint {
            time: time.to_string(),
            temperature: 0.0,
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
    fn filters_hours_in_range() {
        let points = vec![
            point("2026-05-19T21:00"),
            point("2026-05-19T22:00"),
            point("2026-05-19T23:00"),
            point("2026-05-20T00:00"),
        ];
        let filtered = filter_by_hour(&points, Some(22), Some(24));
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].time, "2026-05-19T22:00");
        assert_eq!(filtered[1].time, "2026-05-19T23:00");
    }
}
