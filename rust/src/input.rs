use serde::{Deserialize, Serialize};

/// Twelve monthly values in calendar order, January through December.
pub type MonthlyValues = [f64; 12];

/// Top-level input document for the NTA 8800 kernel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NtaProjectInput {
    pub schema_version: String,
    pub nta_version: String,
    pub project_id: String,
    pub building: BuildingInput,
    pub climate: ClimateInput,
    pub zones: Vec<ZoneInput>,
    pub constructions: Vec<ConstructionInput>,
    pub systems: SystemsInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BuildingInput {
    pub name: String,
    pub construction_year: u16,
    pub location: LocationInput,
    pub gross_floor_area_m2: f64,
    pub volume_m3: f64,
    pub floors: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocationInput {
    pub postcode: Option<String>,
    pub municipality: Option<String>,
    pub latitude_deg: f64,
    pub longitude_deg: f64,
    pub orientation_reference_deg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClimateInput {
    pub source_id: String,
    pub source_version: String,
    pub outdoor_temperature_c: MonthlyValues,
    pub wind_speed_m_s: MonthlyValues,
    /// Global/normal irradiation by orientation. The map key is an explicit
    /// orientation identifier; the kernel never silently invents a climate.
    pub solar_irradiation_kwh_m2: std::collections::BTreeMap<String, MonthlyValues>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ZoneInput {
    pub id: String,
    pub name: String,
    pub usage: UsageFunction,
    pub floor_area_m2: f64,
    pub volume_m3: f64,
    pub conditioned: bool,
    pub indoor_temperature_c: MonthlyValues,
    pub internal_gains_w_m2: Option<MonthlyValues>,
    pub thermal_mass_kj_m2k: Option<f64>,
    pub surfaces: Vec<SurfaceInput>,
    pub openings: Vec<OpeningInput>,
    pub thermal_bridges: Vec<ThermalBridgeInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SurfaceInput {
    pub id: String,
    pub name: String,
    pub area_m2: f64,
    pub boundary: Boundary,
    pub construction_id: String,
    pub orientation_deg: Option<f64>,
    pub tilt_deg: Option<f64>,
    pub adjacent_zone_id: Option<String>,
    pub unheated_b_factor: Option<f64>,
    pub ground_hg_an_w_k: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpeningInput {
    pub id: String,
    pub name: String,
    pub surface_id: Option<String>,
    pub area_m2: f64,
    pub u_value_w_m2k: f64,
    pub g_value: f64,
    pub boundary: Boundary,
    pub orientation_deg: f64,
    pub tilt_deg: f64,
    pub shading_factor: f64,
    pub frame_fraction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConstructionInput {
    pub id: String,
    pub name: String,
    pub r_si_m2k_w: f64,
    pub r_se_m2k_w: f64,
    pub layers: Vec<LayerInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayerInput {
    pub material: String,
    pub thickness_m: f64,
    pub lambda_w_mk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThermalBridgeInput {
    pub kind: ThermalBridgeKind,
    pub psi_w_mk: Option<f64>,
    pub chi_w_k: Option<f64>,
    pub length_m: Option<f64>,
    pub count: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemsInput {
    pub ventilation: Vec<VentilationInput>,
    pub heating: Vec<HeatingSystemInput>,
    pub cooling: Vec<CoolingSystemInput>,
    pub dhw: Vec<DhwSystemInput>,
    pub lighting: Option<LightingInput>,
    pub automation: Option<AutomationInput>,
    pub humidity: Option<HumidityInput>,
    pub pv: Vec<PvInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VentilationInput {
    pub id: String,
    pub system_type: VentilationSystemType,
    pub design_supply_m3_h: Option<f64>,
    pub design_extract_m3_h: Option<f64>,
    pub required_outdoor_air_m3_h: Option<f64>,
    pub heat_recovery_efficiency: Option<f64>,
    pub specific_fan_power_ws_m3: Option<f64>,
    pub control_factor: Option<f64>,
    pub duct_leakage_factor: Option<f64>,
    pub installed_capacity_m3_h: Option<f64>,
    pub bypass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeatingSystemInput {
    pub id: String,
    pub system_type: HeatingSystemType,
    pub efficiency_or_cop: f64,
    pub nominal_capacity_kw: Option<f64>,
    pub supply_temperature_c: Option<f64>,
    pub return_temperature_c: Option<f64>,
    pub distribution_loss_fraction: Option<f64>,
    pub emitter_efficiency: Option<f64>,
    pub backup_fraction: Option<f64>,
    pub bcrg_declaration_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoolingSystemInput {
    pub id: String,
    pub system_type: CoolingSystemType,
    pub efficiency_or_cop: f64,
    pub nominal_capacity_kw: Option<f64>,
    pub supply_temperature_c: Option<f64>,
    pub distribution_loss_fraction: Option<f64>,
    pub bcrg_declaration_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DhwSystemInput {
    pub id: String,
    pub system_type: DhwSystemType,
    pub efficiency_or_cop: f64,
    pub people: f64,
    pub litres_per_person_day: f64,
    pub storage_volume_l: Option<f64>,
    pub storage_loss_kwh_day: Option<f64>,
    pub distribution_loss_fraction: Option<f64>,
    pub bcrg_declaration_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LightingInput {
    pub method: LightingMethod,
    pub annual_kwh_m2: Option<f64>,
    pub installed_power_w_m2: Option<f64>,
    pub control_factor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationInput {
    pub control_class: String,
    pub heating_control_factor: Option<f64>,
    pub cooling_control_factor: Option<f64>,
    pub ventilation_control_factor: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HumidityInput {
    pub enabled: bool,
    pub humidification_kwh: Option<f64>,
    pub dehumidification_kwh: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PvInput {
    pub id: String,
    pub peak_kwp: f64,
    pub orientation_deg: f64,
    pub tilt_deg: f64,
    pub shading_factor: f64,
    pub annual_yield_kwh_per_kwp: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UsageFunction {
    Woonfunctie,
    Logiesfunctie,
    Kantoorfunctie,
    Onderwijsfunctie,
    Bijeenkomstfunctie,
    Gezondheidszorgfunctie,
    Winkelfunctie,
    Celfunctie,
    Sportfunctie,
    Industriefunctie,
    OverigeGebruiksfunctie,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Boundary {
    Outdoor,
    Ground,
    UnheatedSpace,
    AdjacentHeatedZone,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ThermalBridgeKind {
    Linear,
    Point,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VentilationSystemType {
    Natural,
    MechanicalExtract,
    MechanicalSupply,
    BalancedHeatRecovery,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HeatingSystemType {
    GasBoiler,
    DistrictHeating,
    DirectElectric,
    HeatPumpAirWater,
    HeatPumpGroundSource,
    Biomass,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CoolingSystemType {
    None,
    HeatPumpActive,
    DistrictCooling,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DhwSystemType {
    Gas,
    Electric,
    HeatPump,
    DistrictHeating,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LightingMethod {
    Nta8800,
    ExplicitAnnual,
}

impl NtaProjectInput {
    /// Validate structural and physical input constraints before calculation.
    /// Validation is deliberately fail-closed: missing information is reported
    /// instead of being replaced by an undocumented default.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.schema_version.trim().is_empty() { errors.push("schema_version is required".into()); }
        if self.nta_version.trim().is_empty() { errors.push("nta_version is required".into()); }
        if self.building.gross_floor_area_m2 <= 0.0 || !self.building.gross_floor_area_m2.is_finite() { errors.push("building.gross_floor_area_m2 must be finite and > 0".into()); }
        if self.building.volume_m3 <= 0.0 || !self.building.volume_m3.is_finite() { errors.push("building.volume_m3 must be finite and > 0".into()); }
        validate_monthly(&self.climate.outdoor_temperature_c, "climate.outdoor_temperature_c", &mut errors);
        validate_monthly(&self.climate.wind_speed_m_s, "climate.wind_speed_m_s", &mut errors);
        if self.zones.is_empty() { errors.push("at least one calculation zone is required".into()); }
        let area_sum: f64 = self.zones.iter().map(|z| z.floor_area_m2).sum();
        if (area_sum - self.building.gross_floor_area_m2).abs() > 1e-6 { errors.push(format!("zone floor-area sum {area_sum:.6} m² differs from building GFA {:.6} m²", self.building.gross_floor_area_m2)); }
        for zone in &self.zones { validate_zone(zone, &self.constructions, &mut errors); }
        for c in &self.constructions {
            if c.id.trim().is_empty() { errors.push("construction id is required".into()); }
            if c.layers.is_empty() { errors.push(format!("construction {} has no layers", c.id)); }
            for (i, layer) in c.layers.iter().enumerate() {
                if layer.thickness_m <= 0.0 || !layer.thickness_m.is_finite() { errors.push(format!("construction {} layer {i}: thickness must be > 0", c.id)); }
                if layer.lambda_w_mk <= 0.0 || !layer.lambda_w_mk.is_finite() { errors.push(format!("construction {} layer {i}: lambda must be > 0", c.id)); }
            }
        }
        for v in &self.systems.ventilation {
            validate_optional_nonnegative(v.design_supply_m3_h, "ventilation design supply", &v.id, &mut errors);
            validate_optional_nonnegative(v.design_extract_m3_h, "ventilation design extract", &v.id, &mut errors);
            validate_fraction(v.heat_recovery_efficiency, "ventilation heat recovery", &v.id, &mut errors);
            validate_optional_nonnegative(v.specific_fan_power_ws_m3, "ventilation SFP", &v.id, &mut errors);
        }
        for h in &self.systems.heating { if h.efficiency_or_cop <= 0.0 || !h.efficiency_or_cop.is_finite() { errors.push(format!("heating {} efficiency/COP must be > 0", h.id)); } }
        for c in &self.systems.cooling { if c.efficiency_or_cop <= 0.0 || !c.efficiency_or_cop.is_finite() { errors.push(format!("cooling {} efficiency/COP must be > 0", c.id)); } }
        for d in &self.systems.dhw {
            if d.people < 0.0 || !d.people.is_finite() { errors.push(format!("DHW {} people must be >= 0", d.id)); }
            if d.litres_per_person_day < 0.0 || !d.litres_per_person_day.is_finite() { errors.push(format!("DHW {} litres/person/day must be >= 0", d.id)); }
            if d.efficiency_or_cop <= 0.0 || !d.efficiency_or_cop.is_finite() { errors.push(format!("DHW {} efficiency/COP must be > 0", d.id)); }
        }
        if let Some(l) = &self.systems.lighting {
            if let Some(x) = l.annual_kwh_m2 { validate_scalar_nonnegative(x, "lighting annual kWh/m²", &mut errors); }
            if let Some(x) = l.installed_power_w_m2 { validate_scalar_nonnegative(x, "lighting installed W/m²", &mut errors); }
        }
        for pv in &self.systems.pv {
            if pv.peak_kwp < 0.0 || !pv.peak_kwp.is_finite() { errors.push(format!("PV {} peak kWp must be >= 0", pv.id)); }
            if !(0.0..=1.0).contains(&pv.shading_factor) || !pv.shading_factor.is_finite() { errors.push(format!("PV {} shading_factor must be in [0,1]", pv.id)); }
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}

fn validate_zone(zone: &ZoneInput, constructions: &[ConstructionInput], errors: &mut Vec<String>) {
    if zone.floor_area_m2 <= 0.0 || !zone.floor_area_m2.is_finite() { errors.push(format!("zone {} floor area must be > 0", zone.id)); }
    if zone.volume_m3 <= 0.0 || !zone.volume_m3.is_finite() { errors.push(format!("zone {} volume must be > 0", zone.id)); }
    validate_monthly(&zone.indoor_temperature_c, &format!("zone {} indoor temperature", zone.id), errors);
    if let Some(g) = &zone.internal_gains_w_m2 { validate_monthly(g, &format!("zone {} internal gains", zone.id), errors); }
    for surface in &zone.surfaces {
        if surface.area_m2 <= 0.0 || !surface.area_m2.is_finite() { errors.push(format!("zone {} surface {} area must be > 0", zone.id, surface.id)); }
        if !constructions.iter().any(|c| c.id == surface.construction_id) { errors.push(format!("zone {} surface {} references unknown construction {}", zone.id, surface.id, surface.construction_id)); }
        if let Some(b) = surface.unheated_b_factor { if !(0.0..=1.0).contains(&b) || !b.is_finite() { errors.push(format!("zone {} surface {} b_U must be in [0,1]", zone.id, surface.id)); } }
    }
    for opening in &zone.openings {
        if opening.area_m2 <= 0.0 || !opening.area_m2.is_finite() { errors.push(format!("zone {} opening {} area must be > 0", zone.id, opening.id)); }
        if opening.u_value_w_m2k <= 0.0 || !opening.u_value_w_m2k.is_finite() { errors.push(format!("zone {} opening {} U-value must be > 0", zone.id, opening.id)); }
        if !(0.0..=1.0).contains(&opening.g_value) || !opening.g_value.is_finite() { errors.push(format!("zone {} opening {} g-value must be in [0,1]", zone.id, opening.id)); }
        if !(0.0..=1.0).contains(&opening.shading_factor) || !opening.shading_factor.is_finite() { errors.push(format!("zone {} opening {} shading factor must be in [0,1]", zone.id, opening.id)); }
        if !(0.0..=1.0).contains(&opening.frame_fraction) || !opening.frame_fraction.is_finite() { errors.push(format!("zone {} opening {} frame fraction must be in [0,1]", zone.id, opening.id)); }
        if let Some(surface_id) = &opening.surface_id { if !zone.surfaces.iter().any(|s| s.id == *surface_id) { errors.push(format!("zone {} opening {} references unknown surface {}", zone.id, opening.id, surface_id)); } }
    }
}

fn validate_monthly(values: &MonthlyValues, name: &str, errors: &mut Vec<String>) { if values.iter().any(|x| !x.is_finite()) { errors.push(format!("{name} contains non-finite values")); } }
fn validate_fraction(value: Option<f64>, name: &str, id: &str, errors: &mut Vec<String>) { if let Some(x) = value { if !(0.0..=1.0).contains(&x) || !x.is_finite() { errors.push(format!("{name} for {id} must be in [0,1]")); } } }
fn validate_optional_nonnegative(value: Option<f64>, name: &str, id: &str, errors: &mut Vec<String>) { if let Some(x) = value { if x < 0.0 || !x.is_finite() { errors.push(format!("{name} for {id} must be >= 0")); } } }
fn validate_scalar_nonnegative(value: f64, name: &str, errors: &mut Vec<String>) { if value < 0.0 || !value.is_finite() { errors.push(format!("{name} must be >= 0")); } }
