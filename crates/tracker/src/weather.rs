//! Weather information module for statusline
//!
//! Fetches weather data from Taiwan's Central Weather Administration (CWA) API
//! and caches results to minimize API calls.

use crate::config::WeatherSettings;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// Weather information to display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherInfo {
    /// Temperature in Celsius
    pub temp: i32,
    /// Weather condition (e.g., 晴, 多雲, 雨)
    pub condition: String,
    /// Probability of precipitation (0-100%)
    pub pop: Option<u8>,
}

/// Cached weather data with timestamp
#[derive(Debug, Serialize, Deserialize)]
struct CachedWeather {
    data: WeatherInfo,
    #[serde(with = "chrono::serde::ts_seconds")]
    cached_at: DateTime<Utc>,
}

/// Get weather information, using cache if valid
pub async fn get_weather(config: &WeatherSettings) -> Option<WeatherInfo> {
    // Check if API key is configured
    let api_key = config.api_key.as_ref()?;

    // Try cache first
    if let Some(cached) = read_weather_cache(config.cache_ttl_minutes) {
        return Some(cached);
    }

    // Cache miss or expired, fetch from API
    match fetch_from_cwa(api_key, &config.location).await {
        Ok(info) => {
            write_weather_cache(&info);
            Some(info)
        }
        Err(_) => {
            // On error, try to use stale cache as fallback
            read_weather_cache_stale()
        }
    }
}

/// Get cache file path (uses ~/.cache for consistency with other caches)
fn get_cache_path() -> PathBuf {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".cache")
        .join("claude-time-tracker");

    // Create cache directory if needed
    let _ = fs::create_dir_all(&cache_dir);

    cache_dir.join("weather.json")
}

/// Read weather from cache if within TTL
fn read_weather_cache(ttl_minutes: u32) -> Option<WeatherInfo> {
    let cache_path = get_cache_path();
    let content = fs::read_to_string(&cache_path).ok()?;
    let cached: CachedWeather = serde_json::from_str(&content).ok()?;

    // Check if cache is still valid
    let age_seconds = (Utc::now() - cached.cached_at).num_seconds();
    let ttl_seconds = (ttl_minutes as i64) * 60;

    if age_seconds >= 0 && age_seconds < ttl_seconds {
        Some(cached.data)
    } else {
        None
    }
}

/// Read stale cache as fallback when API fails
fn read_weather_cache_stale() -> Option<WeatherInfo> {
    let cache_path = get_cache_path();
    let content = fs::read_to_string(&cache_path).ok()?;
    let cached: CachedWeather = serde_json::from_str(&content).ok()?;
    Some(cached.data)
}

/// Write weather data to cache
fn write_weather_cache(info: &WeatherInfo) {
    let cache_path = get_cache_path();
    let cached = CachedWeather {
        data: info.clone(),
        cached_at: Utc::now(),
    };

    if let Ok(json) = serde_json::to_string(&cached) {
        let _ = fs::write(&cache_path, json);
    }
}

/// Fetch weather from CWA Open Data API
async fn fetch_from_cwa(api_key: &str, location: &str) -> Result<WeatherInfo, WeatherError> {
    // API endpoint for Taipei City townships (F-D0047-061)
    // Note: locationName filter may not work properly, so we fetch all and filter locally
    let url = format!(
        "https://opendata.cwa.gov.tw/api/v1/rest/datastore/F-D0047-061?Authorization={}",
        api_key
    );

    // Create client with timeout (5 seconds for large API response)
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|_| WeatherError::NetworkError)?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|_| WeatherError::NetworkError)?;

    if !response.status().is_success() {
        return Err(WeatherError::ApiError);
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|_| WeatherError::ParseError)?;

    parse_cwa_response(&json, location)
}

/// Parse CWA API response to extract weather info
fn parse_cwa_response(json: &serde_json::Value, target_location: &str) -> Result<WeatherInfo, WeatherError> {
    // Navigate to the locations array
    // CWA uses PascalCase keys: records -> Locations[0] -> Location[]
    let locations = json
        .get("records")
        .and_then(|r| r.get("Locations"))
        .and_then(|l| l.get(0))
        .and_then(|l| l.get("Location"))
        .and_then(|l| l.as_array())
        .ok_or(WeatherError::ParseError)?;

    // Find the target location by name
    let location = locations
        .iter()
        .find(|loc| {
            loc.get("LocationName")
                .and_then(|n| n.as_str())
                .map(|n| n == target_location)
                .unwrap_or(false)
        })
        .ok_or(WeatherError::ParseError)?;

    let weather_elements = location
        .get("WeatherElement")
        .and_then(|e| e.as_array())
        .ok_or(WeatherError::ParseError)?;

    let mut temp: Option<i32> = None;
    let mut condition: Option<String> = None;
    let mut pop: Option<u8> = None;

    for element in weather_elements {
        let element_name = element
            .get("ElementName")
            .and_then(|n| n.as_str())
            .unwrap_or("");

        // Get the first time period's value
        // Structure: Time[0] -> ElementValue[0] -> value (or Temperature for 溫度)
        let time_data = element.get("Time").and_then(|t| t.get(0));

        match element_name {
            "溫度" => {
                // Temperature - uses "Temperature" key instead of "ElementValue"
                if let Some(v) = time_data
                    .and_then(|t| t.get("ElementValue"))
                    .and_then(|ev| ev.get(0))
                    .and_then(|ev| ev.get("Temperature"))
                    .and_then(|v| v.as_str())
                {
                    temp = v.parse().ok();
                }
            }
            "天氣現象" => {
                // Weather condition
                if let Some(v) = time_data
                    .and_then(|t| t.get("ElementValue"))
                    .and_then(|ev| ev.get(0))
                    .and_then(|ev| ev.get("Weather"))
                    .and_then(|v| v.as_str())
                {
                    condition = Some(v.to_string());
                }
            }
            "3小時降雨機率" => {
                // 3-hour probability of precipitation
                if let Some(v) = time_data
                    .and_then(|t| t.get("ElementValue"))
                    .and_then(|ev| ev.get(0))
                    .and_then(|ev| ev.get("ProbabilityOfPrecipitation"))
                    .and_then(|v| v.as_str())
                {
                    pop = v.parse().ok();
                }
            }
            _ => {}
        }
    }

    Ok(WeatherInfo {
        temp: temp.ok_or(WeatherError::ParseError)?,
        condition: condition.ok_or(WeatherError::ParseError)?,
        pop,
    })
}

