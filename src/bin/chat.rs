use anyhow::Result;
use clap::Parser;
use local_vectored_llm::chroma::ChromaStore;
use local_vectored_llm::ollama::OllamaClient;
use local_vectored_llm::utils::logger;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// インタラクティブモード
    #[arg(short, long)]
    interactive: bool,

    /// 質問
    #[arg(short, long, default_value = "")]
    question: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    logger::init_logging()?;

    let cli = Cli::parse();
    let chroma = ChromaStore::new().await?;
    let ollama = OllamaClient::new();

    if cli.interactive {
        println!("チャットを開始します。終了するには 'exit' または 'quit' と入力してください。");
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                break;
            }

            let context = chroma.search(input, 5).await?;
            let response = ollama.answer(input, &context).await?;
            println!("AI: {}", response);
        }
    } else {
        println!("search context");
        let context = chroma.search(&cli.question, 5).await?;
        println!("found: {:?}", context);
        let response = ollama.answer(&cli.question, &context).await?;
        println!("AI: {}", response);
    }

    Ok(())
}
