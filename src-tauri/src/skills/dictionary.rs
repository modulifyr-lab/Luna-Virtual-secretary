use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DefinitionItem {
    definition: String,
    example: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MeaningItem {
    #[serde(rename = "partOfSpeech")]
    part_of_speech: Option<String>,
    definitions: Option<Vec<DefinitionItem>>,
}

#[derive(Debug, Deserialize)]
struct DictionaryEntry {
    word: Option<String>,
    meanings: Option<Vec<MeaningItem>>,
}

pub struct DictionarySkill;

impl DictionarySkill {
    pub async fn lookup_word(word: &str) -> Result<String, String> {
        let clean_word = word.trim().trim_matches(|c: char| !c.is_alphabetic());
        if clean_word.is_empty() {
            return Err("No word provided for dictionary lookup.".to_string());
        }

        let url = format!("https://api.dictionaryapi.dev/api/v2/entries/en/{}", clean_word);

        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to reach dictionary API: {}", e))?;

        if resp.status().as_u16() == 404 {
            return Ok(format!("Sorry, I could not find a definition for '{}'.", clean_word));
        }

        if !resp.status().is_success() {
            return Err(format!("Dictionary API returned status code {}", resp.status()));
        }

        let entries: Vec<DictionaryEntry> = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse dictionary response: {}", e))?;

        if let Some(first_entry) = entries.first() {
            let target_word = first_entry.word.as_deref().unwrap_or(clean_word);
            let mut def_strings = Vec::new();

            if let Some(meanings) = &first_entry.meanings {
                for meaning in meanings {
                    let pos = meaning.part_of_speech.as_deref().unwrap_or("word");
                    if let Some(defs) = &meaning.definitions {
                        if let Some(first_def) = defs.first() {
                            def_strings.push(format!("({}) {}", pos, first_def.definition));
                        }
                    }
                    if def_strings.len() >= 2 {
                        break;
                    }
                }
            }

            if !def_strings.is_empty() {
                Ok(format!("{}: {}", target_word, def_strings.join("; ")))
            } else {
                Ok(format!("Found entry for '{}', but no definitions were listed.", target_word))
            }
        } else {
            Ok(format!("No definitions found for '{}'.", clean_word))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dictionary_empty_word() {
        let res = DictionarySkill::lookup_word("").await;
        assert!(res.is_err());
    }
}
