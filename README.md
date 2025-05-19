# ローカルファイルベクトル化 RAG システム

このプロジェクトは、ローカルファイルをベクトル化して Chroma DB に保存し、Deepseek LLM を使用した RAG ( Retrieval-Augmented Generation ) システムを提供します。

## 構成

- Rust アプリケーション ( 実行ファイルあり )
- Docker コンテナの Ollama, Deepseek
- Docker コンテナの Chroma DB

## セットアップ

### 前提条件

- Rust がインストールされていること ( ビルドする場合のみ )
- Docker と Docker Compose がインストールされていること

### Rust ( ビルドする場合のみ )

```
$ rustup install nightly
```

### Docker コンテナの起動

```bash
$ docker compose up --detach
```

これにより Ollama サーバー ( Deepseek モデル付き ) と Chroma DB サーバーが起動します。

## 使用方法

### ファイルのベクトル化

ローカルファイルをベクトル化して Chroma DB に保存するには、以下のコマンドを実行します。

```bash
$ cargo run --bin vectorize <ディレクトリパス>
```

もしくは

```bash
$ ./dist/vectorize <ディレクトリパス>
```

### クエリの実行

保存されたベクトルを使用して質問に回答するには、以下のコマンドを実行します。

```bash
$ cargo run --bin query "あなたの質問"
```

もしくは

```bash
$ ./dist/query "あなたの質問"
```

## サポートされているファイル形式

- テキストファイル ( `.txt` )
- PDF ファイル ( `.pdf` )
