# BabelSim (TowerProject)

SimTowerの精神を現代に蘇らせる、Agentフレンドリーなタワーシミュレーション。

## コンセプト
- 超高層ビル建設・運営シミュレーション
- LLM / Agent が直接プレイ・実験可能
- CLI / Headless / 将来的にWeb (WASM) 対応
- ゼロコスト・中毒性重視・面白い失敗を許容

## 現在のステータス
- アイデア段階 → コア設計中
- Rust + 最小限の状態管理でスタート

## アーキテクチャ

### babelsim-core (最優先)
- 純粋なゲームロジック
- 状態は常にJSONで完全シリアライズ可能
- Agentが叩きやすいアクションAPI
- 決定論的シミュレーション（再現性重視）

### 計画されているフロントエンド
- **CLI / TUI**: ratatuiベースのテキストインターフェース
- **Headless**: JSON入出力のみ（Agent専用）
- **Web**: WASM + Leptos/Dioxus（後回し）

## 主要メカニクス（SimTower参考）

### フロアタイプ
- Office（オフィス）
- Hotel
- Restaurant / Cafe
- Retail / Shop
- Residential
- Lobby / Observatory（特別）

### 輸送システム
- エレベーター（シャフト単位で複数台配置）
- キャパシティ・速度・待ち時間のトレードオフ
- 階段・エスカレーター（補助）

### コアループ
1. 時間進行（1分〜1日単位）
2. 住人/来訪者の移動シミュレーション
3. 満足度・収益計算
4. イベント（火事、渋滞、VIP来訪など）

### Agent向け設計ポイント
- `get_state()` → 完全なタワー状態JSON
- `build_floor(type, level)`
- `place_elevator(shaft, floors)`
- `advance(minutes)`
- メトリクス: 利益、交通量、満足度スコア

## 開発優先順位
1. 最小コア（状態 + 基本アクション）
2. シンプルなヘッドレスCLI
3. 満足度・経済モデルの実装
4. TUI版
5. Agentが自動でタワーを作るデモ

## 実行方法（予定）
```bash
cargo run --bin babelsim-cli
```

## 参考
- SimTower (Yoot Saito, 1994)
- 現代的再解釈: 交通工学 + 経済シミュレーション + Agent実験場

リサーチ継続中。面白いメカニクス見つけたら随時更新。
