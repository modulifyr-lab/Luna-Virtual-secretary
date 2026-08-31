pub struct WeatherSkill;

impl WeatherSkill {
    pub async fn get_forecast(latitude: f64, longitude: f64) -> Result<String, String> {
        // TODO: Query Open-Meteo API endpoint via reqwest (e.g., https://api.open-meteo.com/v1/forecast?latitude=...&longitude=...)
        // TODO: Parse response JSON and format weather summary
        Ok(format!("Weather stub for lat: {}, lon: {}", latitude, longitude))
    }
}
