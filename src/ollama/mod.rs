use crate::info;
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
            "{}\n{}\n{}\n{}\n{}\n{}",
            "以下の [質問] に [参考情報] を踏まえ回答せよ",
            "回答内容の「根拠となる情報源・出典」を明示すること",
            "[参考情報]",
            context.join("\n"),
            "[質問]",
            query
        );
        let req = GenerationRequest::new("7shi/ezo-gemma-2-jpn:2b-instruct-q8_0".to_string(), prompt);

        info!("Wait response generation...");

        let response = self.client.generate(req).await?;
        Ok(response.response)
    }
}
