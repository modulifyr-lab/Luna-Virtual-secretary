use reqwest::Client;
use std::time::Duration;

/// Performs a fast (1.5s timeout) non-blocking check to verify internet connectivity.
/// Tries hitting Groq's endpoint first, falling back to a known fast endpoint.
pub async fn check_online_status() -> bool {
    let client = match Client::builder().timeout(Duration::from_millis(1500)).build() {
        Ok(c) => c,
        Err(_) => return false,
    };

    if client.head("https://api.groq.com").send().await.is_ok() {
        return true;
    }

    client.head("https://1.1.1.1").send().await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_online_status_runs_fast() {
        let start = std::time::Instant::now();
        let _status = check_online_status().await;
        let elapsed = start.elapsed();
        // Check that connectivity check finishes within 3 seconds even if network is slow/failing
        assert!(elapsed < Duration::from_secs(3));
    }
}
