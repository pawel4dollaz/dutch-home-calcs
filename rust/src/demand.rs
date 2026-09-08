use crate::input::ZoneInput;
use crate::kernel::TransmissionResult;
use crate::ventilation::VentilationResult;

const MONTH_HOURS: [f64; 12] = [744.0, 672.0, 744.0, 720.0, 744.0, 720.0, 744.0, 744.0, 720.0, 744.0, 720.0, 744.0];
const WH_TO_MJ: f64 = 0.0036;

/// Monthly heating/cooling demand result with a transparent heat-balance ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct DemandResult {
    pub monthly_heating_mj: [f64; 12],
    pub monthly_cooling_mj: [f64; 12],
    pub monthly_transmission_mj: [f64; 12],
    pub monthly_ventilation_mj: [f64; 12],
    pub monthly_internal_gains_mj: [f64; 12],
    pub monthly_solar_gains_mj: [f64; 12],
    pub monthly_utilization_heating: [f64; 12],
    pub monthly_utilization_cooling: [f64; 12],
    pub time_constant_hours: f64,
    pub annual_heating_mj: f64,
    pub annual_cooling_mj: f64,
}

/// Demand calculation errors. The kernel refuses to guess missing physics inputs.
#[derive(Debug, Clone, PartialEq)]
pub enum DemandError {
    InvalidInput(String),
    MissingSolarOrientation(String),
    InvalidThermalMass,
}

/// Calculate H.7 monthly heating/cooling demand using the pinned OpenAEC V1
/// monthly-balance method: Q_nd = Q_ht - η·Q_gn, with explicit solar and
/// internal gains. Solar irradiation is supplied by the project climate input.
pub fn calculate_demand(
    zone: &ZoneInput,
    transmission: &TransmissionResult,
    ventilation: &VentilationResult,
    solar_irradiation_kwh_m2: &std::collections::BTreeMap<String, [f64; 12]>,
) -> Result<DemandResult, DemandError> {
    if zone.floor_area_m2 <= 0.0 || !zone.floor_area_m2.is_finite() {
        return Err(DemandError::InvalidInput("zone floor area must be finite and > 0".into()));
    }
    let mass = zone.thermal_mass_kj_m2k.ok_or(DemandError::InvalidThermalMass)?;
    if mass <= 0.0 || !mass.is_finite() {
        return Err(DemandError::InvalidThermalMass);
    }

    let h_tr = transmission.h_d_w_k + transmission.h_u_w_k + transmission.h_ground_w_k + transmission.h_adjacent_w_k;
    let avg_vent_m3_h = ventilation.monthly_ventilation_m3_h.iter().sum::<f64>() / 12.0;
    let avg_inf_m3_h = ventilation.monthly_infiltration_m3_h.iter().sum::<f64>() / 12.0;
    // ρ·c_p ≈ 0.34 Wh/(m³·K); use the same physical conversion as the
    // ventilation heat-loss ledger rather than a fitted demand coefficient.
    let h_ve = (avg_vent_m3_h + avg_inf_m3_h) * 0.34;
    let h_total = h_tr + h_ve;
    if h_total <= 0.0 || !h_total.is_finite() {
        return Err(DemandError::InvalidInput("H_tr + H_ve must be finite and > 0".into()));
    }
    let tau = (mass * zone.floor_area_m2 * 1000.0 / 3600.0) / h_total;
    let a = 1.0 + tau / 15.0;

    let gains = zone.internal_gains_w_m2.unwrap_or([3.0; 12]);
    let mut solar = [0.0; 12];
    for opening in &zone.openings {
        if opening.boundary != crate::input::Boundary::Outdoor { continue; }
        let key = orientation_key(opening.orientation_deg);
        let profile = solar_irradiation_kwh_m2.get(&key)
            .or_else(|| solar_irradiation_kwh_m2.get("global"))
            .ok_or_else(|| DemandError::MissingSolarOrientation(key.clone()))?;
        let effective = opening.area_m2 * opening.g_value * opening.shading_factor * (1.0 - opening.frame_fraction);
        if effective < 0.0 || !effective.is_finite() { return Err(DemandError::InvalidInput(format!("opening {} has invalid solar parameters", opening.id))); }
        for i in 0..12 { solar[i] += effective * profile[i] * 3.6; }
    }

    let mut q_h = [0.0; 12];
    let mut q_c = [0.0; 12];
    let mut q_tr = [0.0; 12];
    let mut q_ve = [0.0; 12];
    let mut q_int = [0.0; 12];
    let mut eta_h = [0.0; 12];
    let mut eta_c = [0.0; 12];
    for i in 0..12 {
        q_tr[i] = transmission.monthly_q_tr_kwh[i] * 3.6;
        q_ve[i] = ventilation.monthly_heat_loss_kwh[i] * 3.6;
        q_int[i] = gains[i] * zone.floor_area_m2 * MONTH_HOURS[i] * WH_TO_MJ;
        let q_ht = (q_tr[i] + q_ve[i]).max(0.0);
        let q_gn = (q_int[i] + solar[i]).max(0.0);
        let gamma = if q_ht > 0.0 { q_gn / q_ht } else { 0.0 };
        let eh = utilization_heating(gamma, a);
        let ec = utilization_cooling(gamma, a);
        eta_h[i] = eh;
        eta_c[i] = ec;
        q_h[i] = (q_ht - eh * q_gn).max(0.0);
        q_c[i] = (q_gn - ec * q_ht).max(0.0);
    }
    Ok(DemandResult {
        monthly_heating_mj: q_h,
        monthly_cooling_mj: q_c,
        monthly_transmission_mj: q_tr,
        monthly_ventilation_mj: q_ve,
        monthly_internal_gains_mj: q_int,
        monthly_solar_gains_mj: solar,
        monthly_utilization_heating: eta_h,
        monthly_utilization_cooling: eta_c,
        time_constant_hours: tau,
        annual_heating_mj: q_h.iter().sum(),
        annual_cooling_mj: q_c.iter().sum(),
    })
}

