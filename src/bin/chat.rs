use anyhow::Result;
use clap::Parser;
use local_vectored_llm::llm;
use local_vectored_llm::utils;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// インタラクティブモード
    #[arg(short, long)]
    interactive: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    utils::init_logging()?;

    let cli = Cli::parse();

    if cli.interactive {
        println!("チャットを開始します。終了するには 'exit' または 'quit' と入力してください。");
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                break;
            }

            let response = llm::answer(input).await?;
            println!("AI: {}", response);
        }
    } else {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let response = llm::answer(input.trim()).await?;
        println!("AI: {}", response);
    }

    Ok(())
}
