//! EPW001a / EPW007 infiltration diagnostic.
//!
//! This is still a diagnostic probe, not a conformance gate. The ventilation
//! design flow is now derived from NTA 8800 §11.2.2.4.1/11.2.2.5 rather than
//! assuming the installed 0.9 dm3/(s m2) capacity is the calculation flow.

use std::collections::HashMap;
use nta8800_demand::{calculate_demand, CoolingSetpoint, HeatingSetpoint, InternalGains, ThermalMassInput};
use nta8800_model::geometry::Window;
use nta8800_model::location::{Orientation, Tilt};
use nta8800_model::time::{Month, MonthlyProfile};
use nta8800_model::zoning::{Rekenzone, UsageFunction};
use nta8800_tables::climate::de_bilt::de_bilt_climate_data;
use nta8800_transmission::{calculate_transmission, BoundaryType, TransmissionElement};
use nta8800_ventilation::VentilationResult;

const AG: f64 = 96.0;
const VOLUME: f64 = 259.2;
const BUILDING_HEIGHT: f64 = 5.4;
const QV_DESIGN: f64 = 0.50 * 0.40 * 1.05 * AG * 3.6; // §11.2.2.4/5: q_usi=0.50, fτ=0.40, LUKA C=1.05
const QV_MECH: f64 = QV_DESIGN;
const ETA_HR: f64 = 0.80;
const SFP_W_PER_M3_H: f64 = 0.45 / 3.6;
const N_LEA: f64 = 0.67;
const RHO_REF: f64 = 1.205;
const T_REF_K: f64 = 293.0;
const G: f64 = 9.81;

fn zone() -> Rekenzone {
    Rekenzone { id: "EPW001a-rz1".into(), name: "EPW001a".into(), gebouw_id: "EPW001a".into(), floor_area: AG, volume: VOLUME, efr_ids: vec!["woonfunctie".into()], constructions: vec![], windows: vec![], openings: vec![], thermal_bridges_linear: vec![], thermal_bridges_point: vec![] }
}

fn elements() -> Vec<TransmissionElement> {
    let u_opaque = 0.162 + 0.10; // NTA §8.2.1 forfaitary thermal-bridge U uplift
    vec![
        TransmissionElement { id: "dak".into(), area: 48.0, u_value: u_opaque, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "vloer".into(), area: 48.0, u_value: 1.0 / 6.21, boundary_type: BoundaryType::Ground, construction_id: None },
        TransmissionElement { id: "gevel-zuid".into(), area: 19.2, u_value: u_opaque, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "gevel-west".into(), area: 32.4, u_value: u_opaque, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "gevel-oost".into(), area: 32.4, u_value: u_opaque, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "gevel-noord".into(), area: 43.2, u_value: u_opaque, boundary_type: BoundaryType::Outdoor, construction_id: None },
        TransmissionElement { id: "ramen-zuid".into(), area: 24.0, u_value: 1.8, boundary_type: BoundaryType::Outdoor, construction_id: None },
    ]
}

fn windows() -> Vec<Window> {
    (1..=4).map(|i| Window::new(format!("raam-{i}"), "EPW001a-raam", 6.0, Orientation::Zuid, Tilt::VERTICAL, 1.8, 0.7, 0.25).expect("valid window")).collect()
}

fn ground_hg() -> f64 {
    let a = 48.0; let p = 28.0; let u = 1.0 / 6.21;
    let b = a / (0.5 * p);
    let d = 0.5 + 2.0 * (1.0 / u + 0.04);
    let ufl = if d < b { 4.0 / (std::f64::consts::PI * b + d) * (std::f64::consts::PI * b / d + 1.0).ln() } else { 2.0 / (0.457 * b + d) };
    a * ufl
}

fn qv1_from_qv10(qv10_lea_ref: f64) -> f64 {
    qv10_lea_ref * AG * 3.6 / 10.0_f64.powf(N_LEA)
}

