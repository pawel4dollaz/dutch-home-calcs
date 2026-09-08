//! EPW001a probe: exercise the real OpenAEC transmission -> ventilation -> demand APIs.

use std::collections::HashMap;
use nta8800_demand::{calculate_demand, CoolingSetpoint, HeatingSetpoint, InternalGains, ThermalMassInput};
use nta8800_model::geometry::Window;
use nta8800_model::location::{Orientation, Tilt};
use nta8800_model::time::MonthlyProfile;
use nta8800_model::zoning::{Rekenzone, UsageFunction};
use nta8800_tables::climate::de_bilt::de_bilt_climate_data;
use nta8800_transmission::{calculate_transmission, BoundaryType, TransmissionElement};
use nta8800_ventilation::{calculate_ventilation, AirFlow, VentilationSystem, WtwSpecification};

fn slab_on_ground_conductance(floor_area_m2: f64, perimeter_m: f64, floor_u_value: f64) -> f64 {
    if !(floor_area_m2 > 0.0 && perimeter_m > 0.0 && floor_u_value > 0.0) { return 0.0; }
    const LAMBDA_GROUND_W_PER_MK: f64 = 2.0;
    const R_SE_GROUND_M2K_PER_W: f64 = 0.04;
    const WALL_THICKNESS_M: f64 = 0.5;
    let b_prime = floor_area_m2 / (0.5 * perimeter_m);
    let r_si_plus_rc = 1.0 / floor_u_value;
    let d_equi = WALL_THICKNESS_M + LAMBDA_GROUND_W_PER_MK * (r_si_plus_rc + R_SE_GROUND_M2K_PER_W);
    let u_fl = if d_equi < b_prime {
        (2.0 * LAMBDA_GROUND_W_PER_MK / (std::f64::consts::PI * b_prime + d_equi))
            * (std::f64::consts::PI * b_prime / d_equi + 1.0).ln()
    } else {
        LAMBDA_GROUND_W_PER_MK / (0.457 * b_prime + d_equi)
    };
    floor_area_m2 * u_fl
}

fn zone() -> Rekenzone {
    Rekenzone {
        id: "EPW001a-rz1".into(), name: "EPW001a".into(), gebouw_id: "EPW001a".into(),
        floor_area: 96.0, volume: 259.2, efr_ids: vec!["woonfunctie".into()],
        constructions: vec![], windows: vec![], openings: vec![], thermal_bridges_linear: vec![], thermal_bridges_point: vec![],
    }
}

fn transmission_elements() -> Vec<TransmissionElement> {
    // EPW001a: linear thermal bridges are forfaitarily accounted for.
    const U_FOR: f64 = 0.10;
    vec![
        TransmissionElement { id: "dak".into(), area: 48.0, u_value: 0.162 + U_FOR, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "vloer".into(), area: 48.0, u_value: 1.0 / (6.0 + 0.21), boundary_type: BoundaryType::Ground, construction_id: None },
        TransmissionElement { id: "gevel-zuid".into(), area: 19.2, u_value: 0.162 + U_FOR, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "gevel-west".into(), area: 32.4, u_value: 0.162 + U_FOR, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "gevel-oost".into(), area: 32.4, u_value: 0.162 + U_FOR, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "gevel-noord".into(), area: 43.2, u_value: 0.162 + U_FOR, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "ramen-zuid".into(), area: 24.0, u_value: 1.8, boundary_type: BoundaryType::Outdoor, construction_id: None },
    ]
}

fn windows() -> Vec<Window> {
    (1..=4).map(|i| Window::new(format!("raam-{i}"), "EPW001a-raam", 6.0, Orientation::Zuid, Tilt::VERTICAL, 1.8, 0.7, 0.25).expect("valid EPW001a window")).collect()
}

#[test]
fn epw001a_transmission_ventilation_demand_probe() {
    let zone = zone();
    let climate = de_bilt_climate_data();
    let indoor = MonthlyProfile::from_constant(20.0);
    let floor_u = 1.0 / (6.0 + 0.21);
    let h_g_an = slab_on_ground_conductance(48.0, 28.0, floor_u);
    let transmission = calculate_transmission(&zone, &transmission_elements(), &[], &[], &indoor, &climate, h_g_an, &HashMap::new(), &HashMap::new()).expect("EPW001a transmission calculation should succeed");

    // D.2 mechanical supply/exhaust with 80% counterflow WTW. Infiltration is
    // deliberately still zero here: the pinned ventilation crate currently
    // does not implement NTA §11.2.1.5 pressure-balance coupling for system D.
    let airflow = AirFlow::new(311.04, 311.04, 0.0);
    let wtw = WtwSpecification::new(0.80, 0.45 / 3.6, true);
    let ventilation = calculate_ventilation(&zone, &VentilationSystem::D { with_wtw: true }, &airflow, Some(&wtw), &indoor, &climate).expect("EPW001a ventilation calculation should succeed");

    let windows_owned = windows();
    let window_refs: Vec<&Window> = windows_owned.iter().collect();
    let internal = InternalGains::forfaitair(UsageFunction::Woonfunctie);
    let heating_sp = HeatingSetpoint::new(MonthlyProfile::from_constant(20.0));
    let cooling_sp = CoolingSetpoint::new(MonthlyProfile::from_constant(24.0));
    let demand = calculate_demand(&zone, &transmission, &ventilation, 311.04 * 1212.23 / 3600.0, &window_refs, &climate, heating_sp, cooling_sp, &internal, ThermalMassInput::zwaar_massief(), 1.0).expect("EPW001a demand calculation should succeed");

    let qh_kwh = demand.annual_heating_demand / 3.6;
    let qh_per_m2 = qh_kwh / zone.floor_area;
    let reference_qh_per_m2 = 42.69_f64;
    let relative_error = (qh_per_m2 - reference_qh_per_m2).abs() / reference_qh_per_m2;

    // Diagnostic only. Do not turn this into the 1% gate until §11.2.1.5 is
    // implemented and the complete EDR input mapping is authoritative.
    eprintln!("EPW001a diagnostic: H_D={:.6} W/K; H_g={:.6} W/K; Q_T;an={:.6} MJ; Q_V;an={:.6} MJ; W_fan;an={:.6} MJ; Q_H;nd;an={:.6} kWh; Q_H;nd;net={:.6} kWh/m²; reference={:.6}; error={:.3}%", transmission.h_d, transmission.h_g_an, transmission.annual_q_t, ventilation.annual_q_v, ventilation.annual_w_fan, qh_kwh, qh_per_m2, reference_qh_per_m2, relative_error * 100.0);
    assert!(qh_per_m2.is_finite() && qh_per_m2 > 0.0);
}
