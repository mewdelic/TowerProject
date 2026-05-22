use babelsim_core::{FloorType, Tower};

fn main() {
    let mut tower = Tower::new();

    println!("=== BabelSim Tower ===\n");
    println!("Building your tower...\n");

    // Build a modest tower with the new economy
    tower.build_floor(FloorType::Lobby, 0).unwrap();
    println!("[BUILD] Lobby (1F)  - $10,000");

    tower.build_floor(FloorType::Retail, 1).unwrap();
    let cost = tower.get_state().total_expenses;
    println!("[BUILD] Retail (2F)  - ${}", cost);

    tower.add_elevator(0).unwrap();
    let cost = tower.get_state().total_expenses;
    println!("[BUILD] Elevator     - $50,000");

    tower.build_floor(FloorType::Restaurant, 3).unwrap();
    let cost = tower.get_state().total_expenses;
    println!("[BUILD] Rest. (4F)   - ${}", cost);

    println!("\n💰 Money: ${}\n", tower.get_state().money);

    // Spawn some initial people
    for _ in 0..3 {
        tower.spawn_person(0, 1);
        tower.spawn_person(0, 3);
        tower.spawn_person(1, 3);
    }

    // Run simulation in 4-hour chunks
    let days = 3;
    for day in 1..=days {
        println!("\n━━━ Day {} ━━━", day);

        // Morning (8:00-12:00)
        let r1 = tower.advance(240);
        let m = tower.metrics();
        println!("  Morning | 💰${:>8} | 📈{:>+8} | 😊{:>5.1}% | 👥{:>4} active | ✅{:>4} served",
            r1.money, r1.money_delta, m.satisfaction, m.people_active, m.people_served);

        // Day (12:00-16:00)
        let r2 = tower.advance(240);
        let m = tower.metrics();
        println!("  Day     | 💰${:>8} | 📈{:>+8} | 😊{:>5.1}% | 👥{:>4} active | ✅{:>4} served",
            r2.money, r2.money_delta, m.satisfaction, m.people_active, m.people_served);

        // Evening (16:00-20:00)
        let r3 = tower.advance(240);
        let m = tower.metrics();
        println!("  Evening | 💰${:>8} | 📈{:>+8} | 😊{:>5.1}% | 👥{:>4} active | ✅{:>4} served",
            r3.money, r3.money_delta, m.satisfaction, m.people_active, m.people_served);

        // Night (20:00-08:00) — 12 hours
        let r4 = tower.advance(720);
        let m = tower.metrics();
        println!("  Night   | 💰${:>8} | 📈{:>+8} | 😊{:>5.1}% | 👥{:>4} active | ✅{:>4} served",
            r4.money, r4.money_delta, m.satisfaction, m.people_active, m.people_served);
    }

    // Final stats
    let m = tower.metrics();
    println!("\n══════════════════════════════════");
    println!("  FINAL REPORT");
    println!("══════════════════════════════════");
    println!("  Time:       {} min ({} days)", m.time, m.time / 1440);
    println!("  Money:      ${}", m.money);
    println!("  Initial:    $150,000");
    println!("  Revenue:    ${}", m.total_revenue);
    println!("  Expenses:   ${}", m.total_expenses);
    println!("  Net:        ${}", m.money - 150_000);
    println!("  Profit %:   {}%", m.profit_rate);
    println!("  Satisfaction: {:.1}%", m.satisfaction);
    println!("  Avg Wait:     {:.1} min", m.avg_wait_ticks);
    println!("  Max Wait:     {} min", m.max_wait_ticks);
    println!("  Floors:       {}", m.floors);
    println!("  Elevators:    {}", m.elevators);
    println!("  Served:       {}", m.people_served);
    println!("  Events:       {}", m.events);

    // Operating revenue vs maintenance
    let est_maint = 9_443 * (m.time / 1440) as i64;
    let op_revenue = m.total_revenue;
    if op_revenue > est_maint && m.money > 0 {
        println!("\n  🎉 PROFITABLE! Tower earns ${}/day", 
            (op_revenue - est_maint) / (m.time.max(1440) / 1440) as i64);
    } else if m.money > 0 {
        println!("\n  😬 BORDERLINE: Not yet covering costs");
    } else {
        println!("\n  💀 BANKRUPT! The tower failed.");
    }
}
