mod epw001_chain {
    include!("epw001_chain.rs");

    #[test]
    fn epw001a_dump_result() {
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
        let demand = calculate_demand(
            &zone, &transmission, &ventilation, 311.04 * 1212.23 / 3600.0,
            &window_refs, &climate,
            HeatingSetpoint::new(MonthlyProfile::from_constant(20.0)),
            CoolingSetpoint::new(MonthlyProfile::from_constant(24.0)),
            &internal, ThermalMassInput::zwaar_massief(), 1.0,
        ).expect("EPW001a demand calculation should succeed");
        let qh_per_m2 = demand.annual_heating_demand / 3.6 / zone.floor_area;
        let error = (qh_per_m2 - 42.69).abs() / 42.69;
        assert!(
            error < 0.01,
            "EPW001a current result: Q_H;nd;net={qh_per_m2:.6} kWh/m²; Q_V;an={:.6} MJ; H_D={:.6} W/K; H_g={:.6} W/K; error={:.3}%",
            ventilation.annual_q_v, transmission.h_d, transmission.h_g_an, error * 100.0
        );
    }
}
