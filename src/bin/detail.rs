use anyhow::Result;
use clap::Parser;
use local_vectored_llm::chroma::store::ChromaStore;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arg {
    /// コレクション名
    #[arg(short, long)]
    collection: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arg::parse();
    let chroma = ChromaStore::new().await?;

    let documents = chroma.get_collection_documents(&args.collection).await?;

    for (i, doc) in documents.iter().enumerate() {
        println!("{:<15} | {}", "No.", i + 1);
        println!("{:<15} | {}", "ID", doc.id);
        println!("{:<15} | {}", "Body", doc.content.chars().take(80).collect::<String>().replace("\n", " "));
        println!("{:<15} | {}", "file.path", &doc.metadata.file.path);
        println!("{:<15} | {}", "file.created_at", &doc.metadata.file.created_at);
        println!("{:<15} | {}", "file.updated_at", &doc.metadata.file.updated_at);
        println!("{:<15} | {}", "chunk.index", &doc.metadata.chunk.index);
        println!("{}", "-".repeat(80));
    }

    Ok(())
}
