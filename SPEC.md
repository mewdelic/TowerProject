# BabelSim Implementation Spec

このドキュメントはBabelSim（TowerProject）の実装に必要な情報を網羅的にまとめたもの。
リサーチ完了時点での設計情報。

## 1. アイテム / オブジェクト一覧

### フロアタイプ（Floor Types）
- **Office**: 基本収益源。労働者が出入り。
- **Hotel**: 宿泊客。夜間利用多め。
- **Restaurant**: 食事需要。満足度向上。
- **Retail / Shop**: 買い物需要。
- **Residential**: 住人。安定した家賃収入。
- **Lobby**: 必須。エントランス。
- **Observatory**: 高層階向け特別フロア。
- **Parking**: 将来的拡張。

### エレベーター関連
- Elevator Shaft（シャフト）
- Elevator Car（1シャフトに複数台可能）
- Stairs / Escalator（補助移動手段）

### その他オブジェクト
- People（住人・来訪者・労働者）
- Events（火事、停電、VIP訪問、渋滞）

## 2. ゲームプロセス / メインループ

### 基本サイクル
1. **時間進行** (`advance(minutes)`)
   - シミュレーション時間を進める
2. **人移動フェーズ**
   - 各人が目的地に向かう
   - エレベーター呼び出し・乗降
3. **需要計算**
   - 各フロアの利用需要を算出
4. **満足度更新**
   - 移動時間・待ち時間・施設利用でスコアリング
5. **収益計算**
   - 家賃・利用料収入 - 維持費
6. **イベント発生**
   - ランダムまたは条件付きイベント

### 1ティックあたりの処理
- 最小単位: 1分
- 推奨: 5分 or 15分単位で高速シミュレーション可能

## 3. 主なシステム

### Traffic System（交通システム）
- 人々の移動経路計算
- エレベーター割り当てアルゴリズム
- ボトルネック検出

### Economy System（経済システム）
- 建設コスト
- 維持費
- 収入（家賃・利用料）
- キャッシュフロー管理

### Satisfaction System（満足度システム）
- 移動時間ペナルティ
- 施設アクセシビリティ
- 混雑度影響
- 総合スコア → 人気・収益に影響

### Event System
- 災害（火事・地震）
- 特殊イベント（コンサート・メンテナンス）

## 4. アルゴリズム

### エレベーター割り当て（Elevator Dispatch）
- 基本: 最も近い空きエレベーター
- 改善案:
  - 方向性考慮（上り/下り）
  - 予測呼び出し（機械学習的アプローチも可）
  - 複数台最適化（巡回セールスマン近似）

### 人移動シミュレーション
- 各人の状態: Waiting / Riding / Walking
- 目的地選択: 確率ベース or スケジュールベース
- 経路: 現在地 → 最寄りエレベーター → 目的フロア

### 満足度計算例
```
satisfaction = base
  - travel_time * 0.8
  - wait_time * 1.5
  + amenity_bonus
  - overcrowding_penalty
```

### 収益モデル
- フロアごと基本家賃
- 利用者数 × 単価
- ピークタイムボーナス

## 5. Agent向けAPI（最小セット）

```rust
// 状態取得
fn get_state() -> TowerState;  // 完全JSON

// 建設
fn build_floor(floor_type: FloorType, level: i32) -> Result;

// エレベーター
fn add_elevator_shaft() -> Result;
fn place_elevator_car(shaft_id: u32, floors: Vec<i32>) -> Result;

// シミュレーション
fn advance(minutes: u32) -> SimulationResult;

// メトリクス
fn get_metrics() -> Metrics;
```

## 6. データ構造（Rust案）

```rust
struct Tower {
    floors: Vec<Floor>,
    elevators: Vec<Elevator>,
    people: Vec<Person>,
    time: u64,
    money: i64,
}

struct Floor {
    level: i32,
    floor_type: FloorType,
    capacity: u32,
    current_occupants: u32,
}

struct Elevator {
    shaft: u32,
    current_floor: i32,
    direction: Direction,
    passengers: Vec<PersonId>,
    capacity: u32,
}
```

## 7. 次のステップ（実装準備完了）

- [ ] babelsim-coreクレート作成
- [ ] 最小状態管理 + JSON出力
- [ ] advance() と build_floor() の実装
- [ ] 基本的な人移動ロジック
- [ ] ヘッドレスCLIでテスト実行

この情報で実装開始可能。
追加リサーチが必要な部分があれば随時更新。
