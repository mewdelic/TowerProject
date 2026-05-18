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
    pub fn build_cost(&self) -> i64 {
        match self {
            FloorType::Office => 50_000,
            FloorType::Hotel => 80_000,
            FloorType::Restaurant => 30_000,
            FloorType::Retail => 25_000,
            FloorType::Residential => 40_000,
            FloorType::Lobby => 10_000,
            FloorType::Observatory => 100_000,
        }
    }

    pub fn monthly_maintenance(&self) -> i64 {
        match self {
            FloorType::Office => 5_000,
            FloorType::Hotel => 8_000,
            FloorType::Restaurant => 4_000,
            FloorType::Retail => 3_000,
            FloorType::Residential => 3_500,
            FloorType::Lobby => 2_000,
            FloorType::Observatory => 6_000,
        }
    }

    pub fn income_per_person(&self) -> i64 {
        match self {
            FloorType::Office => 200,
            FloorType::Hotel => 500,
            FloorType::Restaurant => 150,
            FloorType::Retail => 100,
            FloorType::Residential => 800,
            FloorType::Lobby => 0,
            FloorType::Observatory => 300,
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
                money: 200_000,
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
        let cost = floor_type.build_cost();
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
        let cost = 30_000;
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

        // 1. Spawn random visitors
        let hour = (tick / 60) % 24;
        if hour >= 6 && hour <= 22 && self.fast_rand() % 3 == 0 {
            self.spawn_random_visitor();
        }

        // 2. Person movement
        self.move_people();

        // 3. Elevator movement
        self.move_elevators();

        // 4. Economy: income every hour, maintenance every day
        if tick % 60 == 0 {
            self.process_economy();
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
                // Apply expiration effects once
                if e.name == "fire" {
                    self.state.money -= 20_000;
                }
                false // remove
            }
        });
    }

    fn spawn_random_visitor(&mut self) {
        if self.state.floors.len() < 2 {
            return;
        }
        // Pick two random floors
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
        // People waiting for elevator: try to board
        for person in &mut self.state.people {
            if person.state != "waiting" {
                continue;
            }
            person.wait_ticks += 1;

            // Find best elevator at this floor going the right direction
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
                // Direction-aware: board only if elevator is idle or going same way
                if elev.direction != Direction::Idle && elev.direction != want_dir {
                    continue;
                }

                elev.passengers.push(person.id);
                elev.total_wait_ticks += person.wait_ticks;
                person.state = "riding".to_string();
                person.travel_ticks = 0;

                // Set elevator direction
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
                // Idle: seek nearest waiting person going any direction
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
                // Has passengers: go toward first passenger's destination
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
                    // Arrived! Unload
                    elev.trips_completed += 1;
                    let arrived: Vec<u32> = elev.passengers.drain(..).collect();
                    for pid in &arrived {
                        if let Some(p) = self.state.people.iter_mut().find(|pp| pp.id == *pid) {
                            p.current_floor = elev.current_floor;
                            p.state = "arrived".to_string();
                            self.state.population_served += 1;

                            // Mark occupant on floor
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

            // Update travel ticks for riding passengers
            for pid in &elev.passengers {
                if let Some(p) = self.state.people.iter_mut().find(|pp| pp.id == *pid) {
                    p.travel_ticks += 1;
                }
            }
        }
    }

    fn process_economy(&mut self) {
        let mut income: i64 = 0;

        for floor in &self.state.floors {
            if floor.current_occupants > 0 {
                let per_person = floor.floor_type.income_per_person();
                // Satisfaction modifier: 0.5x ~ 2.0x
                let sat_mod = (floor.satisfaction / 100.0).max(0.3) * 2.0;
                let floor_income = (floor.current_occupants as i64) * per_person;
                income += (floor_income as f64 * sat_mod) as i64;
            }
            // Reset occupants for next cycle
        }

        self.state.money += income;
        self.state.total_revenue += income;
    }

    fn process_maintenance(&mut self) {
        let mut maint: i64 = 0;
        for floor in &self.state.floors {
            maint += floor.floor_type.monthly_maintenance();
        }
        for _ in &self.state.elevators {
            maint += 2_000;
        }
        // Fire event doubles maintenance
        if self.state.active_events.iter().any(|e| e.name == "fire") {
            maint *= 2;
        }
        self.state.money -= maint;
        self.state.total_expenses += maint;

        // Reset occupant counts after revenue cycle
        for floor in &mut self.state.floors {
            floor.current_occupants = 0;
        }
    }

    fn update_satisfaction(&mut self) {
        let mut total_sat = 0.0;
        let mut count = 0;

        for person in self.state.people.iter().filter(|p| p.state == "arrived") {
            let mut sat: f64 = 50.0;
            sat -= person.wait_ticks as f64 * 0.5;
            sat -= person.travel_ticks as f64 * 0.3;
            sat = sat.max(0.0).min(100.0);
            total_sat += sat;
            count += 1;
        }

        // Floor satisfaction
        for floor in &self.state.floors {
            let occupancy_rate = floor.current_occupants as f64 / floor.capacity as f64;
            let mut sat = floor.satisfaction;
            if occupancy_rate > 0.8 {
                sat -= 1.0; // overcrowded
            } else {
                sat += 0.2;
            }
            sat = sat.max(0.0).min(100.0);
            total_sat += sat;
            count += 1;
        }

        if count > 0 {
            let new_avg = total_sat / count as f64;
            // Smooth toward new value
            self.state.overall_satisfaction = self.state.overall_satisfaction * 0.9 + new_avg * 0.1;
        }
    }

    fn trigger_random_event(&mut self) {
        let roll = self.fast_rand() % 100;
        let event = if roll < 30 {
            // Fire on random floor
            let floor_idx = (self.fast_rand() as usize) % self.state.floors.len().max(1);
            Event {
                name: "fire".to_string(),
                floor: Some(self.state.floors[floor_idx].level),
                ticks_remaining: 30 + (self.fast_rand() % 60) as u32,
            }
        } else if roll < 60 {
            // VIP visit - bonus
            self.state.money += 10_000;
            Event {
                name: "vip_visit".to_string(),
                floor: None,
                ticks_remaining: 60,
            }
        } else if roll < 80 {
            // Power outage - elevator penalty
            Event {
                name: "power_outage".to_string(),
                floor: None,
                ticks_remaining: 20,
            }
        } else {
            // Maintenance boost
            Event {
                name: "maintenance_boost".to_string(),
                floor: None,
                ticks_remaining: 120,
            }
        };

        // Event effects
        if event.name == "power_outage" {
            // Slow elevators
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
    pub profit_rate: i64, // percentage
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
        assert!(tower.get_state().money < 200_000, "Construction should cost money");
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
        tower.build_floor(FloorType::Hotel, 5).unwrap();
        tower.add_elevator(0).unwrap();

        let money_before = tower.get_state().money;

        // Simulate a full day (1440 minutes)
        tower.advance(1440);

        let m = tower.metrics();
        // After a day, we should have some revenue and expenses
        assert!(m.total_revenue > 0 || m.total_expenses > 0,
            "Economy should have some activity after a day");
        // Money should change
        assert!(tower.get_state().money != money_before,
            "Money should change over a day of simulation");
    }

    #[test]
    fn test_satisfaction_tracks() {
        let mut tower = Tower::new();
        tower.build_floor(FloorType::Lobby, 0).unwrap();
        tower.build_floor(FloorType::Office, 3).unwrap();
        tower.add_elevator(0).unwrap();

        // Spawn several people, advance a lot
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
        // Observatory costs 100k, we have 200k, so we can build 2 but not 3
        tower.build_floor(FloorType::Observatory, 10).unwrap();
        tower.build_floor(FloorType::Observatory, 20).unwrap();
        let result = tower.build_floor(FloorType::Observatory, 30);
        assert!(result.is_err(), "Should not afford third observatory");
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
        // Compact should be shorter
        assert!(json.len() < tower.to_json().len() + 5);
    }

    #[test]
    fn test_events_fire() {
        let mut tower = Tower::new();
        tower.build_floor(FloorType::Lobby, 0).unwrap();
        tower.build_floor(FloorType::Office, 5).unwrap();
        tower.add_elevator(0).unwrap();

        // Run long enough that events might trigger
        tower.advance(5000);

        // Events could have fired or not (random), just verify no crash
        let m = tower.metrics();
        assert!(m.satisfaction >= 0.0);
    }
}