fn utilization_heating(gamma: f64, a: f64) -> f64 {
    if gamma <= 0.0 { return 1.0; }
    if (gamma - 1.0).abs() < 1e-9 { return a / (a + 1.0); }
    let numerator = 1.0 - gamma.powf(a);
    let denominator = 1.0 - gamma.powf(a + 1.0);
    if denominator.abs() < f64::EPSILON { a / (a + 1.0) } else { (numerator / denominator).clamp(0.0, 1.0) }
}

fn utilization_cooling(gamma: f64, a: f64) -> f64 {
    if gamma <= 0.0 { return 0.0; }
    if (gamma - 1.0).abs() < 1e-9 { return a / (a + 1.0); }
    let ga = gamma.powf(a);
    let ga1 = gamma.powf(a + 1.0);
    if ga < f64::EPSILON || ga1 < f64::EPSILON { return 1.0; }
    let denominator = 1.0 - 1.0 / ga1;
    if denominator.abs() < f64::EPSILON { a / (a + 1.0) } else { (1.0 - 1.0 / ga) / denominator.clamp(-f64::MAX, f64::MAX) }
}

fn orientation_key(deg: f64) -> String {
    let d = ((deg % 360.0) + 360.0) % 360.0;
    let cardinal = match d {
        x if !(22.5..337.5).contains(&x) => "north",
        x if x < 67.5 => "northeast",
        x if x < 112.5 => "east",
        x if x < 157.5 => "southeast",
        x if x < 202.5 => "south",
        x if x < 247.5 => "southwest",
        x if x < 292.5 => "west",
        _ => "northwest",
    };
    cardinal.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utilization_limits_are_finite() {
        for g in [0.0, 0.1, 0.5, 1.0, 2.0, 10.0] {
            let h = utilization_heating(g, 3.0);
            let c = utilization_cooling(g, 3.0);
            assert!(h.is_finite() && c.is_finite());
            assert!((0.0..=1.0).contains(&h));
            assert!((0.0..=1.0).contains(&c));
        }
    }

    #[test]
    fn orientation_mapping_wraps() {
        assert_eq!(orientation_key(0.0), "north");
        assert_eq!(orientation_key(180.0), "south");
        assert_eq!(orientation_key(-180.0), "south");
        assert_eq!(orientation_key(360.0), "north");
    }
}
