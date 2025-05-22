use anyhow::Result;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::models::ModelOptions;
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
            "{}\n{}\n{}\n{}\n{}\n{}\n{}",
            "以下の [質問] に [参考情報] を踏まえ回答せよ",
            "回答内容の「根拠となる情報源・出典」を冒頭に必ず明示すること",
            "[参考情報] が回答の助けにならないと判断した場合は、憶測や不確かな回答を表示せず [与えられたコンテキストからは回答できません] とだけはっきり回答すること",
            "[参考情報]",
            context.join("\n"),
            "[質問]",
            query
        );
        let mut req = GenerationRequest::new("7shi/ezo-gemma-2-jpn:2b-instruct-q8_0".to_string(), prompt);

        // 生成速度と品質のバランスを考慮したオプション設定
        req.options = Some(
            ModelOptions::default()
                // 生成速度の最適化
                // 高 → より創造的な出力になる, 低 → より決定論的な出力になる, default: 0
                .mirostat(0)
                // 高 → 並列処理が増えて高速化, 低 → シングルスレッドで低速, default: 自動検出
                .num_thread(4)
                // 高 → より長い文章を生成, 低 → より短い文章を生成, default: 128
                .num_predict(128)
                // 高 → 低確率トークンの影響を強く抑制, 低 → 低確率トークンも許容, default: 1.0
                .tfs_z(1.0)
                // 出力の決定論性向上
                // 高 → より創造的で多様な出力, 低 → より決定論的で一貫性のある出力, default: 0.8
                .temperature(0.5)
                // 高 → より多様な単語を選択, 低 → より確実な単語を選択, default: 40
                .top_k(20)
                // 高 → より多様な文章を生成, 低 → より確実な文章を生成, default: 0.9
                .top_p(0.7)
                // 固定値 → 同じ入力に対して同じ出力を生成, 0 → ランダム, default: 0
                .seed(42)
                // ハードウェア最適化
                // 高 → より多くのGPUを使用, 低 → より少ないGPUを使用, default: macOS = 1, 他 = 0
                .num_gpu(0)
                // 繰り返し制御
                // 高 → より広い範囲で繰り返しを防止, 低 → より狭い範囲で繰り返しを防止, default: 64
                .repeat_last_n(64)
                // 高 → より強く繰り返しを抑制, 低 → より緩やかに繰り返しを抑制, default: 1.1
                .repeat_penalty(1.1),
        );

        let response = self.client.generate(req).await?;
        Ok(response.response)
    }
}
