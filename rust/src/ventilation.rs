//! NTA 8800 §11 ventilation and infiltration foundation.
//!
//! This module implements the pressure-balance form used for infiltration and
//! keeps all physical inputs explicit. It does not silently substitute an air-
//! tightness class, installed fan flow, or heat-recovery efficiency.

use crate::input::{ClimateInput, VentilationSystemType};

const RHO_REF: f64 = 1.205;
const T_REF_K: f64 = 293.0;
const G: f64 = 9.81;
const N_LEA: f64 = 0.67;
const BALANCE_TOLERANCE_KG_H: f64 = 0.9;
const MONTH_HOURS: [f64; 12] = [744.0, 672.0, 744.0, 720.0, 744.0, 720.0, 744.0, 744.0, 720.0, 744.0, 720.0, 744.0];

/// Explicit leakage input at 10 Pa, in dm³/(s·m²) of floor area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeakageInput { pub qv10_dm3_s_m2: f64 }

/// Explicit mechanical ventilation input for one zone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MechanicalVentilationInput {
    pub system_type: VentilationSystemType,
    pub supply_m3_h: f64,
    pub extract_m3_h: f64,
    pub heat_recovery_efficiency: f64,
    pub sfp_ws_m3: f64,
    pub building_height_m: f64,
}

/// Monthly infiltration and ventilation heat-loss result.
#[derive(Debug, Clone, PartialEq)]
pub struct VentilationResult {
    pub monthly_infiltration_m3_h: [f64; 12],
    pub monthly_ventilation_m3_h: [f64; 12],
    pub monthly_heat_loss_kwh: [f64; 12],
    pub annual_heat_loss_kwh: f64,
    pub annual_fan_kwh: f64,
}

