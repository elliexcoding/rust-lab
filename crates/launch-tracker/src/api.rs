use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de, Deserialize, Deserializer};

pub const LAUNCHES_URL: &str = "https://fdo.rocketlaunch.live/json/launches/next/5";

#[derive(Clone, Debug, PartialEq)]
pub struct Launch {
    pub id: u64,
    pub name: String,
    pub provider: String,
    pub vehicle: String,
    pub pad: String,
    pub location: String,
    pub t0: Option<DateTime<Utc>>,
    pub date_estimate: String,
    pub description: String,
    pub weather: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LaunchResponse {
    result: Vec<ApiLaunch>,
}

#[derive(Debug, Deserialize)]
struct ApiLaunch {
    id: u64,
    name: String,
    provider: NamedResource,
    vehicle: NamedResource,
    pad: ApiPad,
    #[serde(default, deserialize_with = "deserialize_optional_datetime")]
    t0: Option<DateTime<Utc>>,
    date_str: String,
    launch_description: String,
    weather_condition: Option<String>,
    weather_temp: Option<String>,
    weather_wind_mph: Option<String>,
}

fn deserialize_optional_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(&value)
                .map(|date| date.with_timezone(&Utc))
                .or_else(|_| {
                    NaiveDateTime::parse_from_str(&value, "%Y-%m-%dT%H:%MZ")
                        .map(|date| date.and_utc())
                })
                .map_err(de::Error::custom)
        })
        .transpose()
}

#[derive(Debug, Deserialize)]
struct NamedResource {
    name: String,
}

#[derive(Debug, Deserialize)]
struct ApiPad {
    name: String,
    location: ApiLocation,
}

#[derive(Debug, Deserialize)]
struct ApiLocation {
    name: String,
    state: Option<String>,
    country: String,
}

impl From<ApiLaunch> for Launch {
    fn from(value: ApiLaunch) -> Self {
        let location = match value.pad.location.state.as_deref() {
            Some(state) if !state.is_empty() => format!(
                "{}, {}, {}",
                value.pad.location.name, state, value.pad.location.country
            ),
            _ => format!(
                "{}, {}",
                value.pad.location.name, value.pad.location.country
            ),
        };

        let weather = weather_summary(
            value.weather_condition,
            value.weather_temp,
            value.weather_wind_mph,
        );

        Self {
            id: value.id,
            name: value.name,
            provider: value.provider.name,
            vehicle: value.vehicle.name,
            pad: value.pad.name,
            location,
            t0: value.t0,
            date_estimate: value.date_str,
            description: value.launch_description.replace('\n', " "),
            weather,
        }
    }
}

pub fn fetch_launches() -> Result<Vec<Launch>> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(12))
        .no_proxy()
        .user_agent(concat!("launch-tracker/", env!("CARGO_PKG_VERSION")))
        .build()
        .context("could not create the HTTP client")?;

    let response = client
        .get(LAUNCHES_URL)
        .send()
        .context("could not reach RocketLaunch.Live")?
        .error_for_status()
        .context("RocketLaunch.Live returned an error")?
        .json::<LaunchResponse>()
        .context("could not decode the launch feed")?;

    Ok(response.result.into_iter().map(Launch::from).collect())
}

fn weather_summary(
    condition: Option<String>,
    temp_f: Option<String>,
    wind_mph: Option<String>,
) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(condition) = condition.filter(|value| !value.is_empty()) {
        parts.push(condition);
    }
    if let Some(temp) = temp_f.filter(|value| !value.is_empty()) {
        parts.push(format!("{temp} F"));
    }
    if let Some(wind) = wind_mph.filter(|value| !value.is_empty()) {
        parts.push(format!("wind {wind} mph"));
    }

    (!parts.is_empty()).then(|| parts.join("  |  "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scheduled_and_estimated_launches() {
        let fixture = r#"
        {
          "result": [
            {
              "id": 1,
              "name": "Artemis Test",
              "provider": { "name": "NASA" },
              "vehicle": { "name": "SLS" },
              "pad": {
                "name": "LC-39B",
                "location": {
                  "id": 61,
                  "name": "Kennedy Space Center",
                  "state": "FL",
                  "country": "United States"
                }
              },
              "t0": "2026-08-12T10:30Z",
              "date_str": "Aug 12",
              "launch_description": "A test launch.\nSecond line.",
              "weather_condition": "Clear",
              "weather_temp": "78.00",
              "weather_wind_mph": "5.20"
            },
            {
              "id": 2,
              "name": "Estimated Mission",
              "provider": { "name": "Example Space" },
              "vehicle": { "name": "Example 1" },
              "pad": {
                "name": "Pad TBD",
                "location": {
                  "id": 999,
                  "name": "Somewhere",
                  "state": null,
                  "country": "Japan"
                }
              },
              "t0": null,
              "date_str": "Aug 2026",
              "launch_description": "An estimated launch.",
              "weather_condition": null,
              "weather_temp": null,
              "weather_wind_mph": null
            }
          ]
        }
        "#;

        let response: LaunchResponse = serde_json::from_str(fixture).expect("valid fixture");
        let launches: Vec<Launch> = response.result.into_iter().map(Launch::from).collect();

        assert_eq!(launches.len(), 2);
        assert_eq!(
            launches[0].location,
            "Kennedy Space Center, FL, United States"
        );
        assert_eq!(
            launches[0].weather.as_deref(),
            Some("Clear  |  78.00 F  |  wind 5.20 mph")
        );
        assert_eq!(launches[0].description, "A test launch. Second line.");
        assert!(launches[0].t0.is_some());
        assert_eq!(launches[1].location, "Somewhere, Japan");
        assert!(launches[1].t0.is_none());
        assert!(launches[1].weather.is_none());
    }
}
