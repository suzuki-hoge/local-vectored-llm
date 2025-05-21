# ローカルファイルベクトル化 RAG システム

このプロジェクトは、ローカルファイルをベクトル化して Chroma DB に保存し、Deepseek LLM を使用した RAG ( Retrieval-Augmented
Generation ) システムを提供します。

## 構成

- Rust アプリケーション ( 実行ファイルあり )
- Docker コンテナの Ollama ( + Gemini2 )
- Docker コンテナの Chroma DB

## セットアップ

### 前提条件

- PDF 解析ツール Poppler がインストールされていること
- Docker と Docker Compose がインストールされていること
- Rust がインストールされていること ( ビルドする場合のみ )

### Poppler のインストール ( mac OS )

```
$ brew install tesseract
$ brew install tesseract-lang
$ brew install poppler
```

### Docker コンテナの起動

```bash
$ docker compose up --detach
```

### Rust の nightly セットアップ ( ビルドする場合のみ )

```
$ rustup install nightly
```

## 使用方法

### ファイルのベクトル化

ローカルファイルをベクトル化して Chroma DB に保存するには、以下のコマンドを実行します。

```bash
$ cargo run --bin load -- --input <dir-path>
```

もしくは

```bash
$ ./dist/load --input <dir-path>
```

### クエリの実行

保存されたベクトルを使用して質問に回答するには、以下のコマンドを実行します。

```bash
$ cargo run --bin chat -- --question <question>
```

もしくは

```bash
$ ./dist/chat -- --question <question>
```

## サポートされているファイル形式

- `.txt`
- `.pdf`
- `.md`

## ほか

Ollama CLI を直接操作するには、以下のコマンドを実行します。

```
$ docker compose exec ollama ollama run 7shi/ezo-gemma-2-jpn:2b-instruct-q8_0
```
