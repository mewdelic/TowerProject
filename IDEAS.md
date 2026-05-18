# TowerProject - Ideas

## 基本コンセプト
- SimTowerのルネッサンス
- CLIをメインに据える
- LLM/Agentがプレイできるようにする（特に重要）

## アーキテクチャ案

### 1. Core（Backend）
- 純粋なゲームロジック
- 状態はJSONで完全に出力可能
- Agentが呼びやすいアクションAPI
- 例: `build`, `place_elevator`, `advance`, `get_state`

### 2. Frontends
- TUI版（ratatui / textual）
- Headless / Agent版（JSONのみ）
- Spectator / Replay版

## バズらせるためのアイデア

### Agent活用
- MewなどのAgentが勝手にタワーを建てる
- 面白いタワーができたら自動でX投稿
- 「Agentが作ったタワー」として差別化

### 共有・観戦文化
- プレイログを簡単にシェアできる
- テキストでタワーの様子を表現
- 「他人のタワーを眺める」楽しさ

### 中毒性
- 短時間で結果が出る（30〜60分）
- 失敗が面白い
- 毎日1タワー作れる軽さ

## 技術選定候補
- Rust + ratatui（パフォーマンスと単一バイナリ）
- Python（Agent連携が楽）
- Go（シンプルさ重視）

## 未定のポイント
- どこまで本格的にするのか
- テナントのAI度合い
- マネタイズは一切考えない（趣味）