/// Errors are explicit rather than repaired with defaults.
#[derive(Debug, Clone, PartialEq)]
pub enum VentilationError { InvalidInput(&'static str), PressureBalanceDidNotConverge(usize) }

/// Calculate leakage flow coefficient C from qv10 and floor area.
pub fn leakage_c_per_path(ag_m2: f64, leakage: LeakageInput, path_fraction: f64) -> Result<f64, VentilationError> {
    if ag_m2 <= 0.0 || !ag_m2.is_finite() || leakage.qv10_dm3_s_m2 < 0.0 || !leakage.qv10_dm3_s_m2.is_finite() || path_fraction < 0.0 || !path_fraction.is_finite() { return Err(VentilationError::InvalidInput("invalid leakage input")); }
    let q_v1 = leakage.qv10_dm3_s_m2 * ag_m2 * 3.6 / 10.0_f64.powf(N_LEA);
    Ok(path_fraction * q_v1)
}

fn air_density(t_c: f64) -> f64 { RHO_REF * T_REF_K / (t_c + 273.0) }
fn external_pressure(t_e_c: f64, wind_m_s: f64, h_path_m: f64, cp: f64) -> f64 { RHO_REF * (T_REF_K / (t_e_c + 273.0)) * (0.5 * cp * wind_m_s.powi(2) - h_path_m * G) }
fn internal_path_pressure(p_ref: f64, h_path_m: f64, t_int_c: f64) -> f64 { p_ref - RHO_REF * h_path_m * G * T_REF_K / (t_int_c + 273.0) }

/// Solve the monthly mass-balance pressure for windward, leeward and roof paths.
pub fn solve_infiltration_month(t_e_c: f64, wind_m_s: f64, t_int_c: f64, building_height_m: f64, ag_m2: f64, leakage: LeakageInput, mechanical: MechanicalVentilationInput, month_index: usize) -> Result<f64, VentilationError> {
    if month_index >= 12 || !t_e_c.is_finite() || !wind_m_s.is_finite() || !t_int_c.is_finite() || building_height_m <= 0.0 || ag_m2 <= 0.0 { return Err(VentilationError::InvalidInput("invalid monthly ventilation input")); }
    if mechanical.supply_m3_h < 0.0 || mechanical.extract_m3_h < 0.0 || mechanical.heat_recovery_efficiency < 0.0 || mechanical.heat_recovery_efficiency > 1.0 || mechanical.sfp_ws_m3 < 0.0 || !mechanical.sfp_ws_m3.is_finite() { return Err(VentilationError::InvalidInput("invalid mechanical ventilation input")); }
    let paths = [
        (leakage_c_per_path(ag_m2, leakage, 0.40)?, 0.5 * building_height_m, 0.25),
        (leakage_c_per_path(ag_m2, leakage, 0.40)?, 0.5 * building_height_m, -0.50),
        (leakage_c_per_path(ag_m2, leakage, 0.20)?, building_height_m, -0.60),
    ];
    let rho_i = air_density(t_int_c); let rho_e = air_density(t_e_c);
    let supply_discharge_c = if t_e_c < t_int_c { t_e_c + mechanical.heat_recovery_efficiency * (t_int_c - t_e_c) } else { t_e_c };
    let rho_supply = air_density(supply_discharge_c);
    let mass_balance = |p_ref: f64| -> (f64, f64) {
        let mut mass_kg_h = rho_supply * mechanical.supply_m3_h - rho_i * mechanical.extract_m3_h;
        let mut incoming_m3_h = 0.0;
        for (c, h, cp) in paths {
            let dp = external_pressure(t_e_c, wind_m_s, h, cp) - internal_path_pressure(p_ref, h, t_int_c);
            if dp > 0.0 { let q = c * dp.powf(N_LEA); incoming_m3_h += q; mass_kg_h += rho_e * q; }
            else if dp < 0.0 { let q = c * (-dp).powf(N_LEA); mass_kg_h -= rho_i * q; }
        }
        (mass_kg_h, incoming_m3_h)
    };
    let pressures: Vec<f64> = paths.iter().map(|(_, h, cp)| external_pressure(t_e_c, wind_m_s, *h, *cp)).collect();
    let mut lo = pressures.iter().copied().fold(f64::INFINITY, f64::min) - 200.0;
    let mut hi = pressures.iter().copied().fold(f64::NEG_INFINITY, f64::max) + 200.0;
    let mut f_lo = mass_balance(lo).0; let mut f_hi = mass_balance(hi).0;
    for _ in 0..20 {
        if f_lo.abs() <= BALANCE_TOLERANCE_KG_H { return Ok(mass_balance(lo).1); }
        if f_hi.abs() <= BALANCE_TOLERANCE_KG_H { return Ok(mass_balance(hi).1); }
        if f_lo.signum() != f_hi.signum() { break; }
        lo -= 200.0; hi += 200.0; f_lo = mass_balance(lo).0; f_hi = mass_balance(hi).0;
    }
    if f_lo.signum() == f_hi.signum() { return Err(VentilationError::PressureBalanceDidNotConverge(month_index)); }
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0; let f_mid = mass_balance(mid).0;
        if f_mid.abs() <= BALANCE_TOLERANCE_KG_H { return Ok(mass_balance(mid).1); }
        if f_mid.signum() == f_lo.signum() { lo = mid; f_lo = f_mid; } else { hi = mid; f_hi = f_mid; }
    }
    Err(VentilationError::PressureBalanceDidNotConverge(month_index))
}

/// Calculate the monthly ventilation/infiltration heat-loss ledger.
pub fn calculate_ventilation(climate: &ClimateInput, t_int_c: [f64; 12], ag_m2: f64, leakage: LeakageInput, mechanical: MechanicalVentilationInput) -> Result<VentilationResult, VentilationError> {
    if climate.outdoor_temperature_c.iter().any(|x| !x.is_finite()) || climate.wind_speed_m_s.iter().any(|x| !x.is_finite()) { return Err(VentilationError::InvalidInput("non-finite climate")); }
    let mut infiltration = [0.0; 12]; let mut ventilation = [0.0; 12]; let mut heat_loss = [0.0; 12]; let mut fan = [0.0; 12];
    for i in 0..12 {
        infiltration[i] = solve_infiltration_month(climate.outdoor_temperature_c[i], climate.wind_speed_m_s[i], t_int_c[i], mechanical.building_height_m, ag_m2, leakage, mechanical, i)?;
        ventilation[i] = mechanical.supply_m3_h.max(mechanical.extract_m3_h);
        let dt = (t_int_c[i] - climate.outdoor_temperature_c[i]).max(0.0);
        let effective_mechanical_flow = match mechanical.system_type { VentilationSystemType::BalancedHeatRecovery => ventilation[i] * (1.0 - mechanical.heat_recovery_efficiency), _ => ventilation[i] };
        heat_loss[i] = (effective_mechanical_flow + infiltration[i]) * 0.0003367 * dt * MONTH_HOURS[i];
        fan[i] = mechanical.sfp_ws_m3 * ventilation[i] * MONTH_HOURS[i] / 1000.0;
    }
    Ok(VentilationResult { monthly_infiltration_m3_h: infiltration, monthly_ventilation_m3_h: ventilation, monthly_heat_loss_kwh: heat_loss, annual_heat_loss_kwh: heat_loss.iter().sum(), annual_fan_kwh: fan.iter().sum() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn climate() -> ClimateInput {
        ClimateInput { source_id: "test".into(), source_version: "test".into(), outdoor_temperature_c: [2.0; 12], wind_speed_m_s: [3.0; 12], solar_irradiation_kwh_m2: BTreeMap::new() }
    }
    fn mechanical() -> MechanicalVentilationInput {
        MechanicalVentilationInput { system_type: VentilationSystemType::BalancedHeatRecovery, supply_m3_h: 100.0, extract_m3_h: 100.0, heat_recovery_efficiency: 0.8, sfp_ws_m3: 0.125, building_height_m: 5.4 }
    }

    #[test]
    fn qv10_conversion_is_dimensionally_stable() {
        let c = leakage_c_per_path(96.0, LeakageInput { qv10_dm3_s_m2: 0.7 }, 1.0).unwrap();
        assert!((c - 51.721578869).abs() < 1e-9);
    }

    #[test]
    fn balanced_ventilation_has_lower_heat_loss_with_heat_recovery() {
        let recovered = calculate_ventilation(&climate(), [20.0; 12], 96.0, LeakageInput { qv10_dm3_s_m2: 0.7 }, mechanical()).unwrap();
        let mut no_recovery = mechanical(); no_recovery.heat_recovery_efficiency = 0.0;
        let unrecovered = calculate_ventilation(&climate(), [20.0; 12], 96.0, LeakageInput { qv10_dm3_s_m2: 0.7 }, no_recovery).unwrap();
        assert!(recovered.annual_heat_loss_kwh < unrecovered.annual_heat_loss_kwh);
    }

    #[test]
    fn pressure_solver_returns_finite_positive_infiltration() {
        let q = solve_infiltration_month(2.0, 3.0, 20.0, 5.4, 96.0, LeakageInput { qv10_dm3_s_m2: 0.98 }, mechanical(), 0).unwrap();
        assert!(q.is_finite() && q >= 0.0);
    }

    #[test]
    fn nonphysical_inputs_fail_closed() {
        let err = leakage_c_per_path(0.0, LeakageInput { qv10_dm3_s_m2: 0.7 }, 1.0).unwrap_err();
        assert!(matches!(err, VentilationError::InvalidInput(_)));
    }
}