/// Exact §11.2.1.1–11.2.1.7 pressure-balance structure for the EPW001a
/// one-zone, H<15 m case. qv10_lea_ref is already the final reference value:
/// measured EPW007c must not receive ftype; the forfaitary EPW001a value does.
fn monthly_infiltration(climate: &nta8800_model::ClimateData, qv10_lea_ref: f64) -> MonthlyProfile<f64> {
    let qv1 = qv1_from_qv10(qv10_lea_ref);
    let paths = [
        (0.40 * qv1, 0.5 * BUILDING_HEIGHT, 0.25_f64), // windward
        (0.40 * qv1, 0.5 * BUILDING_HEIGHT, -0.50_f64), // leeward
        (0.20 * qv1, BUILDING_HEIGHT, -0.60_f64), // roof
    ];
    let rho = |t_k: f64| RHO_REF * T_REF_K / t_k;
    MonthlyProfile::new(std::array::from_fn(|idx| {
        let m = Month::all()[idx];
        let te = climate.outdoor_temperature[m];
        let ti = 293.0;
        let te_k = te + 273.0;
        let rho_e = rho(te_k);
        let rho_i = rho(ti);
        let wind = climate.wind_speed[m];
        let p_ext = |h: f64, cp: f64| RHO_REF * (T_REF_K / te_k) * (0.5 * cp * wind * wind - h * G);
        let p_int = |p_ref: f64, h: f64| p_ref - RHO_REF * h * G * T_REF_K / ti;
        let mass_balance = |p_ref: f64| {
            // §11.2.1.5: mechanical supply uses supply-air density; exhaust uses indoor density.
            let theta_sup = if te < 20.0 { te + ETA_HR * (20.0 - te) } else { te };
            let rho_sup = rho(theta_sup + 273.0);
            let mut sum = rho_sup * QV_MECH - rho_i * QV_MECH;
            let mut q_in = 0.0;
            for (c, h, cp) in paths {
                let dp = p_ext(h, cp) - p_int(p_ref, h);
                if dp > 0.0 { q_in += c * dp.powf(N_LEA); sum += rho_e * c * dp.powf(N_LEA); }
                else if dp < 0.0 { sum -= rho_i * c * (-dp).powf(N_LEA); }
            }
            (sum, q_in)
        };
        let pe: Vec<f64> = paths.iter().map(|(_, h, cp)| p_ext(*h, *cp)).collect();
        // §11.2.1.6 step 1 / formula 11.15.
        let pgem = (pe.iter().copied().fold(f64::INFINITY, f64::min) + pe.iter().copied().fold(f64::NEG_INFINITY, f64::max)) / 2.0;
        let mut pa = pgem + rho_i * (0.5 * BUILDING_HEIGHT) * G;
        let mut ma = mass_balance(pa).0;
        if ma.abs() <= 0.9 { return mass_balance(pa).1; }
        let mut pb = pa + 2.0;
        let mut mb = mass_balance(pb).0;
        while mb.abs() > 0.9 && ma.signum() == mb.signum() {
            let r = (pb - pa).signum() * (mb - ma).signum();
            if ma.abs() > mb.abs() { pa = pb; ma = mb; }
            pb = pa - 2.0 * ma.signum() * r;
            mb = mass_balance(pb).0;
        }
        if mb.abs() <= 0.9 { return mass_balance(pb).1; }
        for _ in 0..200 {
            let pc = (pa + pb) / 2.0;
            let mc = mass_balance(pc).0;
            if mc.abs() <= 0.9 { return mass_balance(pc).1; }
            if mc.signum() == ma.signum() { pa = pc; ma = mc; } else { pb = pc; }
        }
        mass_balance((pa + pb) / 2.0).1
    }))
}

