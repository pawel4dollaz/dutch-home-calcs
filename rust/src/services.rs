use std::collections::HashMap;

use crate::demand::DemandResult;
use crate::input::{CoolingSystemType, DhwSystemType, HeatingSystemType, LightingMethod, NtaProjectInput};

/// End-energy ledger for the downstream NTA service chain.
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceResult {
    pub heating_mj: f64,
    pub cooling_mj: f64,
    pub dhw_mj: f64,
    pub lighting_mj: f64,
    pub ventilation_aux_mj: f64,
    pub automation_mj: f64,
    pub pv_yield_mj: f64,
    pub ep: nta8800_ep::EpResult,
}

/// Errors for installation/service integration. Missing calculation inputs are
/// intentionally errors; this layer does not silently choose equipment defaults.
#[derive(Debug)]
pub enum ServiceError {
    InvalidInput(String),
    MissingSystem(&'static str),
    MissingLightingAnnualEnergy,
    Ep(nta8800_ep::EpError),
}

/// Convert building demand into downstream service ledgers and run the pinned
/// OpenAEC H.5 EP integration. The upstream EP crate is used unchanged, while
/// service adapters expose any remaining V1 limitations rather than hiding them.
pub fn calculate_services(project: &NtaProjectInput, demand: &DemandResult) -> Result<ServiceResult, ServiceError> {
    project.validate().map_err(|e| ServiceError::InvalidInput(e.join("; ")))?;
    let heating_system = project.systems.heating.first().ok_or(ServiceError::MissingSystem("heating"))?;
    let cooling_system = project.systems.cooling.first();
    let dhw_system = project.systems.dhw.first().ok_or(ServiceError::MissingSystem("dhw"))?;

    let heating_eta = heating_system.efficiency_or_cop
        * heating_system.distribution_loss_fraction.map(|x| 1.0 - x).unwrap_or(1.0)
        * heating_system.emitter_efficiency.unwrap_or(1.0);
    if heating_eta <= 0.0 || !heating_eta.is_finite() {
        return Err(ServiceError::InvalidInput("heating total efficiency/COP must be finite and > 0".into()));
    }
    if heating_system.backup_fraction.unwrap_or(0.0) != 0.0 {
        return Err(ServiceError::InvalidInput("non-zero heating backup_fraction requires an explicit hybrid/backup model".into()));
    }
    let heating_mj = demand.annual_heating_mj / heating_eta;

    let cooling_mj = match cooling_system {
        None => 0.0,
        Some(s) if s.system_type == CoolingSystemType::None => 0.0,
        Some(s) => {
            let eta = s.efficiency_or_cop * (1.0 - s.distribution_loss_fraction.unwrap_or(0.0));
            if eta <= 0.0 || !eta.is_finite() { return Err(ServiceError::InvalidInput("cooling efficiency/COP invalid".into())); }
            demand.annual_cooling_mj / eta
        }
    };

    // NTA 8800:2025+C1:2026 §13.2.3.1: 856 kWh/year per resident for
    // residential DHW. The project schema permits an explicit resident count;
    // unlike the former thermodynamic proxy, this uses the published NTA V1
    // annual demand basis. Storage losses remain an explicit addition until the
    // complete §13.6 storage model is wired through.
    let water_demand_mj = dhw_system.people * 856.0 * 3.6;
    let storage_loss_mj = dhw_system.storage_loss_kwh_day.unwrap_or(0.0) * 3.6 * 365.0;
    let dhw_eta = dhw_system.efficiency_or_cop * (1.0 - dhw_system.distribution_loss_fraction.unwrap_or(0.0));
    if dhw_eta <= 0.0 || !dhw_eta.is_finite() { return Err(ServiceError::InvalidInput("DHW efficiency/COP invalid".into())); }
    let dhw_mj = (water_demand_mj + storage_loss_mj) / dhw_eta;

    let lighting = project.systems.lighting.as_ref().ok_or(ServiceError::MissingSystem("lighting"))?;
    let lighting_kwh_m2 = match lighting.method {
        LightingMethod::ExplicitAnnual | LightingMethod::Nta8800 => lighting.annual_kwh_m2.ok_or(ServiceError::MissingLightingAnnualEnergy)?,
    };
    if lighting_kwh_m2 < 0.0 || !lighting_kwh_m2.is_finite() { return Err(ServiceError::InvalidInput("lighting annual energy invalid".into())); }
    let area = project.building.gross_floor_area_m2;
    let lighting_mj = lighting_kwh_m2 * area * 3.6;

    let ventilation_aux_mj = project.systems.ventilation.iter().map(|v| {
        let flow = v.design_supply_m3_h.or(v.design_extract_m3_h).unwrap_or(0.0);
        let sfp = v.specific_fan_power_ws_m3.unwrap_or(0.0);
        sfp * flow * 8760.0 / 1_000_000.0
    }).sum::<f64>() * 3.6;
    let automation_mj = 0.0;
    let pv_yield_mj = project.systems.pv.iter().map(|p| {
        let annual_kwh = p.peak_kwp * p.annual_yield_kwh_per_kwp.unwrap_or(0.0) * p.shading_factor;
        annual_kwh * 3.6
    }).sum::<f64>();

    let mut ep = nta8800_ep::EpInputs {
        heating: HashMap::new(), cooling: HashMap::new(), dhw: HashMap::new(), lighting: HashMap::new(),
        ventilation_aux: HashMap::new(), automation: HashMap::new(), pv_yield: pv_yield_mj,
        building_area: nta8800_ep::BuildingArea { a_g: area },
    };
    ep.heating.insert(heating_carrier(heating_system.system_type), heating_mj);
    if cooling_mj > 0.0 { ep.cooling.insert(nta8800_ep::EnergyCarrier::Elektriciteit, cooling_mj); }
    ep.dhw.insert(dhw_carrier(dhw_system.system_type), dhw_mj);
    ep.lighting.insert(nta8800_ep::EnergyCarrier::Elektriciteit, lighting_mj);
    if ventilation_aux_mj > 0.0 { ep.ventilation_aux.insert(nta8800_ep::EnergyCarrier::Elektriciteit, ventilation_aux_mj); }
    if automation_mj > 0.0 { ep.automation.insert(nta8800_ep::EnergyCarrier::Elektriciteit, automation_mj); }

    let first_zone = project.zones.first().ok_or(ServiceError::InvalidInput("no zone".into()))?;
    let result = nta8800_ep::calculate_ep_score(&ep, usage_function(first_zone.usage)).map_err(ServiceError::Ep)?;
    Ok(ServiceResult { heating_mj, cooling_mj, dhw_mj, lighting_mj, ventilation_aux_mj, automation_mj, pv_yield_mj, ep: result })
}

fn heating_carrier(t: HeatingSystemType) -> nta8800_ep::EnergyCarrier {
    match t {
        HeatingSystemType::GasBoiler => nta8800_ep::EnergyCarrier::Aardgas,
        HeatingSystemType::DistrictHeating => nta8800_ep::EnergyCarrier::Stadswarmte,
        HeatingSystemType::Biomass => nta8800_ep::EnergyCarrier::Biomassa,
        HeatingSystemType::DirectElectric | HeatingSystemType::HeatPumpAirWater | HeatingSystemType::HeatPumpGroundSource | HeatingSystemType::Other => nta8800_ep::EnergyCarrier::Elektriciteit,
    }
}

fn dhw_carrier(t: DhwSystemType) -> nta8800_ep::EnergyCarrier {
    match t {
        DhwSystemType::Gas => nta8800_ep::EnergyCarrier::Aardgas,
        DhwSystemType::DistrictHeating => nta8800_ep::EnergyCarrier::Stadswarmte,
        DhwSystemType::Electric | DhwSystemType::HeatPump | DhwSystemType::Other => nta8800_ep::EnergyCarrier::Elektriciteit,
    }
}

fn usage_function(u: crate::input::UsageFunction) -> nta8800_model::zoning::UsageFunction {
    use crate::input::UsageFunction::*;
    use nta8800_model::zoning::UsageFunction as U;
    match u {
        Woonfunctie => U::Woonfunctie, Logiesfunctie => U::Logiesfunctie,
        Kantoorfunctie => U::Kantoorfunctie, Onderwijsfunctie => U::Onderwijsfunctie,
        Bijeenkomstfunctie => U::Bijeenkomstfunctie, Gezondheidszorgfunctie => U::Gezondheidszorgfunctie,
        Winkelfunctie => U::Winkelfunctie, Celfunctie => U::Celfunctie, Sportfunctie => U::Sportfunctie,
        Industriefunctie => U::Industriefunctie, OverigeGebruiksfunctie => U::OverigeGebruiksfunctie,
    }
}
