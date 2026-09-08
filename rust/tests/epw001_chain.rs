//! EPW001a probe: exercise the real OpenAEC transmission -> ventilation -> demand APIs.

use std::collections::HashMap;

use nta8800_demand::{calculate_demand, CoolingSetpoint, HeatingSetpoint, InternalGains, ThermalMassInput};
use nta8800_model::geometry::Window;
use nta8800_model::location::{Orientation, Tilt};
use nta8800_model::time::MonthlyProfile;
use nta8800_model::zoning::{Rekenzone, UsageFunction};
use nta8800_tables::climate::de_bilt::de_bilt_climate_data;
use nta8800_transmission::{calculate_transmission, BoundaryType, TransmissionElement};
use nta8800_ventilation::VentilationResult;

fn slab_on_ground_conductance(floor_area_m2: f64, perimeter_m: f64, floor_u_value: f64) -> f64 {
    if !(floor_area_m2 > 0.0 && perimeter_m > 0.0 && floor_u_value > 0.0) {
        return 0.0;
    }
    const LAMBDA_GROUND_W_PER_MK: f64 = 2.0;
    const R_SE_GROUND_M2K_PER_W: f64 = 0.04;
    const WALL_THICKNESS_M: f64 = 0.5;
    let b_prime = floor_area_m2 / (0.5 * perimeter_m);
    let r_si_plus_rc = 1.0 / floor_u_value;
    let d_equi = WALL_THICKNESS_M
        + LAMBDA_GROUND_W_PER_MK * (r_si_plus_rc + R_SE_GROUND_M2K_PER_W);
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
    // EPW001a: linear thermal bridges are forfaitarily accounted for.
    const U_FOR: f64 = 0.10;
    vec![
        TransmissionElement {
            id: "dak".into(), area: 48.0, u_value: 0.162 + U_FOR,
            boundary_type: BoundaryType::Outdoor, construction_id: None,
        },
        TransmissionElement {
            id: "vloer".into(), area: 48.0, u_value: 1.0 / (6.0 + 0.21),
            boundary_type: BoundaryType::Ground, construction_id: None,
        },
        TransmissionElement {
            id: "gevel-zuid".into(), area: 19.2, u_value: 0.162 + U_FOR,
            boundary_type: BoundaryType::Outdoor, construction_id: None,
        },
        TransmissionElement {
            id: "gevel-west".into(), area: 32.4, u_value: 0.162 + U_FOR,
            boundary_type: BoundaryType::Outdoor, construction_id: None,
        },
        TransmissionElement {
            id: "gevel-oost".into(), area: 32.4, u_value: 0.162 + U_FOR,
            boundary_type: BoundaryType::Outdoor, construction_id: None,
        },
        TransmissionElement {
            id: "gevel-noord".into(), area: 43.2, u_value: 0.162 + U_FOR,
            boundary_type: BoundaryType::Outdoor, construction_id: None,
        },
        TransmissionElement {
            id: "ramen-zuid".into(), area: 24.0, u_value: 1.8,
            boundary_type: BoundaryType::Outdoor, construction_id: None,
        },
    ]
}

fn windows() -> Vec<Window> {
    (1..=4)
        .map(|i| {
            Window::new(
                format!("raam-{i}"), "EPW001a-raam", 6.0, Orientation::Zuid,
                Tilt::VERTICAL, 1.8, 0.7, 0.25,
            )
            .expect("valid EPW001a window")
        })
        .collect()
}

