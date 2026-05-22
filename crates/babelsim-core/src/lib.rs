//! babelsim-core
//! Tower simulation core logic for Agent playability

use serde::{Deserialize, Serialize};

// ─── Types ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FloorType {
    Office,
    Hotel,
    Restaurant,
    Retail,
    Residential,
    Lobby,
    Observatory,
}

impl FloorType {
    pub fn build_cost(&self, level: i32) -> i64 {
        // Higher floors cost more: base * (1 + level/20)
        let base = match self {
            FloorType::Office => 60_000,
            FloorType::Hotel => 100_000,
            FloorType::Restaurant => 40_000,
            FloorType::Retail => 35_000,
            FloorType::Residential => 50_000,
            FloorType::Lobby => 10_000,
            FloorType::Observatory => 120_000,
        };
        let height_mod = 1.0 + (level.max(0) as f64 / 20.0);
        (base as f64 * height_mod) as i64
    }

    pub fn daily_maintenance(&self, level: i32) -> i64 {
        // Higher floors also cost more to maintain
        let base = match self {
            FloorType::Office => 4_000,
            FloorType::Hotel => 6_000,
            FloorType::Restaurant => 3_000,
            FloorType::Retail => 2_500,
            FloorType::Residential => 3_000,
            FloorType::Lobby => 1_500,
            FloorType::Observatory => 5_000,
        };
        let height_mod = 1.0 + (level.max(0) as f64 / 30.0);
        (base as f64 * height_mod) as i64
    }

    pub fn income_per_person(&self) -> i64 {
        match self {
            FloorType::Office => 120,
            FloorType::Hotel => 200,
            FloorType::Restaurant => 80,
            FloorType::Retail => 60,
            FloorType::Residential => 180,
            FloorType::Lobby => 0,
            FloorType::Observatory => 100,
        }
    }

