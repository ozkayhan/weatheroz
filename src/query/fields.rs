use crate::providers::models::HourlyPoint;

/// One queryable column: a display name plus a string getter (for output) and an optional
/// numeric getter (for `--where`, `--sort`, `--aggregate`). `get_num` is `None` for fields with
/// no numeric meaning (e.g. `time`).
pub struct FieldDef {
    pub name: &'static str,
    pub get_str: fn(&HourlyPoint) -> String,
    pub get_num: fn(&HourlyPoint) -> Option<f64>,
}

fn opt_f64(v: Option<f64>) -> String {
    v.map(|n| format!("{:.1}", n)).unwrap_or_default()
}

pub const FIELDS: &[FieldDef] = &[
    FieldDef {
        name: "time",
        get_str: |h| h.time.clone(),
        get_num: |_| None,
    },
    FieldDef {
        name: "temp",
        get_str: |h| format!("{:.1}", h.temperature),
        get_num: |h| Some(h.temperature),
    },
    FieldDef {
        name: "apparent_temp",
        get_str: |h| format!("{:.1}", h.apparent_temperature),
        get_num: |h| Some(h.apparent_temperature),
    },
    FieldDef {
        name: "humidity",
        get_str: |h| format!("{:.1}", h.humidity),
        get_num: |h| Some(h.humidity),
    },
    FieldDef {
        name: "wind_speed",
        get_str: |h| format!("{:.1}", h.wind_speed),
        get_num: |h| Some(h.wind_speed),
    },
    FieldDef {
        name: "wind_direction",
        get_str: |h| format!("{:.1}", h.wind_direction),
        get_num: |h| Some(h.wind_direction),
    },
    FieldDef {
        name: "precipitation",
        get_str: |h| format!("{:.1}", h.precipitation),
        get_num: |h| Some(h.precipitation),
    },
    FieldDef {
        name: "precipitation_probability",
        get_str: |h| format!("{:.1}", h.precipitation_probability),
        get_num: |h| Some(h.precipitation_probability),
    },
    FieldDef {
        name: "cloud_cover",
        get_str: |h| format!("{:.1}", h.cloud_cover),
        get_num: |h| Some(h.cloud_cover),
    },
    FieldDef {
        name: "weather_code",
        get_str: |h| h.weather_code.to_string(),
        get_num: |h| Some(h.weather_code as f64),
    },
    FieldDef {
        name: "uv_index",
        get_str: |h| opt_f64(h.uv_index),
        get_num: |h| h.uv_index,
    },
    FieldDef {
        name: "visibility",
        get_str: |h| opt_f64(h.visibility),
        get_num: |h| h.visibility,
    },
    FieldDef {
        name: "soil_temperature",
        get_str: |h| opt_f64(h.soil_temperature),
        get_num: |h| h.soil_temperature,
    },
    FieldDef {
        name: "soil_moisture",
        get_str: |h| opt_f64(h.soil_moisture),
        get_num: |h| h.soil_moisture,
    },
];

pub fn find(name: &str) -> Option<&'static FieldDef> {
    FIELDS.iter().find(|f| f.name == name)
}

pub fn names() -> Vec<&'static str> {
    FIELDS.iter().map(|f| f.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_known_field() {
        assert!(find("temp").is_some());
        assert!(find("nonexistent").is_none());
    }
}
