use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CurrentWeather {
    temperature: f64,
    windspeed: f64,
    weathercode: i32,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current_weather: Option<CurrentWeather>,
}

pub struct WeatherSkill;

impl WeatherSkill {
    /// Interpret Open-Meteo weather code into natural description.
    fn interpret_weather_code(code: i32) -> &'static str {
        match code {
            0 => "clear sky",
            1..=3 => "partly cloudy",
            45 | 48 => "foggy",
            51..=55 => "drizzling",
            61..=65 => "raining",
            71..=75 => "snowing",
            80..=82 => "rain showers",
            95..=99 => "thunderstorms",
            _ => "variable conditions",
        }
    }

    pub async fn get_forecast(lat_opt: Option<f64>, lon_opt: Option<f64>) -> Result<String, String> {
        // Default to London (51.5074, -0.1278) if no coordinates provided
        let lat = lat_opt.unwrap_or(51.5074);
        let lon = lon_opt.unwrap_or(-0.1278);

        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true",
            lat, lon
        );

        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch weather from Open-Meteo: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Open-Meteo HTTP request failed with status {}", resp.status()));
        }

        let body: OpenMeteoResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse weather JSON: {}", e))?;

        if let Some(cw) = body.current_weather {
            let desc = Self::interpret_weather_code(cw.weathercode);
            Ok(format!(
                "The current weather is {} with a temperature of {:.1}°C and wind speed of {:.1} km/h.",
                desc, cw.temperature, cw.windspeed
            ))
        } else {
            Err("No current weather data found in Open-Meteo response.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_weather_code_interpretation() {
        assert_eq!(WeatherSkill::interpret_weather_code(0), "clear sky");
        assert_eq!(WeatherSkill::interpret_weather_code(61), "raining");
    }
}
