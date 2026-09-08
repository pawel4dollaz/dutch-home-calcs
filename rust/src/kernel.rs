use std::collections::HashMap;

use crate::input::{Boundary, NtaProjectInput, ZoneInput};

/// Calculated U-value and resistance for a construction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstructionResult {
    pub r_total_m2k_w: f64,
    pub u_value_w_m2k: f64,
}

/// Transparent monthly transmission result for one calculation zone.
#[derive(Debug, Clone, PartialEq)]
pub struct TransmissionResult {
    pub h_d_w_k: f64,
    pub h_u_w_k: f64,
    pub h_ground_w_k: f64,
    pub h_adjacent_w_k: f64,
    pub h_thermal_bridges_w_k: f64,
    pub monthly_q_tr_kwh: [f64; 12],
    pub annual_q_tr_kwh: f64,
}

/// Fail-closed kernel errors for the calculation foundation.
#[derive(Debug, Clone, PartialEq)]
pub enum KernelError {
    InvalidInput(Vec<String>),
    UnknownConstruction(String),
    MissingUnheatedBFactor(String),
    MissingAdjacentZone(String),
    InvalidConstruction(String),
}

/// Calculate Rtot and U from an explicit layered construction.
///
/// This is deliberately independent of the screening engine. No material
/// lambda, layer thickness, or surface resistance is invented here.
pub fn calculate_construction(
    input: &crate::input::ConstructionInput,
) -> Result<ConstructionResult, KernelError> {
    let mut r = input.r_si_m2k_w + input.r_se_m2k_w;
    if !r.is_finite() || r < 0.0 {
        return Err(KernelError::InvalidConstruction(input.id.clone()));
    }
    for layer in &input.layers {
        if layer.thickness_m <= 0.0 || layer.lambda_w_mk <= 0.0 || !layer.thickness_m.is_finite() || !layer.lambda_w_mk.is_finite() {
            return Err(KernelError::InvalidConstruction(input.id.clone()));
        }
        r += layer.thickness_m / layer.lambda_w_mk;
    }
    if r <= 0.0 || !r.is_finite() {
        return Err(KernelError::InvalidConstruction(input.id.clone()));
    }
    Ok(ConstructionResult { r_total_m2k_w: r, u_value_w_m2k: 1.0 / r })
}