/// NTA 8800 §11.2.1 pressure-balance model for EPW001a's infiltration paths.
///
/// This is deliberately kept in the EDR probe until the upstream OpenAEC
/// ventilation crate exposes the same pressure-balance model. The path split,
/// pressure equations and solver follow the 2026 chapter-11 text rather than
/// fitting an effective infiltration flow to the EDR result.
fn epw001a_monthly_infiltration(
    climate: &nta8800_model::ClimateData,
    indoor_temperature_c: f64,
    mechanical_supply_m3_h: f64,
    mechanical_exhaust_m3_h: f64,
    wtw_efficiency: f64,
) -> MonthlyProfile<f64> {
    const RHO_REF: f64 = 1.205;
    const T_REF_K: f64 = 293.0;
    const G: f64 = 9.81;
    const H: f64 = 5.4;
    const H_WINDWARD: f64 = 2.7;
    const N_LEA: f64 = 0.67;
    // EPW001a: qv10;spec;reken = 0.70 dm³/(s·m²), ftype = 1.4, fy = 0.7 (bouwjaar >= 2010).
    let qv10_lea_ref = 0.70 * 1.4 * 0.7;
    let qv1_lea_ref = qv10_lea_ref * 96.0 * 3.6 / 10.0_f64.powf(N_LEA);
    let paths = [
        (0.40 * qv1_lea_ref, H_WINDWARD, 0.25_f64),
        (0.40 * qv1_lea_ref, H_WINDWARD, -0.50_f64),
        (0.20 * qv1_lea_ref, H, -0.60_f64),
    ];
    let rho = |t_k: f64| RHO_REF * T_REF_K / t_k;

    MonthlyProfile::new(std::array::from_fn(|idx| {
        let month = nta8800_model::time::Month::all()[idx];
        let theta_e = climate.outdoor_temperature[month];
        let wind = climate.wind_speed[month];
        let theta_i_k = indoor_temperature_c + 273.0;
        let theta_e_k = theta_e + 273.0;
        let rho_e = rho(theta_e_k);
        let rho_i = rho(theta_i_k);
        let theta_supply = if theta_e < indoor_temperature_c {
            theta_e + wtw_efficiency * (indoor_temperature_c - theta_e)
        } else {
            theta_e
        };
        let rho_supply = rho(theta_supply + 273.0);

        let external_pressure = |h: f64, cp: f64| {
            RHO_REF * (T_REF_K / theta_e_k) * (0.5 * cp * wind * wind - h * G)
        };
        let internal_pressure_at = |p_ref: f64, h: f64| {
            p_ref - RHO_REF * h * G * T_REF_K / theta_i_k
        };

        let mass_sum = |p_ref: f64| {
            let mut sum = rho_supply * mechanical_supply_m3_h
                - rho_i * mechanical_exhaust_m3_h;
            let mut q_in = 0.0;
            for (c, h, cp) in paths {
                let dp = external_pressure(h, cp) - internal_pressure_at(p_ref, h);
                if dp > 0.0 {
                    let q = c * dp.powf(N_LEA);
                    sum += rho_e * q;
                    q_in += q;
                } else if dp < 0.0 {
                    let q = c * (-dp).powf(N_LEA);
                    sum -= rho_i * q;
                }
            }
            (sum, q_in)
        };

        let external_pressures: Vec<f64> = paths
            .iter()
            .map(|(_, h, cp)| external_pressure(*h, *cp))
            .collect();
        let mut pa = (external_pressures.iter().copied().fold(f64::INFINITY, f64::min)
            + external_pressures.iter().copied().fold(f64::NEG_INFINITY, f64::max))
            / 2.0
            + rho_i * H_WINDWARD * G;
        let mut ma = mass_sum(pa).0;
        const ACCURACY_KG_PER_H: f64 = 0.9;
        if ma.abs() <= ACCURACY_KG_PER_H {
            return mass_sum(pa).1;
        }

        let mut pb = pa + 2.0;
        let mut mb = mass_sum(pb).0;
        if mb.abs() > ACCURACY_KG_PER_H {
            for _ in 0..100 {
                if ma.signum() != mb.signum() {
                    break;
                }
                let r = (pb - pa).signum() * (mb - ma).signum();
                if ma.abs() > mb.abs() {
                    pa = pb;
                    ma = mb;
                }
                pb = pa - 2.0 * ma.signum() * r;
                mb = mass_sum(pb).0;
                if mb.abs() <= ACCURACY_KG_PER_H {
                    return mass_sum(pb).1;
                }
            }
        }

        let mut pc = (pa + pb) / 2.0;
        for _ in 0..200 {
            pc = (pa + pb) / 2.0;
            let mc = mass_sum(pc).0;
            if mc.abs() <= ACCURACY_KG_PER_H {
                break;
            }
            if mc.signum() == ma.signum() {
                pa = pc;
                ma = mc;
            } else {
                pb = pc;
            }
        }
        mass_sum(pc).1
    }))
}

