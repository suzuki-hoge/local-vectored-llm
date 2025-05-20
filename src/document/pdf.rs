use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn extract_text(path: &Path) -> Result<String> {
    // 一時ディレクトリの作成
    let temp_dir =
        tempfile::Builder::new().prefix("pdf_images_").tempdir().context("Failed to create temporary directory")?;

    let temp_dir_path = temp_dir.path().to_str().unwrap();
    let pdf_path = path.to_str().unwrap();

    // PDFを画像に変換
    let output = Command::new("pdftoppm")
        .args(["-png", pdf_path, &format!("{}/page", temp_dir_path)])
        .output()
        .context("Failed to convert PDF to images")?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("PDF to image conversion failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    // 変換された画像からテキストを抽出
    let mut all_text = String::new();
    let mut page_num = 1;

    loop {
        let image_path = format!("{}/page-{}.png", temp_dir_path, page_num);
        if !std::path::Path::new(&image_path).exists() {
            break;
        }

        let output = Command::new("tesseract")
            .args([&image_path, "stdout", "-l", "jpn"])
            .output()
            .context("Failed to perform OCR")?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("OCR failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let page_text = String::from_utf8(output.stdout).context("Failed to decode OCR output")?;

        all_text.push_str(&page_text);
        all_text.push('\n');

        page_num += 1;
    }

    Ok(all_text)
}
