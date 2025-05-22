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

```bash
$ rustup install nightly
```

## 使用方法

### ファイルのベクトル化

ローカルファイルをベクトル化して Chroma DB に保存するには、以下のコマンドを実行します。

```bash
$ ./dist/load --input <dir-path>
```

実行例

```
$ ./dist/load --input ~/Documents/tmp/llm-input

[2025-05-22 14:30:33] INFO Converted: test/基本情報技術者試験　科目B　サンプル問題　解答例.pdf
[2025-05-22 14:30:33] INFO Converted: 健康度.md
[2025-05-22 14:30:33] INFO Converted: health-care/service-spec/ヘルスケアマネージャ仕様書.txt
[2025-05-22 14:30:33] INFO Converted: health-care/db-spec/DB仕様書.txt
[2025-05-22 14:30:33] INFO Converted: health-care/api-spec/API仕様書.txt
[2025-05-22 14:30:50] INFO [ 1 / 5 ] Saved: test/基本情報技術者試験　科目B　サンプル問題　解答例.pdf-0
[2025-05-22 14:30:56] INFO [ 2 / 5 ] Saved: 健康度.md-0
[2025-05-22 14:31:02] INFO [ 2 / 5 ] Saved: 健康度.md-1
[2025-05-22 14:31:06] INFO [ 2 / 5 ] Saved: 健康度.md-2
[2025-05-22 14:31:12] INFO [ 3 / 5 ] Saved: health-care/service-spec/ヘルスケアマネージャ仕様書.txt-0
[2025-05-22 14:31:13] INFO [ 4 / 5 ] Saved: health-care/db-spec/DB仕様書.txt-0
[2025-05-22 14:31:14] INFO [ 5 / 5 ] Saved: health-care/api-spec/API仕様書.txt-0
[2025-05-22 14:31:14] INFO Processed: success = 7, failure = 0
```

対象ディレクトリの 2 層目までをコレクション名とします。

### クエリの実行

保存されたベクトルを使用して質問に回答するには、以下のコマンドを実行します。

```bash
$ ./dist/chat -- --question <question>
```

実行例

```
$ ./dist/chat --question 'DBMS は何？'

1. health-care-api-spec           ( 1 documents )
2. health-care-db-spec            ( 1 documents )
3. health-care-service-spec       ( 1 documents )
4. root                           ( 13 documents )
5. test                           ( 1 documents )

Choose the collection you want to use ( e.g. 1,3,4 ): 2
[2025-05-22 14:36:21] INFO Search context... ( from [ health-care-db-spec ] )
[2025-05-22 14:36:31] INFO Found 1 contexts: [ # DB 仕様書MySQL を使います... ]
[2025-05-22 14:36:31] INFO Wait response generation...

回答：MySQLを用いたデータベース管理システム（DBMS）である。

根拠となる情報源・出典：参考情報「DB 仕様書」


[2025-05-22 14:36:39] INFO Complete
```

### コレクションの確認

コレクション一覧を表示するには、以下のコマンドを実行します。

```bash
$ ./dist/list
```

実行例

```
$ ./dist/list

name                           | data count
-------------------------------+-----------
health-care-api-spec           | 1
health-care-db-spec            | 1
health-care-service-spec       | 1
root                           | 13
test                           | 1
```

### コレクションの中身の確認

コレクションのドキュメントを表示するには、以下のコマンドを実行します。

```bash
$ ./dist/detail --collection <collection-name>
```

実行例

```
$ ./dist/detail --collection root

No.             | 1
ID              | 健康度.md-0
Body            | # HealthPointShell 関数分析レポート  ## 公開関数  ### `_welcome()` - **呼び出し関係**:   - 呼んでいる H...
file.path       | 健康度.md
file.created_at | 2025-05-16 02:01:13 UTC
file.updated_at | 2025-05-16 02:01:13 UTC
chunk.index     | 0
----------------+------------------------------------------------------------------
No.             | 2
ID              | 健康度.md-1
Body            | call()` から間接的に呼び出し  ### `exercise()` - **呼び出し関係**:   - 呼ん...
file.path       | 健康度.md
file.created_at | 2025-05-16 02:01:13 UTC
file.updated_at | 2025-05-16 02:01:13 UTC
chunk.index     | 1
----------------+------------------------------------------------------------------
No.             | 3
:
:
```

## サポートされているファイル形式

- `.txt`
- `.pdf`
- `.md`

## ほか

Ollama CLI を直接操作するには、以下のコマンドを実行します。

```bash
$ docker compose exec ollama ollama run 7shi/ezo-gemma-2-jpn:2b-instruct-q8_0
```
