use anyhow::Result;
use clap::Parser;
use local_vectored_llm::chroma::ChromaStore;

#[tokio::main]
async fn main() -> Result<()> {
    let chroma = ChromaStore::new().await?;

    let collections = chroma.get_collections().await?;

    println!("{:<30} | {:<10}", "name", "data count");
    println!("{}-+-{}", "-".repeat(30), "-".repeat(10));

    for collection in collections {
        println!("{:<30} | {:<10}", 
            collection.name,
            collection.count
        );
    }

    Ok(())
} 