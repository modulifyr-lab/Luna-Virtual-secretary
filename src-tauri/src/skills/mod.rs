pub mod app_launch;
pub mod dictionary;
pub mod file_search;
pub mod news;
pub mod office_bridge;
pub mod weather;
pub mod web_search;

use app_launch::AppLaunch;
use dictionary::DictionarySkill;
use file_search::FileSearch;
use news::NewsSkill;
use office_bridge::OfficeBridge;
use weather::WeatherSkill;
use web_search::WebSearchSkill;

pub struct SkillDispatcher;

impl SkillDispatcher {
    /// Attempts to match user prompt to a skill intent. Returns Some(Result<String, String>) if matched, or None if prompt should fall back to LLM.
    pub async fn try_dispatch(prompt: &str) -> Option<Result<String, String>> {
        let trimmed = prompt.trim();
        let lower = trimmed.to_lowercase();

        // 1. Weather intent
        if lower.contains("weather") || lower.contains("forecast") || lower.contains("temperature outside") {
            return Some(WeatherSkill::get_forecast(None, None).await);
        }

        // 2. Dictionary intent ("define [word]", "what is the definition of [word]", "meaning of [word]")
        if lower.starts_with("define ") || lower.contains("definition of") || lower.contains("meaning of") {
            let word = if let Some(rest) = lower.strip_prefix("define ") {
                rest
            } else if let Some(idx) = lower.find("definition of ") {
                &lower[idx + "definition of ".len()..]
            } else if let Some(idx) = lower.find("meaning of ") {
                &lower[idx + "meaning of ".len()..]
            } else {
                trimmed
            };
            return Some(DictionarySkill::lookup_word(word).await);
        }

        // 3. News intent ("any news", "news", "headlines", "latest news")
        if lower.contains("news") || lower.contains("headlines") {
            return Some(NewsSkill::fetch_news(None).await);
        }

        // 4. File search intent ("find file", "find [x]", "search file", "search for file")
        if lower.starts_with("find file ") || lower.starts_with("find ") || lower.starts_with("search file ") {
            let term = if lower.starts_with("find file ") {
                &trimmed[10..]
            } else if lower.starts_with("find ") {
                &trimmed[5..]
            } else if lower.starts_with("search file ") {
                &trimmed[12..]
            } else {
                trimmed
            };
            return Some(FileSearch::search(term));
        }

        // 5. Office intent ("open word document", "create word document", "open powerpoint", "draft email", "outlook", etc.)
        if lower.contains("word document") || lower.contains("open word") || lower.contains("powerpoint") || lower.contains("outlook") || lower.contains("draft email") {
            let bridge = OfficeBridge::new("python-bridge/office_control.py");
            let (app, action) = if lower.contains("powerpoint") {
                ("powerpoint", "create_presentation")
            } else if lower.contains("outlook") || lower.contains("email") {
                ("outlook", "draft_email")
            } else {
                ("word", "create_doc")
            };
            return Some(bridge.execute(app, action, "{}"));
        }

        // 6. App launch intent ("open [app]", "launch [app]", "start [app]")
        if lower.starts_with("open ") || lower.starts_with("launch ") || lower.starts_with("start ") {
            let app_name = if lower.starts_with("open ") {
                &trimmed[5..]
            } else if lower.starts_with("launch ") {
                &trimmed[7..]
            } else if lower.starts_with("start ") {
                &trimmed[6..]
            } else {
                trimmed
            };
            return Some(AppLaunch::launch_app(app_name));
        }

        // 7. Web search intent ("search for [x]", "google [x]", "search web for [x]")
        if lower.starts_with("search for ") || lower.starts_with("search web ") || lower.starts_with("search ") {
            let query = if lower.starts_with("search for ") {
                &trimmed[11..]
            } else if lower.starts_with("search web ") {
                &trimmed[11..]
            } else if lower.starts_with("search ") {
                &trimmed[7..]
            } else {
                trimmed
            };
            return Some(WebSearchSkill::search(query).await);
        }

        None
    }

    pub async fn dispatch(skill_name: &str, params: &str) -> Result<String, String> {
        match skill_name.to_lowercase().as_str() {
            "weather" => WeatherSkill::get_forecast(None, None).await,
            "dictionary" => DictionarySkill::lookup_word(params).await,
            "news" => NewsSkill::fetch_news(None).await,
            "file_search" => FileSearch::search(params),
            "office" => {
                let bridge = OfficeBridge::new("python-bridge/office_control.py");
                bridge.execute("word", "create_doc", params)
            }
            "app_launch" | "launch_app" | "open_app" => AppLaunch::launch_app(params),
            "web_search" => WebSearchSkill::search(params).await,
            _ => Err(format!("Unknown skill name: {}", skill_name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_try_dispatch_weather() {
        let res = SkillDispatcher::try_dispatch("what's the weather today").await;
        assert!(res.is_some());
    }

    #[tokio::test]
    async fn test_try_dispatch_dictionary() {
        let res = SkillDispatcher::try_dispatch("define serendipity").await;
        assert!(res.is_some());
    }

    #[tokio::test]
    async fn test_try_dispatch_news() {
        let res = SkillDispatcher::try_dispatch("is there any news").await;
        assert!(res.is_some());
    }

    #[tokio::test]
    async fn test_try_dispatch_file_search() {
        let res = SkillDispatcher::try_dispatch("find file report.docx").await;
        assert!(res.is_some());
    }

    #[tokio::test]
    async fn test_try_dispatch_office() {
        let res = SkillDispatcher::try_dispatch("open a new word document").await;
        assert!(res.is_some());
    }

    #[tokio::test]
    async fn test_try_dispatch_app_launch() {
        let res1 = SkillDispatcher::try_dispatch("Open Chrome").await;
        assert!(res1.is_some());
        let res2 = SkillDispatcher::try_dispatch("LAUNCH Discord").await;
        assert!(res2.is_some());
    }

    #[tokio::test]
    async fn test_try_dispatch_web_search() {
        let res = SkillDispatcher::try_dispatch("search for rust language").await;
        assert!(res.is_some());
    }

    #[tokio::test]
    async fn test_try_dispatch_unmatched() {
        let res = SkillDispatcher::try_dispatch("tell me a funny joke").await;
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn test_dispatch_explicit() {
        let res = SkillDispatcher::dispatch("dictionary", "hello").await;
        assert!(res.is_ok());
    }
}
