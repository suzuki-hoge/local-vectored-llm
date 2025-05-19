use anyhow::Result;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;

/// クエリに対してRAGで回答を返す
pub async fn answer(query: &str) -> Result<String> {
    // 1. 関連ドキュメントを検索（ここはダミー実装）
    let retrieved = retrieve_documents(query).await;

    // 2. 検索結果とクエリをLLMに渡して生成
    let ollama = Ollama::new("http://localhost:11434", 11434);
    let prompt = format!("以下の情報を参考に質問に答えてください。\n\n[参考情報]\n{}\n\n[質問]\n{}", retrieved, query);
    let req = GenerationRequest::new("llama2".to_string(), prompt);
    let response = ollama.generate(req).await?;
    Ok(response.response)
}

/// ダミーのドキュメント検索関数
async fn retrieve_documents(_query: &str) -> String {
    // TODO: 実際の検索ロジックに置き換える
    "これはダミーの参考情報です。".to_string()
}
