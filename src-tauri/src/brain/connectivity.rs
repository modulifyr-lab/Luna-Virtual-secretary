use reqwest::Client;
use std::time::Duration;

pub async fn check_online_status() -> bool {
    // TODO: Perform fast short-timeout HTTP GET/HEAD ping to verify internet connectivity
    let client = Client::builder()
        .timeout(Duration::from_millis(1500))
        .build();

    if let Ok(client) = client {
        client.head("https://1.1.1.1").send().await.is_ok()
    } else {
        false
    }
}
