pub mod dictionary;
pub mod file_search;
pub mod news;
pub mod office_bridge;
pub mod weather;
pub mod web_search;

pub struct SkillDispatcher;

impl SkillDispatcher {
    pub async fn dispatch(skill_name: &str, params: &str) -> Result<String, String> {
        // TODO: Parse skill_name and delegate to appropriate skill module
        Ok(format!("Skill dispatch stub: {} with params {}", skill_name, params))
    }
}