fn epw001a_ventilation_result(
    climate: &nta8800_model::ClimateData,
    indoor_temperature_c: f64,
) -> VentilationResult {
    const RHO_C: f64 = 1212.23;
    const MECHANICAL_FLOW_M3_H: f64 = 311.04;
    const WTW_EFF: f64 = 0.80;
    const SFP_W_PER_M3_H: f64 = 0.45 / 3.6;
    let infiltration = epw001a_monthly_infiltration(
        climate,
        indoor_temperature_c,
        MECHANICAL_FLOW_M3_H,
        MECHANICAL_FLOW_M3_H,
        WTW_EFF,
    );

    let mut monthly_q_v = [0.0; 12];
    let mut monthly_w_fan = [0.0; 12];
    let mut monthly_wtw_recovery = [0.0; 12];
    let months = nta8800_model::time::Month::all();
    let month_hours = [744.0, 672.0, 744.0, 720.0, 744.0, 720.0, 744.0, 744.0, 720.0, 744.0, 720.0, 744.0];

    for idx in 0..12 {
        let month = months[idx];
        let theta_e = climate.outdoor_temperature[month];
        let hours = month_hours[idx];
        let theta_supply = if theta_e < indoor_temperature_c {
            theta_e + WTW_EFF * (indoor_temperature_c - theta_e)
        } else {
            theta_e
        };
        let q_mech = if theta_e < indoor_temperature_c {
            MECHANICAL_FLOW_M3_H * RHO_C * (indoor_temperature_c - theta_supply) * hours / 1_000_000.0
        } else {
            0.0
        };
        let q_inf = if theta_e < indoor_temperature_c {
            infiltration[month] * RHO_C * (indoor_temperature_c - theta_e) * hours / 1_000_000.0
        } else {
            0.0
        };
        monthly_q_v[idx] = q_mech + q_inf;
        monthly_wtw_recovery[idx] = if theta_e < indoor_temperature_c {
            MECHANICAL_FLOW_M3_H * RHO_C * WTW_EFF * (indoor_temperature_c - theta_e) * hours / 1_000_000.0
        } else {
            0.0
        };
        monthly_w_fan[idx] = SFP_W_PER_M3_H * 2.0 * MECHANICAL_FLOW_M3_H * hours * 3600.0 / 1_000_000.0;
    }

    let annual_q_v = monthly_q_v.iter().sum();
    let annual_w_fan = monthly_w_fan.iter().sum();
    let annual_wtw_recovery = monthly_wtw_recovery.iter().sum();
    VentilationResult {
        monthly_q_v: MonthlyProfile::new(monthly_q_v),
        annual_q_v,
        monthly_w_fan: MonthlyProfile::new(monthly_w_fan),
        annual_w_fan,
        monthly_wtw_recovery: MonthlyProfile::new(monthly_wtw_recovery),
        annual_wtw_recovery,
    }
}

#[test]
fn epw001a_transmission_ventilation_demand_probe() {
    let zone = zone();
    let climate = de_bilt_climate_data();
    let indoor = MonthlyProfile::from_constant(20.0);
    let floor_u = 1.0 / (6.0 + 0.21);
    let h_g_an = slab_on_ground_conductance(48.0, 28.0, floor_u);
    let transmission = calculate_transmission(
        &zone, &transmission_elements(), &[], &[], &indoor, &climate, h_g_an,
        &HashMap::new(), &HashMap::new(),
    ).expect("EPW001a transmission calculation should succeed");

    let ventilation = epw001a_ventilation_result(&climate, 20.0);

    let windows_owned = windows();
    let window_refs: Vec<&Window> = windows_owned.iter().collect();
    let internal = InternalGains::forfaitair(UsageFunction::Woonfunctie);
    let heating_sp = HeatingSetpoint::new(MonthlyProfile::from_constant(20.0));
    let cooling_sp = CoolingSetpoint::new(MonthlyProfile::from_constant(24.0));
    let demand = calculate_demand(
        &zone, &transmission, &ventilation, 311.04 * 1212.23 / 3600.0,
        &window_refs, &climate, heating_sp, cooling_sp, &internal,
        ThermalMassInput::zwaar_massief(), 1.0,
    ).expect("EPW001a demand calculation should succeed");

    let qh_kwh = demand.annual_heating_demand / 3.6;
    let qh_per_m2 = qh_kwh / zone.floor_area;
    let reference_qh_per_m2 = 42.69_f64;
    let relative_error = (qh_per_m2 - reference_qh_per_m2).abs() / reference_qh_per_m2;

    eprintln!(
        "EPW001a diagnostic: H_D={:.6} W/K; H_g={:.6} W/K; Q_T;an={:.6} MJ; Q_V;an={:.6} MJ; W_fan;an={:.6} MJ; Q_H;nd;an={:.6} kWh; Q_H;nd;net={:.6} kWh/m²; reference={:.6}; error={:.3}%",
        transmission.h_d, transmission.h_g_an, transmission.annual_q_t, ventilation.annual_q_v,
        ventilation.annual_w_fan, qh_kwh, qh_per_m2, reference_qh_per_m2, relative_error * 100.0
    );
    assert!(qh_per_m2.is_finite() && qh_per_m2 > 0.0);
}
