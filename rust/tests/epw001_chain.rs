//! EPW001a probe: exercise the real OpenAEC transmission -> ventilation -> demand APIs.
//!
//! This is deliberately a diagnostic probe, not a conformance assertion yet.
//! EPW001a is the cleanest first EDR case: one zone, known geometry, four south
//! windows, Rc 6 envelope, ground floor, D.2 balanced ventilation with WTW.
//! The test prints intermediate quantities so CI can expose the first numerical
//! gap before we encode a 1% acceptance gate.

use std::collections::HashMap;

use nta8800_demand::{calculate_demand, CoolingSetpoint, HeatingSetpoint, InternalGains, ThermalMassInput};
use nta8800_model::geometry::Window;
use nta8800_model::location::{Orientation, Tilt};
use nta8800_model::time::MonthlyProfile;
use nta8800_model::zoning::{Rekenzone, UsageFunction};
use nta8800_tables::climate::de_bilt::de_bilt_climate_data;
use nta8800_transmission::{calculate_transmission, BoundaryType, TransmissionElement};
use nta8800_ventilation::{calculate_ventilation, AirFlow, VentilationSystem, WtwSpecification};

fn zone() -> Rekenzone {
    Rekenzone {
        id: "EPW001a-rz1".into(),
        name: "EPW001a".into(),
        gebouw_id: "EPW001a".into(),
        floor_area: 96.0,
        volume: 259.2,
        efr_ids: vec!["woonfunctie".into()],
        constructions: vec![],
        windows: vec![],
        openings: vec![],
        thermal_bridges_linear: vec![],
        thermal_bridges_point: vec![],
    }
}

fn transmission_elements() -> Vec<TransmissionElement> {
    // EPW001a: Ao = 247.2 m². Wall areas are gross external areas; 24 m²
    // windows occupy the south wall, leaving 19.2 m² opaque south wall.
    vec![
        TransmissionElement { id: "dak".into(), area: 48.0, u_value: 0.162, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "vloer".into(), area: 48.0, u_value: 1.0 / (6.0 + 0.21), boundary_type: BoundaryType::Ground, construction_id: None },
        TransmissionElement { id: "gevel-zuid".into(), area: 19.2, u_value: 0.162, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "gevel-west".into(), area: 32.4, u_value: 0.162, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "gevel-oost".into(), area: 32.4, u_value: 0.162, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "gevel-noord".into(), area: 43.2, u_value: 0.162, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "ramen-zuid".into(), area: 24.0, u_value: 1.8, boundary_type: BoundaryType::Outdoor, construction_id: None },
    ]
}

fn windows() -> Vec<Window> {
    (1..=4)
        .map(|i| {
            Window::new(
                format!("raam-{i}"),
                "EPW001a-raam",
                6.0,
                Orientation::Zuid,
                Tilt::VERTICAL,
                1.8,
                0.7,
                0.25,
            )
            .expect("valid EPW001a window")
        })
        .collect()
}

#[test]
fn epw001a_transmission_ventilation_demand_probe() {
    let zone = zone();
    let climate = de_bilt_climate_data();
    let indoor = MonthlyProfile::from_constant(20.0);

    // EPW001a specifies the NTA ground-contact floor but the pinned OpenAEC
    // V1 transmission API intentionally leaves the §8.3 ground coefficient to
    // the consumer. Keep it explicit at zero in this first probe rather than
    // inventing a ground model; the printed delta identifies this gap.
    let transmission = calculate_transmission(
        &zone,
        &transmission_elements(),
        &[],
        &[],
        &indoor,
        &climate,
        0.0,
        &HashMap::new(),
        &HashMap::new(),
    )
    .expect("EPW001a transmission calculation should succeed");

    // EPW001a does not provide a measured ventilation flow. For this probe use
    // the dwelling design flow 0.9 dm³/(s·m²) × Ag = 86.4 dm³/s = 311.04 m³/h.
    // The D.2 WTW is represented explicitly. Infiltration is kept at zero in
    // this first probe because qv10 -> qV;lea requires the NTA §11 pressure/
    // resistance procedure and must not be guessed here.
    let airflow = AirFlow::new(311.04, 311.04, 0.0);
    let wtw = WtwSpecification::new(0.80, 0.45 / 3.6, true);
    let ventilation = calculate_ventilation(
        &zone,
        &VentilationSystem::D { with_wtw: true },
        &airflow,
        Some(&wtw),
        &indoor,
        &climate,
    )
    .expect("EPW001a ventilation calculation should succeed");

    let windows_owned = windows();
    let window_refs: Vec<&Window> = windows_owned.iter().collect();
    let internal = InternalGains::forfaitair(UsageFunction::Woonfunctie);
    let heating_sp = HeatingSetpoint::new(MonthlyProfile::from_constant(20.0));
    let cooling_sp = CoolingSetpoint::new(MonthlyProfile::from_constant(24.0));

    let demand = calculate_demand(
        &zone,
        &transmission,
        &ventilation,
        311.04 * 1212.23 / 3600.0,
        &window_refs,
        &climate,
        heating_sp,
        cooling_sp,
        &internal,
        ThermalMassInput::zwaar_massief(),
        1.0,
    )
    .expect("EPW001a demand calculation should succeed");

    let qh_kwh = demand.annual_heating_demand / 3.6;
    let qh_per_m2 = qh_kwh / zone.floor_area;
    let reference_qh_per_m2 = 42.69_f64;
    let relative_error = (qh_per_m2 - reference_qh_per_m2).abs() / reference_qh_per_m2;

    println!("EPW001a diagnostic");
    println!("  H_D = {:.6} W/K", transmission.h_d);
    println!("  H_g;an supplied = 0.0 W/K (known V1 gap)");
    println!("  Q_T;an = {:.6} MJ", transmission.annual_q_t);
    println!("  Q_V;an = {:.6} MJ", ventilation.annual_q_v);
    println!("  W_fan;an = {:.6} MJ", ventilation.annual_w_fan);
    println!("  Q_H;nd;an = {:.6} kWh", qh_kwh);
    println!("  Q_H;nd;net = {:.6} kWh/m²", qh_per_m2);
    println!("  EDR reference Q_H;nd;net = {:.6} kWh/m²", reference_qh_per_m2);
    println!("  relative error = {:.3}%", relative_error * 100.0);

    // First gate: the real chain is executable and finite. The 1% gate is
    // deliberately not enabled until the documented V1 gaps are replaced.
    assert!(qh_per_m2.is_finite() && qh_per_m2 > 0.0);
}
