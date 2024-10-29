use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CalculationRequest {
    battery_type: String,
    battery_size: String,
    load_current: f64, // in milliamperes (mA)
}

#[derive(Serialize)]
struct CalculationResult {
    runtime_hours: f64,
}

#[derive(Serialize)]
struct DischargeDataPoint {
    time: f64,    // in hours
    voltage: f64, // in volts
}

async fn calculate(req: web::Query<CalculationRequest>) -> impl Responder {
    let capacity = get_battery_capacity(&req.battery_type, &req.battery_size);

    match capacity {
        Some(cap) => {
            let runtime = cap / req.load_current;
            let result = CalculationResult {
                runtime_hours: runtime,
            };
            HttpResponse::Ok().json(result)
        }
        None => HttpResponse::BadRequest().body("Invalid battery type or size"),
    }
}

async fn discharge_curve(req: web::Query<CalculationRequest>) -> impl Responder {
    let capacity = get_battery_capacity(&req.battery_type, &req.battery_size);
    let nominal_voltage = get_battery_voltage(&req.battery_type, &req.battery_size);

    match (capacity, nominal_voltage) {
        (Some(cap), Some(voltage)) => {
            let runtime_hours = cap / req.load_current;
            let data = simulate_discharge_curve(runtime_hours, voltage);
            HttpResponse::Ok().json(data)
        }
        _ => HttpResponse::BadRequest().body("Invalid battery type or size"),
    }
}

fn get_battery_capacity(battery_type: &str, battery_size: &str) -> Option<f64> {
    // Capacities are in milliampere-hours (mAh)
    match (battery_type.to_lowercase().as_str(), battery_size.to_lowercase().as_str()) {
        ("alkaline", "aa") => Some(2500.0),
        ("alkaline", "aaa") => Some(1200.0),
        ("nimh", "aa") => Some(2000.0),
        ("nimh", "aaa") => Some(800.0),
        ("nicd", "aa") => Some(1000.0),
        ("nicd", "aaa") => Some(300.0),
        ("lithium-ion", "aa") => Some(3500.0),
        ("lithium-ion", "aaa") => Some(2000.0),
        _ => None,
    }
}

fn get_battery_voltage(battery_type: &str, _battery_size: &str) -> Option<f64> {
    // Nominal voltages in volts (V)
    match battery_type.to_lowercase().as_str() {
        "alkaline" => Some(1.5),
        "nimh" => Some(1.2),
        "nicd" => Some(1.2),
        "lithium-ion" => Some(3.7),
        _ => None,
    }
}

fn simulate_discharge_curve(runtime_hours: f64, nominal_voltage: f64) -> Vec<DischargeDataPoint> {
    let mut data_points = Vec::new();
    let intervals = 20;
    let time_step = runtime_hours / intervals as f64;

    for i in 0..=intervals {
        let time = i as f64 * time_step;
        let voltage = calculate_voltage_at_time(time, runtime_hours, nominal_voltage);
        data_points.push(DischargeDataPoint { time, voltage });
    }

    data_points
}

fn calculate_voltage_at_time(time: f64, runtime_hours: f64, nominal_voltage: f64) -> f64 {
    // Simple linear discharge model for demonstration purposes
    let end_voltage = nominal_voltage * 0.9; // Assume battery is "dead" at 90% voltage

    if time >= runtime_hours {
        end_voltage
    } else {
        nominal_voltage - (nominal_voltage - end_voltage) * (time / runtime_hours)
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://localhost:8080");
    HttpServer::new(|| {
        App::new()
            .route("/calculate", web::get().to(calculate))
            .route("/discharge_curve", web::get().to(discharge_curve))
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
