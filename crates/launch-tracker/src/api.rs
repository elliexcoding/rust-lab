use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{de, Deserialize, Deserializer};

pub const LAUNCHES_URL: &str = "https://fdo.rocketlaunch.live/json/launches/next/5";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
}

impl GeoPoint {
    pub const fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Launch {
    pub id: u64,
    pub name: String,
    pub provider: String,
    pub vehicle: String,
    pub pad: String,
    pub site_name: String,
    pub location: String,
    pub coordinates: Option<GeoPoint>,
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
    id: u64,
    name: String,
    state: Option<String>,
    country: String,
}

impl From<ApiLaunch> for Launch {
    fn from(value: ApiLaunch) -> Self {
        let site_name = value.pad.location.name.clone();
        let coordinates = coordinates_for_location(value.pad.location.id, &site_name);
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
            site_name,
            location,
            coordinates,
            t0: value.t0,
            date_estimate: value.date_str,
            description: value.launch_description.replace('\n', " "),
            weather,
        }
    }
}

fn coordinates_for_location(id: u64, name: &str) -> Option<GeoPoint> {
    let name = name.to_lowercase();

    // RocketLaunch.Live exposes exact coordinates through its authenticated
    // Locations resource, but the free next-five feed only includes an ID and
    // name. These approximate facility centres keep the free app useful without
    // pretending that mobile or unknown launch positions are precise.
    let point = if id == 60 || name.contains("vandenberg") {
        GeoPoint::new(34.7420, -120.5724)
    } else if name.contains("cape canaveral") {
        GeoPoint::new(28.4889, -80.5778)
    } else if name.contains("kennedy space center") {
        GeoPoint::new(28.5729, -80.6490)
    } else if name.contains("baikonur") {
        GeoPoint::new(45.9200, 63.3420)
    } else if name.contains("plesetsk") {
        GeoPoint::new(62.9270, 40.5770)
    } else if name.contains("vostochny") {
        GeoPoint::new(51.8840, 128.3330)
    } else if name.contains("kapustin yar") {
        GeoPoint::new(48.5780, 46.2540)
    } else if name.contains("jiuquan") {
        GeoPoint::new(40.9606, 100.2983)
    } else if name.contains("xichang") {
        GeoPoint::new(28.2460, 102.0266)
    } else if name.contains("wenchang") {
        GeoPoint::new(19.6145, 110.9510)
    } else if name.contains("taiyuan") {
        GeoPoint::new(38.8491, 111.6085)
    } else if name.contains("satish dhawan") || name.contains("sriharikota") {
        GeoPoint::new(13.7199, 80.2304)
    } else if name.contains("mahia") {
        GeoPoint::new(-39.2615, 177.8649)
    } else if name.contains("tanegashima") {
        GeoPoint::new(30.4009, 130.9775)
    } else if name.contains("uchinoura") {
        GeoPoint::new(31.2510, 131.0810)
    } else if name.contains("guiana space") || name.contains("kourou") {
        GeoPoint::new(5.2360, -52.7750)
    } else if name.contains("wallops") || name.contains("mid-atlantic regional") {
        GeoPoint::new(37.9402, -75.4664)
    } else if name.contains("pacific spaceport") || name.contains("kodiak") {
        GeoPoint::new(57.4350, -152.3390)
    } else if name.contains("starbase") || name.contains("boca chica") {
        GeoPoint::new(25.9972, -97.1573)
    } else if name.contains("spaceport america") {
        GeoPoint::new(32.9903, -106.9749)
    } else if name.contains("mojave") {
        GeoPoint::new(35.0594, -118.1516)
    } else if name.contains("corn ranch") {
        GeoPoint::new(31.4227, -104.7574)
    } else if name.contains("ronald reagan ballistic") || name.contains("kwajalein") {
        GeoPoint::new(9.0477, 167.7431)
    } else if name.contains("jeju island") {
        GeoPoint::new(33.3846, 126.5535)
    } else if name.contains("naro space") {
        GeoPoint::new(34.4319, 127.5350)
    } else if name.contains("imam khomeini") || name.contains("semnan") {
        GeoPoint::new(35.2340, 53.9210)
    } else if name.contains("andøya") || name.contains("andoya") {
        GeoPoint::new(69.2940, 16.0207)
    } else if name.contains("saxavord") {
        GeoPoint::new(60.8180, -0.7740)
    } else if name.contains("esrange") {
        GeoPoint::new(67.8930, 21.1040)
    } else if name.contains("arnhem space") {
        GeoPoint::new(-12.1900, 136.7800)
    } else if name.contains("woomera") {
        GeoPoint::new(-30.9490, 136.5300)
    } else if name.contains("palmachim") {
        GeoPoint::new(31.8970, 34.6900)
    } else if name.contains("alcântara") || name.contains("alcantara") {
        GeoPoint::new(-2.3730, -44.3960)
    } else {
        return None;
    };

    Some(point)
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
        assert_eq!(launches[0].site_name, "Kennedy Space Center");
        assert_eq!(
            launches[0].coordinates,
            Some(GeoPoint::new(28.5729, -80.6490))
        );
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
        assert!(launches[1].coordinates.is_none());
        assert!(launches[1].t0.is_none());
        assert!(launches[1].weather.is_none());
    }
}
