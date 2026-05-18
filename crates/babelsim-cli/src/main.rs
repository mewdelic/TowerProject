use babelsim_core::{FloorType, Tower};

fn main() {
    let mut tower = Tower::new();

    println!("=== BabelSim Tower ===\n");
    println!("Building your tower...\n");

    // Build floors
    tower.build_floor(FloorType::Lobby, 0).unwrap();
    println!("[BUILD] Lobby (1F)  - $10,000");

    tower.build_floor(FloorType::Office, 5).unwrap();
    println!("[BUILD] Office (5F)  - $50,000");

    tower.build_floor(FloorType::Hotel, 10).unwrap();
    println!("[BUILD] Hotel (10F)  - $80,000");

    tower.build_floor(FloorType::Retail, 7).unwrap();
    println!("[BUILD] Retail (7F)  - $25,000");

    // Elevator
    tower.add_elevator(0).unwrap();
    println!("[BUILD] Elevator Shaft 0 - $30,000");

    println!("\n💰 Money: ${}\n", tower.get_state().money);

    // Spawn some initial people
    for _ in 0..5 {
        tower.spawn_person(0, 5);
        tower.spawn_person(0, 10);
        tower.spawn_person(3, 7);
    }

    // Run simulation in 60-minute chunks
    let days = 3;
    for day in 1..=days {
        println!("
━━━ Day {} ━━━", day);

        for hour in 6..=22 {
            let result = tower.advance(60);
            let m = tower.metrics();

            // Show snapshot every 4 hours
            if hour % 4 == 0 {
                println!(
                    "  {:02}:00 | 💰${:>8} | 😊{:>5.1}% | 📈+${:>6} | 👥{:>4} active | ✅{:>4} served",
                    hour,
                    result.money,
                    m.satisfaction,
                    result.money_delta,
                    m.people_active,
                    m.people_served,
                );

                // Events
                if result.active_events > 0 {
                    for e in &tower.get_state().active_events {
                        println!("           ⚡ {} ({} min left)", e.name, e.ticks_remaining);
                    }
                }
            }
        }
    }

    // Final stats
    let m = tower.metrics();
    println!("
══════════════════════════════════");
    println!("  FINAL REPORT");
    println!("══════════════════════════════════");
    println!("  Time:       {} min ({} days)", m.time, m.time / 1440);
    println!("  Money:      ${}", m.money);
    println!("  Revenue:    ${}", m.total_revenue);
    println!("  Expenses:   ${}", m.total_expenses);
    println!("  Profit:     ${}", m.total_revenue as i64 - m.total_expenses as i64);
    println!("  Profit %:   {}%", m.profit_rate);
    println!("  Satisfaction: {:.1}%", m.satisfaction);
    println!("  Floors:       {}", m.floors);
    println!("  Elevators:    {}", m.elevators);
    println!("  Served:       {}", m.people_served);
    println!("  Events:       {}", m.events);

    if m.money > 200_000 {
        println!("
  🎉 PROFITABLE! The tower is making money!");
    } else if m.money > 0 {
        println!("
  😬 Breaking even or slight loss...");
    } else {
        println!("
  💀 BANKRUPT! The tower failed.");
    }
}
