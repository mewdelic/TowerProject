use babelsim_core::{FloorType, Tower};
use serde_json::json;
use std::sync::Mutex;
use tiny_http::{Header, Response, Server};

fn main() {
    let tower = Mutex::new(Tower::new());
    let server = Server::http("0.0.0.0:8080").expect("Failed to start server on port 8080");
    println!("🌐 BabelSim web UI at http://localhost:8080");

    for request in server.incoming_requests() {
        let url = request.url().to_string();
        let method = request.method().as_str().to_string();

        let response = match (method.as_str(), url.as_str()) {
            ("GET", "/") | ("GET", "/index.html") => serve_html(),
            ("GET", "/state") => serve_json(&tower, |t| {
                let state = t.get_state();
                json!({
                    "time": state.time,
                    "money": state.money,
                    "total_revenue": state.total_revenue,
                    "total_expenses": state.total_expenses,
                    "floors": state.floors.iter().map(|f| json!({
                        "level": f.level,
                        "type": format!("{:?}", f.floor_type),
                        "capacity": f.capacity,
                        "occupants": f.current_occupants,
                        "satisfaction": f.satisfaction,
                    })).collect::<Vec<_>>(),
                    "elevators": state.elevators.iter().map(|e| json!({
                        "shaft": e.shaft,
                        "floor": e.current_floor,
                        "direction": format!("{:?}", e.direction),
                        "passengers": e.passengers.len(),
                        "trips": e.trips_completed,
                    })).collect::<Vec<_>>(),
                    "people_waiting": state.people.iter().filter(|p| p.state == "waiting").count(),
                    "people_riding": state.people.iter().filter(|p| p.state == "riding").count(),
                    "people_served": state.population_served,
                    "overall_satisfaction": state.overall_satisfaction,
                    "active_events": state.active_events.len(),
                })
            }),
            ("GET", "/metrics") => serve_json(&tower, |t| {
                let m = t.metrics();
                json!({
                    "time": m.time,
                    "day": m.time / 1440,
                    "money": m.money,
                    "revenue": m.total_revenue,
                    "expenses": m.total_expenses,
                    "profit_rate": m.profit_rate,
                    "satisfaction": m.satisfaction,
                    "floors": m.floors,
                    "elevators": m.elevators,
                    "people_active": m.people_active,
                    "people_served": m.people_served,
                    "events": m.events,
                    "avg_wait_min": m.avg_wait_ticks,
                    "max_wait_min": m.max_wait_ticks,
                })
            }),
            ("POST", url) if url.starts_with("/reset") => {
                *tower.lock().unwrap() = Tower::new();
                json_ok("Tower reset")
            }
            ("POST", url) if url.starts_with("/build") => {
                let params = parse_params(url);
                let floor_type = match params.get("type").map(String::as_str) {
                    Some("Office") => FloorType::Office,
                    Some("Hotel") => FloorType::Hotel,
                    Some("Restaurant") => FloorType::Restaurant,
                    Some("Retail") => FloorType::Retail,
                    Some("Residential") => FloorType::Residential,
                    Some("Lobby") => FloorType::Lobby,
                    Some("Observatory") => FloorType::Observatory,
                    _ => FloorType::Office,
                };
                let level: i32 = params.get("level").and_then(|v| v.parse().ok()).unwrap_or(0);
                let mut t = tower.lock().unwrap();
                match t.build_floor(floor_type, level) {
                    Ok(_) => json_ok(&format!("Built {:?} at level {}", params.get("type").unwrap_or(&"?".into()), level)),
                    Err(e) => json_err(&e),
                }
            }
            ("POST", url) if url.starts_with("/elevator") => {
                let params = parse_params(url);
                let shaft: u32 = params.get("shaft").and_then(|v| v.parse().ok()).unwrap_or(0);
                let mut t = tower.lock().unwrap();
                match t.add_elevator(shaft) {
                    Ok(_) => json_ok(&format!("Added elevator shaft {}", shaft)),
                    Err(e) => json_err(&e),
                }
            }
            ("POST", url) if url.starts_with("/spawn") => {
                let params = parse_params(url);
                let from: i32 = params.get("from").and_then(|v| v.parse().ok()).unwrap_or(0);
                let to: i32 = params.get("to").and_then(|v| v.parse().ok()).unwrap_or(0);
                let count: u32 = params.get("count").and_then(|v| v.parse().ok()).unwrap_or(1);
                let mut t = tower.lock().unwrap();
                for _ in 0..count {
                    t.spawn_person(from, to);
                }
                json_ok(&format!("Spawned {} people ({}→{})", count, from, to))
            }
            ("POST", url) if url.starts_with("/advance") => {
                let params = parse_params(url);
                let minutes: u32 = params.get("minutes").and_then(|v| v.parse().ok()).unwrap_or(60);
                let mut t = tower.lock().unwrap();
                let _ = t.advance(minutes);
                let m = t.metrics();
                json!({
                    "ok": true,
                    "message": format!("Advanced {} minutes", minutes),
                    "money": m.money,
                    "time": m.time,
                    "satisfaction": m.satisfaction,
                    "people_served": m.people_served,
                }).to_string()
            }
            _ => json_err("Not found"),
        };

        let is_html = url == "/" || url == "/index.html";
        let content_type = if is_html { "text/html; charset=utf-8" } else { "application/json" };

        let resp = Response::from_string(response)
            .with_header(Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap())
            .with_header(Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap());

        if let Err(e) = request.respond(resp) {
            eprintln!("Response error: {}", e);
        }
    }
}

fn serve_html() -> String {
    let html = include_str!("index.html");
    html.to_string()
}

fn serve_json<F>(tower: &Mutex<Tower>, f: F) -> String
where
    F: FnOnce(&Tower) -> serde_json::Value,
{
    let t = tower.lock().unwrap();
    f(&t).to_string()
}

fn json_ok(msg: &str) -> String {
    json!({"ok": true, "message": msg}).to_string()
}

fn json_err(msg: &str) -> String {
    json!({"ok": false, "message": msg}).to_string()
}

fn parse_params(url: &str) -> std::collections::HashMap<String, String> {
    let mut params = std::collections::HashMap::new();
    if let Some(qs) = url.split('?').nth(1) {
        for pair in qs.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                params.insert(
                    urlencoding(k).to_string(),
                    urlencoding(v).to_string(),
                );
            }
        }
    }
    params
}

fn urlencoding(s: &str) -> String {
    // Simple URL decode (handles basic cases)
    s.replace("%20", " ")
        .replace("%2F", "/")
        .replace("%3A", ":")
        .replace("%3F", "?")
        .replace("%26", "&")
        .replace("%3D", "=")
}