/// Weather fetch errors
#[derive(Debug)]
enum WeatherError {
    NetworkError,
    ApiError,
    ParseError,
}

/// Format weather for statusline display
pub fn format_weather(w: &WeatherInfo) -> String {
    const ICON_RAINDROP: &str = "\u{e371}";  //  (raindrop/humidity)

    let weather_icon = get_weather_icon(&w.condition);

    if let Some(pop) = w.pop {
        format!("{}{}° {}{}%", weather_icon, w.temp, ICON_RAINDROP, pop)
    } else {
        format!("{}{}°", weather_icon, w.temp)
    }
}

/// Get appropriate weather icon based on condition
fn get_weather_icon(condition: &str) -> &'static str {
    // Nerd Font weather icons
    const ICON_SUNNY: &str = "\u{e30d}";     //  (sunny)
    const ICON_CLOUDY: &str = "\u{e312}";    //  (cloudy)
    const ICON_RAIN: &str = "\u{e318}";      //  (rain)
    const ICON_THUNDER: &str = "\u{e31d}";   //  (thunderstorm)
    const ICON_FOG: &str = "\u{e313}";       //  (fog)

    if condition.contains("雷") {
        ICON_THUNDER
    } else if condition.contains("雨") {
        ICON_RAIN
    } else if condition.contains("陰") || condition.contains("雲") {
        ICON_CLOUDY
    } else if condition.contains("霧") || condition.contains("霾") {
        ICON_FOG
    } else {
        // 晴, 多雲時晴, etc.
        ICON_SUNNY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_icon_selection() {
        assert_eq!(get_weather_icon("晴"), "\u{e30d}");
        assert_eq!(get_weather_icon("多雲"), "\u{e312}");
        assert_eq!(get_weather_icon("陰天"), "\u{e312}");
        assert_eq!(get_weather_icon("雨"), "\u{e318}");
        assert_eq!(get_weather_icon("短暫雨"), "\u{e318}");
        assert_eq!(get_weather_icon("雷陣雨"), "\u{e31d}");
    }

    #[test]
    fn test_format_weather() {
        let w = WeatherInfo {
            temp: 25,
            condition: "晴".to_string(),
            pop: Some(10),
        };
        // Format: {weather_icon}{temp}° {raindrop_icon}{pop}%
        assert_eq!(format_weather(&w), "\u{e30d}25° \u{e371}10%");

        let w_rain = WeatherInfo {
            temp: 18,
            condition: "短暫雨".to_string(),
            pop: Some(60),
        };
        assert_eq!(format_weather(&w_rain), "\u{e318}18° \u{e371}60%");

        let w_no_pop = WeatherInfo {
            temp: 20,
            condition: "晴".to_string(),
            pop: None,
        };
        assert_eq!(format_weather(&w_no_pop), "\u{e30d}20°");
    }

    /// Integration test for CWA API - run with: cargo test test_cwa_api_integration -- --ignored --nocapture
    #[tokio::test]
    #[ignore] // Requires real API key
    async fn test_cwa_api_integration() {
        let config = WeatherSettings {
            api_key: Some("CWA-CE93342C-56C3-44C4-9DC9-C0CE76DDC846".to_string()),
            location: "北投區".to_string(),
            cache_ttl_minutes: 60,
        };

        // Clear cache first
        let cache_path = get_cache_path();
        let _ = fs::remove_file(&cache_path);

        let result = get_weather(&config).await;
        println!("Weather result: {:?}", result);

        assert!(result.is_some(), "Should fetch weather successfully");
        let info = result.unwrap();
        println!("Temperature: {}°C", info.temp);
        println!("Condition: {}", info.condition);
        println!("PoP: {:?}%", info.pop);
        println!("Formatted: {}", format_weather(&info));

        // Check cache was written
        assert!(cache_path.exists(), "Cache file should exist");
    }
}
