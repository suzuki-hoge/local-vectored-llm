use anyhow::Result;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;

pub struct OllamaClient {
    client: Ollama,
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new()
    }
}

impl OllamaClient {
    pub fn new() -> Self {
        Self { client: Ollama::new("http://localhost", 11434) }
    }

    pub async fn answer(&self, query: &str, context: &[String]) -> Result<String> {
        let prompt = format!(
            "以下の情報を参考に質問に答えてください。\n\n[参考情報]\n{}\n\n[質問]\n{}",
            context.join("\n"),
            query
        );
        let req = GenerationRequest::new("7shi/ezo-gemma-2-jpn:2b-instruct-q8_0".to_string(), prompt);
        let response = self.client.generate(req).await?;
        Ok(response.response)
    }
}