/// Calculate the H.7/H.8 transmission foundation for one zone.
///
/// Outdoor surfaces use `U*A` after explicitly subtracting openings assigned to
/// that surface. Windows/openings contribute their declared U*A separately.
/// Unheated-space surfaces require an explicit b-factor. Ground and adjacent
/// zone transfers require explicit data. Thermal bridges are added from their
/// declared ψ·L and χ·count terms. Monthly energy uses the NTA monthly-hour
/// convention and does not apply hidden correction factors.
pub fn calculate_transmission(
    project: &NtaProjectInput,
    zone: &ZoneInput,
) -> Result<TransmissionResult, KernelError> {
    if let Err(errors) = project.validate() {
        return Err(KernelError::InvalidInput(errors));
    }

    let constructions: HashMap<_, _> = project.constructions.iter().map(|c| (c.id.as_str(), c)).collect();
    let mut opening_area_by_surface: HashMap<&str, f64> = HashMap::new();
    for opening in &zone.openings {
        if let Some(surface_id) = opening.surface_id.as_deref() {
            *opening_area_by_surface.entry(surface_id).or_default() += opening.area_m2;
        }
    }

    let mut h_d = 0.0;
    let mut h_u = 0.0;
    let mut h_ground = 0.0;
    let mut h_adjacent = 0.0;
    let mut h_bridges = 0.0;

    for surface in &zone.surfaces {
        let construction = constructions
            .get(surface.construction_id.as_str())
            .ok_or_else(|| KernelError::UnknownConstruction(surface.construction_id.clone()))?;
        let c = calculate_construction(construction)?;
        let openings = opening_area_by_surface.get(surface.id.as_str()).copied().unwrap_or(0.0);
        if openings > surface.area_m2 + 1e-9 {
            return Err(KernelError::InvalidInput(vec![format!("surface {} has {:.6} m² openings on {:.6} m² surface", surface.id, openings, surface.area_m2)]));
        }
        let opaque_area = (surface.area_m2 - openings).max(0.0);
        match surface.boundary {
            Boundary::Outdoor => h_d += c.u_value_w_m2k * opaque_area,
            Boundary::Ground => {
                if let Some(hg) = surface.ground_hg_an_w_k {
                    if hg < 0.0 || !hg.is_finite() { return Err(KernelError::InvalidInput(vec![format!("surface {} has invalid H_g;an", surface.id)])); }
                    h_ground += hg;
                }
            }
            Boundary::UnheatedSpace => {
                let b = surface.unheated_b_factor.ok_or_else(|| KernelError::MissingUnheatedBFactor(surface.id.clone()))?;
                if !(0.0..=1.0).contains(&b) || !b.is_finite() { return Err(KernelError::InvalidInput(vec![format!("surface {} b_U must be in [0,1]", surface.id)])); }
                h_u += c.u_value_w_m2k * opaque_area * b;
            }
            Boundary::AdjacentHeatedZone => {
                if surface.adjacent_zone_id.is_none() { return Err(KernelError::MissingAdjacentZone(surface.id.clone())); }
                h_adjacent += c.u_value_w_m2k * opaque_area;
            }
        }
    }

    for opening in &zone.openings {
        if opening.boundary == Boundary::Outdoor {
            h_d += opening.u_value_w_m2k * opening.area_m2;
        }
    }

    for bridge in &zone.thermal_bridges {
        match bridge.kind {
            crate::input::ThermalBridgeKind::Linear => {
                let psi = bridge.psi_w_mk.ok_or_else(|| KernelError::InvalidInput(vec!["linear thermal bridge missing psi".into()]))?;
                let length = bridge.length_m.ok_or_else(|| KernelError::InvalidInput(vec!["linear thermal bridge missing length".into()]))?;
                if psi < 0.0 || length < 0.0 || !psi.is_finite() || !length.is_finite() { return Err(KernelError::InvalidInput(vec!["invalid linear thermal bridge".into()])); }
                h_bridges += psi * length;
            }
            crate::input::ThermalBridgeKind::Point => {
                let chi = bridge.chi_w_k.ok_or_else(|| KernelError::InvalidInput(vec!["point thermal bridge missing chi".into()]))?;
                let count = bridge.count.ok_or_else(|| KernelError::InvalidInput(vec!["point thermal bridge missing count".into()]))?;
                if chi < 0.0 || count < 0.0 || !chi.is_finite() || !count.is_finite() { return Err(KernelError::InvalidInput(vec!["invalid point thermal bridge".into()])); }
                h_bridges += chi * count;
            }
        }
    }

    let month_hours = [744.0, 672.0, 744.0, 720.0, 744.0, 720.0, 744.0, 744.0, 720.0, 744.0, 720.0, 744.0];
    let annual_hours: f64 = month_hours.iter().sum();
    let annual_outdoor = project.climate.outdoor_temperature_c.iter().zip(month_hours.iter()).map(|(t, h)| t * h).sum::<f64>() / annual_hours;
    let mut monthly = [0.0; 12];
    let adjacent_by_id: HashMap<&str, &ZoneInput> = project.zones.iter().map(|z| (z.id.as_str(), z)).collect();
    for i in 0..12 {
        let theta_i = zone.indoor_temperature_c[i];
        let theta_e = project.climate.outdoor_temperature_c[i];
        let h_out = h_d + h_bridges;
        let q_out = h_out * (theta_i - theta_e) * 0.001 * month_hours[i];
        let q_u = h_u * (theta_i - theta_e) * 0.001 * month_hours[i];
        let q_g = h_ground * (theta_i - annual_outdoor) * 0.001 * month_hours[i];
        let mut q_a = 0.0;
        for surface in &zone.surfaces {
            if surface.boundary != Boundary::AdjacentHeatedZone { continue; }
            let adjacent = adjacent_by_id.get(surface.adjacent_zone_id.as_deref().unwrap_or(""))
                .ok_or_else(|| KernelError::MissingAdjacentZone(surface.id.clone()))?;
            let construction = constructions.get(surface.construction_id.as_str())
                .ok_or_else(|| KernelError::UnknownConstruction(surface.construction_id.clone()))?;
            let c = calculate_construction(construction)?;
            let openings = opening_area_by_surface.get(surface.id.as_str()).copied().unwrap_or(0.0);
            q_a += c.u_value_w_m2k * (surface.area_m2 - openings).max(0.0) * (theta_i - adjacent.indoor_temperature_c[i]) * 0.001 * month_hours[i];
        }
        monthly[i] = q_out + q_u + q_g + q_a;
    }
    Ok(TransmissionResult {
        h_d_w_k: h_d,
        h_u_w_k: h_u,
        h_ground_w_k: h_ground,
        h_adjacent_w_k: h_adjacent,
        h_thermal_bridges_w_k: h_bridges,
        monthly_q_tr_kwh: monthly,
        annual_q_tr_kwh: monthly.iter().sum(),
    })
}
