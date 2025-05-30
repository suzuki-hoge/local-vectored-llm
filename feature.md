# 採用する機能と理由

## インデックス設定

- **採用**: FLAT インデックス
- **理由**:
    - 100 ファイル程度の小規模データセット
    - チャンク数は数百件程度と見込まれる
    - シンプルな実装で十分なパフォーマンスが得られる

## コレクション管理

- **採用**: ディレクトリ構造ベースの分割
    - 解析対象ディレクトリの 2 階層目までで 1 コレクション
    - 空になったコレクションは自動削除
- **理由**:
    - 実装が簡単
    - 既存の構造を活用できる
    - 処理が高速
    - 段階的な改善が可能
    - 適度な粒度での検索が可能

## メタデータ

- **採用**: 3 種のメタデータ
    - ファイル関連
        - ファイルパス
        - 作成日時
        - 最終更新日時
    - チャンク関連
        - チャンクインデックス
    - 検索関連
        - （拡張性のための予約領域）
- **理由**:
    - 差分更新の実装が容易
    - 処理時間の最小化
    - 基本的な検索機能の実現に十分
    - 将来の機能拡張に対応可能

## 差分更新

- **採用**: 3 種の更新処理
    - 追加: 新規ファイル名の場合、ファイル全体を追加
    - 変更: 同一ファイルがあり更新日時が不一致の場合、関連ファイル全体を削除してから全体を追加
    - 削除: 全解析完了後、ディレクトリに存在しなかったファイルの解析結果を削除
- **理由**:
    - 実装が明確
    - 処理が高速
    - 小規模データセットで十分な機能
    - データの整合性が保証される

## 検索最適化

- **採用**: ユーザー指定のコレクション選択
    - 複数コレクションの選択が可能
    - 選択したコレクション内で検索
- **理由**:
    - 検索範囲の柔軟な制御
    - ユーザーの意図を反映
    - 検索精度の向上
    - 実装が比較的シンプル 
