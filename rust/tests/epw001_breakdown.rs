//! EPW001a diagnostic: expose demand components while infiltration is being isolated.

use std::collections::HashMap;
use nta8800_demand::{calculate_demand, CoolingSetpoint, HeatingSetpoint, InternalGains, ThermalMassInput};
use nta8800_model::geometry::Window;
use nta8800_model::location::{Orientation, Tilt};
use nta8800_model::time::MonthlyProfile;
use nta8800_model::zoning::{Rekenzone, UsageFunction};
use nta8800_tables::climate::de_bilt::de_bilt_climate_data;
use nta8800_transmission::{calculate_transmission, BoundaryType, TransmissionElement};
use nta8800_ventilation::VentilationResult;

fn ground_hg() -> f64 {
    let a = 48.0; let p = 28.0; let u = 1.0 / (6.0 + 0.21);
    let b = a / (0.5 * p);
    let d = 0.5 + 2.0 * (1.0 / u + 0.04);
    let ufl = 2.0 * 2.0 / (std::f64::consts::PI * b + d) * (std::f64::consts::PI * b / d + 1.0).ln();
    a * ufl
}

fn zone() -> Rekenzone {
    Rekenzone { id: "EPW001a-rz1".into(), name: "EPW001a".into(), gebouw_id: "EPW001a".into(), floor_area: 96.0, volume: 259.2, efr_ids: vec!["woonfunctie".into()], constructions: vec![], windows: vec![], openings: vec![], thermal_bridges_linear: vec![], thermal_bridges_point: vec![] }
}

fn elements() -> Vec<TransmissionElement> {
    let u = 0.162 + 0.10;
    vec![
        TransmissionElement { id: "dak".into(), area: 48.0, u_value: u, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "vloer".into(), area: 48.0, u_value: 1.0 / 6.21, boundary_type: BoundaryType::Ground, construction_id: None },
        TransmissionElement { id: "zuid".into(), area: 19.2, u_value: u, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "west".into(), area: 32.4, u_value: u, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "oost".into(), area: 32.4, u_value: u, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "noord".into(), area: 43.2, u_value: u, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "ramen".into(), area: 24.0, u_value: 1.8, boundary_type: BoundaryType::Outdoor, construction_id: None },
    ]
}

fn windows() -> Vec<Window> {
    (0..4).map(|i| Window::new(format!("w{i}"), "w", 6.0, Orientation::Zuid, Tilt::VERTICAL, 1.8, 0.7, 0.25).unwrap()).collect()
}

fn ventilation(qv10: f64, climate: &nta8800_model::ClimateData) -> VentilationResult {
    const QMECH: f64 = 311.04; const RHO_C: f64 = 1212.23; const ETA: f64 = 0.8;
    let mut qv = [0.0; 12];
    // For this diagnostic, reuse the established probe's annual infiltration
    // result by approximating its pressure-balanced monthly airflow with the
    // calibrated relation visible from the differential run. This file is only
    // intended to expose demand internals, not to establish a new equation.
    let infiltration_m3_h = qv10 * 1.4 * 96.0 * 3.6 / 10.0_f64.powf(0.67) * 0.10;
    for m in nta8800_model::time::Month::all() {
        let i = m.index(); let h = [744.,672.,744.,720.,744.,720.,744.,744.,720.,744.,720.,744.][i];
        let te = climate.outdoor_temperature[m];
        let dt = (20.0 - te).max(0.0);
        let mech = QMECH * RHO_C * (1.0 - ETA) * dt * h / 1_000_000.0;
        let inf = infiltration_m3_h * RHO_C * dt * h / 1_000_000.0;
        qv[i] = mech + inf;
    }
    VentilationResult { monthly_q_v: MonthlyProfile::new(qv), annual_q_v: qv.iter().sum(), monthly_w_fan: MonthlyProfile::from_constant(0.0), annual_w_fan: 0.0, monthly_wtw_recovery: MonthlyProfile::from_constant(0.0), annual_wtw_recovery: 0.0 }
}

#[test]
fn print_epw001a_breakdown() {
    let z = zone(); let c = de_bilt_climate_data(); let indoor = MonthlyProfile::from_constant(20.0);
    let tr = calculate_transmission(&z, &elements(), &[], &[], &indoor, &c, ground_hg(), &HashMap::new(), &HashMap::new()).unwrap();
    for qv10 in [0.0, 0.2, 0.7] {
        let vn = ventilation(qv10, &c); let wo = windows(); let wr: Vec<&Window> = wo.iter().collect();
        let d = calculate_demand(&z, &tr, &vn, vn.annual_q_v / 8760.0 * 1_000_000.0 / (1212.23 * 3600.0), &wr, &c,
            HeatingSetpoint::new(MonthlyProfile::from_constant(20.0)), CoolingSetpoint::new(MonthlyProfile::from_constant(24.0)),
            &InternalGains::forfaitair(UsageFunction::Woonfunctie), ThermalMassInput::zwaar_massief(), 1.0).unwrap();
        let sum = |p: &MonthlyProfile<f64>| p.as_array().iter().sum::<f64>();
        eprintln!("qv10={qv10:.2}: H_D={:.3} H_g={:.3} Q_T={:.1}MJ Q_V={:.1}MJ tau={:.2}h Qsol={:.1}MJ Qint={:.1}MJ Qht={:.1}MJ Qgn={:.1}MJ QH={:.3}kWh/m2",
            tr.h_d, tr.h_g_an, tr.annual_q_t, vn.annual_q_v, d.breakdown.time_constant_hours, sum(&d.breakdown.monthly_q_sol), sum(&d.breakdown.monthly_q_int), sum(&d.breakdown.monthly_q_ht), sum(&d.breakdown.monthly_q_gn), d.annual_heating_demand/3.6/z.floor_area);
    }
}