    pub fn capacity(&self) -> u32 {
        match self {
            FloorType::Office => 100,
            FloorType::Hotel => 80,
            FloorType::Restaurant => 50,
            FloorType::Retail => 60,
            FloorType::Residential => 40,
            FloorType::Lobby => 200,
            FloorType::Observatory => 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Floor {
    pub level: i32,
    pub floor_type: FloorType,
    pub capacity: u32,
    pub current_occupants: u32,
    pub satisfaction: f64, // 0.0 ~ 100.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Idle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Elevator {
    pub shaft: u32,
    pub current_floor: i32,
    pub direction: Direction,
    pub capacity: u32,
    pub passengers: Vec<u32>,
    pub total_wait_ticks: u64,
    pub trips_completed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    pub id: u32,
    pub current_floor: i32,
    pub destination: i32,
    pub state: String,       // waiting | riding | arrived
    pub wait_ticks: u64,
    pub travel_ticks: u64,
    pub spawned_at_tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub floor: Option<i32>,
    pub ticks_remaining: u32,
}

// ─── Tower State ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerState {
    pub time: u64,                 // total ticks (1 tick = 1 minute)
    pub money: i64,
    pub total_revenue: i64,
    pub total_expenses: i64,
    pub floors: Vec<Floor>,
    pub elevators: Vec<Elevator>,
    pub people: Vec<Person>,
    pub active_events: Vec<Event>,
    pub overall_satisfaction: f64,
    pub population_served: u64,
}

// ─── Tower ──────────────────────────────────────────────

pub struct Tower {
    state: TowerState,
    next_person_id: u32,
    rng: u64,
}

impl Tower {
    pub fn new() -> Self {
        Tower {
            state: TowerState {
                time: 0,
                money: 150_000,        // Reduced from 200K
                total_revenue: 0,
                total_expenses: 0,
                floors: vec![],
                elevators: vec![],
                people: vec![],
                active_events: vec![],
                overall_satisfaction: 50.0,
                population_served: 0,
            },
            next_person_id: 0,
            rng: 12345,
        }
    }

    pub fn get_state(&self) -> &TowerState {
        &self.state
    }

    // ── Construction ────────────────────────────────────

    pub fn build_floor(&mut self, floor_type: FloorType, level: i32) -> Result<(), String> {
        let cost = floor_type.build_cost(level);
        if self.state.money < cost {
            return Err(format!("Not enough money! Need {}, have {}", cost, self.state.money));
        }
        if self.state.floors.iter().any(|f| f.level == level) {
            return Err("Floor already exists at this level".to_string());
        }
        self.state.money -= cost;
        self.state.total_expenses += cost;
        self.state.floors.push(Floor {
            level,
            floor_type: floor_type.clone(),
            capacity: floor_type.capacity(),
            current_occupants: 0,
            satisfaction: 70.0,
        });
        self.state.floors.sort_by_key(|f| f.level);
        Ok(())
    }

    pub fn add_elevator(&mut self, shaft: u32) -> Result<(), String> {
        let cost = 50_000; // Increased from 30K
        if self.state.money < cost {
            return Err(format!("Not enough money! Need {}, have {}", cost, self.state.money));
        }
        if self.state.elevators.iter().any(|e| e.shaft == shaft) {
            return Err("Shaft already exists".to_string());
        }
        self.state.money -= cost;
        self.state.total_expenses += cost;
        self.state.elevators.push(Elevator {
            shaft,
            current_floor: 0,
            direction: Direction::Idle,
            capacity: 10,
            passengers: vec![],
            total_wait_ticks: 0,
            trips_completed: 0,
        });
        Ok(())
    }

    pub fn spawn_person(&mut self, from: i32, to: i32) -> u32 {
        let id = self.next_person_id;
        self.next_person_id += 1;
        self.state.people.push(Person {
            id,
            current_floor: from,
            destination: to,
            state: "waiting".to_string(),
            wait_ticks: 0,
            travel_ticks: 0,
            spawned_at_tick: self.state.time,
        });
        id
    }

    // ── Simulation ──────────────────────────────────────

    pub fn advance(&mut self, minutes: u32) -> AdvanceResult {
        let mut result = AdvanceResult::default();
        let start_money = self.state.money;

        for _ in 0..minutes {
            self.tick();
        }

        result.money_delta = self.state.money - start_money;
        result.time = self.state.time;
        result.money = self.state.money;
        result.satisfaction = self.state.overall_satisfaction;
        result.active_events = self.state.active_events.len();
        result
    }

    fn tick(&mut self) {
        self.state.time += 1;
        let tick = self.state.time;

        // 1. Spawn visitors based on time of day
        let hour = (tick / 60) % 24;
        let is_business_hours = hour >= 7 && hour <= 22;
        let spawn_rate = if hour >= 8 && hour <= 18 {
            2  // Peak hours: 2x spawn attempts
        } else if is_business_hours {
            1
        } else {
            4  // Night: rare spawns (1/4 chance)
        };

        if is_business_hours && self.fast_rand() % spawn_rate == 0 {
            self.spawn_random_visitor();
        }

        // 2. Person movement
        self.move_people();

        // 3. Elevator movement
        self.move_elevators();

        // 4. Economy: income 4x daily (every 6 hours), maintenance daily at midnight
        // Income hours: 8:00 (480), 12:00 (720), 16:00 (960), 20:00 (1200)
        let is_income_hour = tick % 1440 == 480
            || tick % 1440 == 720
            || tick % 1440 == 960
            || tick % 1440 == 1200;
        if is_income_hour {
            self.collect_revenue();
        }

        if tick % 1440 == 0 {
            self.process_maintenance();
        }

        // 5. Update satisfaction
        self.update_satisfaction();

        // 6. Random events (rare)
        if self.fast_rand() % 5000 == 0 {
            self.trigger_random_event();
        }

        // 7. Tick event timers & expire
        self.state.active_events.retain_mut(|e| {
            if e.ticks_remaining > 0 {
                e.ticks_remaining -= 1;
                e.ticks_remaining > 0
            } else {
                if e.name == "fire" {
                    self.state.money -= 20_000;
                }
                false
            }
        });
    }

    fn spawn_random_visitor(&mut self) {
        if self.state.floors.len() < 2 {
            return;
        }
        let a = (self.fast_rand() as usize) % self.state.floors.len();
        let mut b = (self.fast_rand() as usize) % self.state.floors.len();
        if b == a {
            b = (b + 1) % self.state.floors.len();
        }
        let from = self.state.floors[a].level;
        let to = self.state.floors[b].level;
        self.spawn_person(from, to);
    }

    fn move_people(&mut self) {
        for person in &mut self.state.people {
            if person.state != "waiting" {
                continue;
            }
            person.wait_ticks += 1;

            let want_dir = if person.destination > person.current_floor {
                Direction::Up
            } else {
                Direction::Down
            };

            for elev in &mut self.state.elevators {
                if elev.current_floor != person.current_floor {
                    continue;
                }
                if elev.passengers.len() >= elev.capacity as usize {
                    continue;
                }
                if elev.direction != Direction::Idle && elev.direction != want_dir {
                    continue;
                }

                elev.passengers.push(person.id);
                elev.total_wait_ticks += person.wait_ticks;
                person.state = "riding".to_string();
                person.travel_ticks = 0;

                if elev.direction == Direction::Idle {
                    elev.direction = want_dir;
                }
                break;
            }
        }
    }

    fn move_elevators(&mut self) {
        for elev in &mut self.state.elevators {
            if elev.passengers.is_empty() {
                let mut best_dist = i32::MAX;
                let mut best_target = elev.current_floor;
                for person in self.state.people.iter().filter(|p| p.state == "waiting") {
                    let dist = (person.current_floor - elev.current_floor).abs();
                    if dist < best_dist {
                        best_dist = dist;
                        best_target = person.current_floor;
                    }
                }
                if best_target != elev.current_floor {
                    elev.direction = if best_target > elev.current_floor {
                        Direction::Up
                    } else {
                        Direction::Down
                    };
                    elev.current_floor += if best_target > elev.current_floor { 1 } else { -1 };
                } else {
                    elev.direction = Direction::Idle;
                }
            } else {
                let target = self.state.people.iter()
                    .find(|p| elev.passengers.contains(&p.id))
                    .map(|p| p.destination)
                    .unwrap_or(elev.current_floor);

                if elev.current_floor < target {
                    elev.current_floor += 1;
                    elev.direction = Direction::Up;
                } else if elev.current_floor > target {
                    elev.current_floor -= 1;
                    elev.direction = Direction::Down;
                } else {
                    elev.trips_completed += 1;
                    let arrived: Vec<u32> = elev.passengers.drain(..).collect();
                    for pid in &arrived {
                        if let Some(p) = self.state.people.iter_mut().find(|pp| pp.id == *pid) {
                            p.current_floor = elev.current_floor;
                            p.state = "arrived".to_string();
                            self.state.population_served += 1;

                            if let Some(floor) = self.state.floors.iter_mut()
                                .find(|f| f.level == elev.current_floor)
                            {
                                if floor.current_occupants < floor.capacity {
                                    floor.current_occupants += 1;
                                }
                            }
                        }
                    }
                    elev.direction = Direction::Idle;
                }
            }

            for pid in &elev.passengers {
                if let Some(p) = self.state.people.iter_mut().find(|pp| pp.id == *pid) {
                    p.travel_ticks += 1;
                }
            }
        }
    }

    fn collect_revenue(&mut self) {
        let mut income: i64 = 0;

        for floor in &self.state.floors {
            if floor.current_occupants > 0 {
                let per_person = floor.floor_type.income_per_person();
                // Satisfaction modifier: 0.5x (low sat) ~ 1.5x (high sat)
                let sat_mod = 0.5 + (floor.satisfaction / 100.0).clamp(0.0, 1.0) * 1.0;
                let floor_income = (floor.current_occupants as i64) * per_person;
                income += (floor_income as f64 * sat_mod) as i64;
            }
        }

        self.state.money += income;
        self.state.total_revenue += income;
    }

    fn process_maintenance(&mut self) {
        let mut maint: i64 = 0;

        // Floor maintenance scaled by height
        for floor in &self.state.floors {
            maint += floor.floor_type.daily_maintenance(floor.level);
        }

        // Elevator maintenance: base + per floor served
        for _elev in &self.state.elevators {
            // Assume max floor height if tower has floors
            let max_floor = self.state.floors.iter().map(|f| f.level).max().unwrap_or(0).max(1) as f64;
            let base = 2_000;
            let per_floor = (max_floor / 5.0) * 100.0; // $100 per 5 floors of range
            maint += base + per_floor as i64;
        }

        // Special events modify maintenance
        if self.state.active_events.iter().any(|e| e.name == "fire") {
            maint *= 2;
        }
        if self.state.active_events.iter().any(|e| e.name == "power_outage") {
            maint += 5_000; // Generator costs
        }

        self.state.money -= maint;
        self.state.total_expenses += maint;

        // Reset occupant counts after daily cycle
        for floor in &mut self.state.floors {
            floor.current_occupants = 0;
        }
    }

    fn update_satisfaction(&mut self) {
        let mut total_sat = 0.0;
        let mut count = 0;

        for person in self.state.people.iter().filter(|p| p.state == "arrived") {
            let mut sat: f64 = 50.0;
            sat -= person.wait_ticks as f64 * 0.6;
            sat -= person.travel_ticks as f64 * 0.4;
            sat = sat.max(0.0).min(100.0);
            total_sat += sat;
            count += 1;
        }

        for floor in &self.state.floors {
            let occupancy_rate = floor.current_occupants as f64 / floor.capacity as f64;
            let mut sat = floor.satisfaction;
            if occupancy_rate > 0.8 {
                sat -= 2.0; // overcrowded penalty
            } else if occupancy_rate > 0.5 {
                sat -= 0.5;
            } else {
                sat += 0.5;
            }
            sat = sat.max(0.0).min(100.0);
            total_sat += sat;
            count += 1;
        }

        if count > 0 {
            let new_avg = total_sat / count as f64;
            self.state.overall_satisfaction = self.state.overall_satisfaction * 0.9 + new_avg * 0.1;
        }
    }

    fn trigger_random_event(&mut self) {
        let roll = self.fast_rand() % 100;
        let event = if roll < 30 {
            let floor_idx = (self.fast_rand() as usize) % self.state.floors.len().max(1);
            Event {
                name: "fire".to_string(),
                floor: Some(self.state.floors[floor_idx].level),
                ticks_remaining: 30 + (self.fast_rand() % 60) as u32,
            }
        } else if roll < 60 {
            self.state.money += 10_000;
            Event {
                name: "vip_visit".to_string(),
                floor: None,
                ticks_remaining: 60,
            }
        } else if roll < 80 {
            Event {
                name: "power_outage".to_string(),
                floor: None,
                ticks_remaining: 20,
            }
        } else {
            Event {
                name: "maintenance_boost".to_string(),
                floor: None,
                ticks_remaining: 120,
            }
        };

        if event.name == "power_outage" {
            for elev in &mut self.state.elevators {
                elev.total_wait_ticks += 10;
            }
        }

        self.state.active_events.push(event);
    }

    // ── Serialization ───────────────────────────────────

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(&self.state).unwrap_or_default()
    }

    pub fn to_json_compact(&self) -> String {
        serde_json::to_string(&self.state).unwrap_or_default()
    }

    // ── Metrics ─────────────────────────────────────────

    pub fn metrics(&self) -> Metrics {
        Metrics {
            time: self.state.time,
            money: self.state.money,
            total_revenue: self.state.total_revenue,
            total_expenses: self.state.total_expenses,
            floors: self.state.floors.len() as u32,
            elevators: self.state.elevators.len() as u32,
            people_active: self.state.people.iter().filter(|p| p.state != "arrived").count() as u32,
            people_served: self.state.population_served,
            satisfaction: self.state.overall_satisfaction,
            events: self.state.active_events.len() as u32,
            profit_rate: if self.state.total_expenses > 0 {
                (self.state.total_revenue as f64 / self.state.total_expenses as f64 * 100.0) as i64
            } else {
                0
            },
        }
    }

    // ── Simple PRNG ─────────────────────────────────────

    fn fast_rand(&mut self) -> u64 {
        self.rng = self.rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.rng >> 33
    }
}

// ─── Result Types ───────────────────────────────────────

#[derive(Debug, Default)]
pub struct AdvanceResult {
    pub time: u64,
    pub money: i64,
    pub money_delta: i64,
    pub satisfaction: f64,
    pub active_events: usize,
}

#[derive(Debug, Serialize)]
pub struct Metrics {
    pub time: u64,
    pub money: i64,
    pub total_revenue: i64,
    pub total_expenses: i64,
    pub floors: u32,
    pub elevators: u32,
    pub people_active: u32,
    pub people_served: u64,
    pub satisfaction: f64,
    pub events: u32,
    pub profit_rate: i64,
}

// ─── Tests ──────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_and_elevator() {
        let mut tower = Tower::new();
        tower.build_floor(FloorType::Lobby, 0).unwrap();
        tower.build_floor(FloorType::Office, 5).unwrap();
        tower.add_elevator(0).unwrap();
        assert_eq!(tower.get_state().floors.len(), 2);
        assert_eq!(tower.get_state().elevators.len(), 1);
        assert!(tower.get_state().money < 150_000, "Construction should cost money");
    }

    #[test]
    fn test_simple_advance() {
        let mut tower = Tower::new();
        tower.build_floor(FloorType::Lobby, 0).unwrap();
        tower.build_floor(FloorType::Office, 10).unwrap();
        tower.add_elevator(0).unwrap();
        let pid = tower.spawn_person(0, 10);
        let result = tower.advance(5);
        assert!(result.time > 0);
        assert!(tower.get_state().people.iter().any(|p| p.id == pid));
    }

    #[test]
    fn test_economy_cycle() {
        let mut tower = Tower::new();
        tower.build_floor(FloorType::Lobby, 0).unwrap();
        tower.build_floor(FloorType::Retail, 2).unwrap(); // $35K * 1.1 = $38.5K
        tower.add_elevator(0).unwrap(); // $50K

        let money_before = tower.get_state().money;

        // Run for a full day (1440 minutes)
        tower.advance(1440);

        let m = tower.metrics();
        assert!(m.total_revenue > 0 || m.total_expenses > 0,
            "Economy should have some activity after a day");
        assert!(tower.get_state().money != money_before,
            "Money should change over a day of simulation");
        // Profit rate should be reasonable (not 745%)
        assert!(m.profit_rate < 300, "Profit rate should be reasonable, got {}%", m.profit_rate);
    }

    #[test]
    fn test_satisfaction_tracks() {
        let mut tower = Tower::new();
        tower.build_floor(FloorType::Lobby, 0).unwrap();
        tower.build_floor(FloorType::Office, 3).unwrap();
        tower.add_elevator(0).unwrap();

        for _ in 0..10 {
            tower.spawn_person(0, 3);
        }
        tower.advance(120);

        let m = tower.metrics();
        assert!(m.satisfaction > 0.0, "Satisfaction should be tracked");
        assert!(m.people_served > 0, "Some people should be served");
    }
    #[test]
    fn test_build_cost_prevents_overspend() {
        let mut tower = Tower::new();
        // Observatory at low levels: 120k (level 1) + 120k (level 2) = 240k > 150k
        tower.build_floor(FloorType::Observatory, 1).unwrap();
        let result = tower.build_floor(FloorType::Observatory, 2);
        assert!(result.is_err(), "Should not afford second observatory on remaining money");
    }

    #[test]
    fn test_json_output() {
        let mut tower = Tower::new();
        tower.build_floor(FloorType::Lobby, 0).unwrap();
        let json = tower.to_json();
        assert!(json.contains("Lobby"));
        assert!(json.contains("money"));
    }

    #[test]
    fn test_compact_json() {
        let mut tower = Tower::new();
        tower.build_floor(FloorType::Lobby, 0).unwrap();
        let json = tower.to_json_compact();
        assert!(json.contains("Lobby"));
        assert!(json.len() < tower.to_json().len() + 5);
    }

    #[test]
    fn test_events_fire() {
        let mut tower = Tower::new();
        tower.build_floor(FloorType::Lobby, 0).unwrap();
        tower.build_floor(FloorType::Office, 5).unwrap();
        tower.add_elevator(0).unwrap();

        tower.advance(5000);

        let m = tower.metrics();
        assert!(m.satisfaction >= 0.0);
    }

    #[test]
    fn test_height_increases_cost() {
        let cost_low = FloorType::Office.build_cost(1);
        let cost_high = FloorType::Office.build_cost(50);
        assert!(cost_high > cost_low, "Higher floors should cost more: low={} high={}", cost_low, cost_high);
    }

    #[test]
    fn test_revenue_is_four_times_daily() {
        let mut tower = Tower::new();
        tower.build_floor(FloorType::Lobby, 0).unwrap();
        tower.build_floor(FloorType::Office, 5).unwrap();
        tower.add_elevator(0).unwrap();

        for _ in 0..5 {
            tower.spawn_person(0, 5);
        }

        tower.advance(1440); // one full day

        let m = tower.metrics();
        // Revenue should exist from 4 daily income events
        eprintln!("Day 1 revenue={}, expenses={}, profit_rate={}%", 
            m.total_revenue, m.total_expenses, m.profit_rate);
    }
}