fn ventilation(climate: &nta8800_model::ClimateData, qv10_lea_ref: f64) -> VentilationResult {
    let infiltration = monthly_infiltration(climate, qv10_lea_ref);
    let mut qv = [0.0; 12];
    let mut wf = [0.0; 12];
    let mut wh = [0.0; 12];
    let hours = [744.0,672.0,744.0,720.0,744.0,720.0,744.0,744.0,720.0,744.0,720.0,744.0];
    for m in Month::all() {
        let i = m.index(); let te = climate.outdoor_temperature[m]; let h = hours[i];
        let dt = (20.0 - te).max(0.0);
        let theta_sup = if te < 20.0 { te + ETA_HR * dt } else { te };
        let q_mech_heat = QV_MECH * RHO_REF * T_REF_K / (theta_sup + 273.0) * (20.0 - theta_sup).max(0.0) * h / 1_000_000.0;
        let q_inf_heat = infiltration[m] * RHO_REF * T_REF_K / (te + 273.0) * dt * h / 1_000_000.0;
        qv[i] = q_mech_heat + q_inf_heat;
        wf[i] = SFP_W_PER_M3_H * 2.0 * QV_MECH * h * 3600.0 / 1_000_000.0;
        wh[i] = QV_MECH * RHO_REF * T_REF_K / (20.0 + 273.0) * ETA_HR * dt * h / 1_000_000.0;
    }
    VentilationResult { monthly_q_v: MonthlyProfile::new(qv), annual_q_v: qv.iter().sum(), monthly_w_fan: MonthlyProfile::new(wf), annual_w_fan: wf.iter().sum(), monthly_wtw_recovery: MonthlyProfile::new(wh), annual_wtw_recovery: wh.iter().sum() }
}

fn qh_per_m2(zone: &Rekenzone, climate: &nta8800_model::ClimateData, transmission: &nta8800_transmission::TransmissionResult, qv10_lea_ref: f64) -> (f64, f64) {
    let v = ventilation(climate, qv10_lea_ref);
    let wo = windows(); let wr: Vec<&Window> = wo.iter().collect();
    let d = calculate_demand(zone, transmission, &v, QV_MECH * 1212.23 / 3600.0, &wr, climate,
        HeatingSetpoint::new(MonthlyProfile::from_constant(20.0)), CoolingSetpoint::new(MonthlyProfile::from_constant(24.0)),
        &InternalGains::forfaitair(UsageFunction::Woonfunctie), ThermalMassInput::zwaar_massief(), 1.0).expect("demand");
    (d.annual_heating_demand / 3.6 / AG, v.annual_q_v)
}

#[test]
fn epw001a_infiltration_differential_probe() {
    let z = zone(); let c = de_bilt_climate_data(); let indoor = MonthlyProfile::from_constant(20.0);
    let tr = calculate_transmission(&z, &elements(), &[], &[], &indoor, &c, ground_hg(), &HashMap::new(), &HashMap::new()).expect("transmission");
    // EPW001a: qv10;spec;reken=0.7, ftype=1.4 => qv10;lea;ref=0.98.
    // EPW007c: measured qv10=0.2 => use 0.2 directly, per §11.2.5.
    // EPW007a: 1950 forfaitary case gives qv10;lea;ref=2.94; other building-year inputs are not changed here.
    for (name, qv10_ref, reference) in [("EPW007c", 0.20, 37.98), ("EPW001a", 0.70 * 1.40, 42.69), ("EPW007a-input", 2.94, f64::NAN)] {
        let (qh, qv) = qh_per_m2(&z, &c, &tr, qv10_ref);
        let err = if reference.is_finite() { (qh - reference).abs() / reference * 100.0 } else { f64::NAN };
        eprintln!("{name}: qv10;lea;ref={qv10_ref:.3}; Q_V;an={qv:.2} MJ; Q_H;nd;net={qh:.6} kWh/m²; EDR={reference:.2}; error={err:.3}%");
    }
    let (baseline, _) = qh_per_m2(&z, &c, &tr, 0.98);
    assert!(baseline.is_finite() && baseline > 0.0);
}
