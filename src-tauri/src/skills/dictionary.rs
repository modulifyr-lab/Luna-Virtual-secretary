pub struct DictionarySkill;

impl DictionarySkill {
    pub async fn lookup_word(word: &str) -> Result<String, String> {
        // TODO: Query https://api.dictionaryapi.dev/api/v2/entries/en/<word> via reqwest
        // TODO: Parse definition, phonetics, and examples
        Ok(format!("Dictionary stub for word: {}", word))
    }
}
