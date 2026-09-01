use rss::Channel;

pub struct NewsSkill;

impl NewsSkill {
    pub const DEFAULT_FEEDS: &'static [&'static str] = &[
        "https://feeds.bbci.co.uk/news/rss.xml",
        "https://rss.nytimes.com/services/xml/rss/nyt/HomePage.xml",
    ];

    pub async fn fetch_news(feed_url_opt: Option<&str>) -> Result<String, String> {
        let feed_url = feed_url_opt.unwrap_or(Self::DEFAULT_FEEDS[0]);
        let client = reqwest::Client::new();
        let resp = client
            .get(feed_url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch RSS feed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("RSS feed HTTP request failed with status {}", resp.status()));
        }

        let content = resp
            .bytes()
            .await
            .map_err(|e| format!("Failed to read RSS feed bytes: {}", e))?;

        let channel = Channel::read_from(&content[..])
            .map_err(|e| format!("Failed to parse RSS XML: {}", e))?;

        let headlines: Vec<String> = channel
            .items()
            .iter()
            .filter_map(|item| item.title().map(|t| t.trim().to_string()))
            .filter(|t| !t.is_empty())
            .take(5)
            .collect();

        if headlines.is_empty() {
            Ok(format!("No headlines found in news feed from {}.", channel.title()))
        } else {
            Ok(format!(
                "Here are the top headlines from {}: {}",
                channel.title(),
                headlines.join("; ")
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rss_channel_parsing() {
        let xml_data = r#"<?xml version="1.0" encoding="UTF-8"?>
        <rss version="2.0">
            <channel>
                <title>Test News</title>
                <item><title>Headline 1</title></item>
                <item><title>Headline 2</title></item>
            </channel>
        </rss>"#;
        let channel = Channel::read_from(xml_data.as_bytes()).unwrap();
        assert_eq!(channel.title(), "Test News");
        let titles: Vec<&str> = channel.items().iter().filter_map(|i| i.title()).collect();
        assert_eq!(titles, vec!["Headline 1", "Headline 2"]);
    }
}
