//! EPW001a / EPW007 diagnostic probe.
//! The pressure-balance equations follow NTA 8800 §11.2.1.5/11.2.1.6.
//! This remains non-gating until all EPW001 inputs are confirmed from the EDR form.

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
const H: f64 = 5.4;
const ETA_HR: f64 = 0.80;
const N: f64 = 0.67;
const RHO_REF: f64 = 1.205;
const T_REF: f64 = 293.0;
const G: f64 = 9.81;
const RHO_CP: f64 = 1212.23; // J/(m3 K)
const SFP: f64 = 0.45 / 3.6;

fn zone() -> Rekenzone {
    Rekenzone { id:"EPW001a-rz1".into(), name:"EPW001a".into(), gebouw_id:"EPW001a".into(), floor_area:AG, volume:259.2, efr_ids:vec!["woonfunctie".into()], constructions:vec![], windows:vec![], openings:vec![], thermal_bridges_linear:vec![], thermal_bridges_point:vec![] }
}
fn elements() -> Vec<TransmissionElement> {
    let u=0.162+0.10;
    vec![
      TransmissionElement{id:"dak".into(),area:48.0,u_value:u,boundary_type:BoundaryType::Outdoor,construction_id:None},
      TransmissionElement{id:"vloer".into(),area:48.0,u_value:1.0/6.21,boundary_type:BoundaryType::Ground,construction_id:None},
      TransmissionElement{id:"zuid".into(),area:19.2,u_value:u,boundary_type:BoundaryType::Outdoor,construction_id:None},
      TransmissionElement{id:"west".into(),area:32.4,u_value:u,boundary_type:BoundaryType::Outdoor,construction_id:None},
      TransmissionElement{id:"oost".into(),area:32.4,u_value:u,boundary_type:BoundaryType::Outdoor,construction_id:None},
      TransmissionElement{id:"noord".into(),area:43.2,u_value:u,boundary_type:BoundaryType::Outdoor,construction_id:None},
      TransmissionElement{id:"ramen".into(),area:24.0,u_value:1.8,boundary_type:BoundaryType::Outdoor,construction_id:None},
    ]
}
fn windows()->Vec<Window>{(1..=4).map(|i|Window::new(format!("w{i}"),"w",6.0,Orientation::Zuid,Tilt::VERTICAL,1.8,0.7,0.25).unwrap()).collect()}
fn ground_hg()->f64{
    let a=48.0; let p=28.0; let u=1.0/6.21; let b=a/(0.5*p); let d=0.5+2.0*(1.0/u+0.04);
    a*(4.0/(std::f64::consts::PI*b+d)*(std::f64::consts::PI*b/d+1.0).ln())
}
fn qv1(qv10_ref:f64)->f64{qv10_ref*AG*3.6/10.0_f64.powf(N)}

fn infiltration(climate:&nta8800_model::ClimateData,qv10_ref:f64,qv_mech:f64)->MonthlyProfile<f64>{
    let q=qv1(qv10_ref); let paths=[(0.40*q,0.5*H,0.25),(0.40*q,0.5*H,-0.50),(0.20*q,H,-0.60)];
    let rho=|tk:f64|RHO_REF*T_REF/tk;
    MonthlyProfile::new(std::array::from_fn(|i|{
      let m=Month::all()[i]; let te=climate.outdoor_temperature[m]; let te_k=te+273.0; let ti=293.0; let ri=rho(ti); let re=rho(te_k); let wind=climate.wind_speed[m];
      let pe=|h:f64,cp:f64|RHO_REF*(T_REF/te_k)*(0.5*cp*wind*wind-h*G);
      let pz=|pref:f64,h:f64|pref-RHO_REF*h*G*T_REF/ti;
      let mb=|pref:f64|{
        let ts=if te<20.0{te+ETA_HR*(20.0-te)}else{te}; let rs=rho(ts+273.0); let mut sum=rs*qv_mech-ri*qv_mech; let mut qin=0.0;
        for (c,h,cp) in paths{let dp=pe(h,cp)-pz(pref,h);if dp>0.0{let x=c*dp.powf(N);qin+=x;sum+=re*x}else if dp<0.0{sum-=ri*c*(-dp).powf(N)}}(sum,qin)
      };
      let pes:Vec<f64>=paths.iter().map(|(_,h,cp)|pe(*h,*cp)).collect();
      let mut pa=(pes.iter().copied().fold(f64::INFINITY,f64::min)+pes.iter().copied().fold(f64::NEG_INFINITY,f64::max))/2.0+ri*(0.5*H)*G; let mut ma=mb(pa).0;
      if ma.abs()<=0.9{return mb(pa).1}; let mut pb=pa+2.0; let mut mbb=mb(pb).0;
      while mbb.abs()>0.9&&ma.signum()==mbb.signum(){let r=(pb-pa).signum()*(mbb-ma).signum();if ma.abs()>mbb.abs(){pa=pb;ma=mbb}pb=pa-2.0*ma.signum()*r;mbb=mb(pb).0}
      if mbb.abs()<=0.9{return mb(pb).1};
      for _ in 0..200{let pc=(pa+pb)/2.0;let mc=mb(pc).0;if mc.abs()<=0.9{return mb(pc).1};if mc.signum()==ma.signum(){pa=pc;ma=mc}else{pb=pc}}
      mb((pa+pb)/2.0).1
    }))
}

fn ventilation(climate:&nta8800_model::ClimateData,qv10_ref:f64,qv_mech:f64)->VentilationResult{
    let inf=infiltration(climate,qv10_ref,qv_mech);let mut qv=[0.0;12];let mut wf=[0.0;12];let mut wr=[0.0;12];
    let hours=[744.,672.,744.,720.,744.,720.,744.,744.,720.,744.,720.,744.];
    for m in Month::all(){let i=m.index();let te=climate.outdoor_temperature[m];let h=hours[i];let dt=(20.0-te).max(0.0);let ts=if te<20.0{te+ETA_HR*dt}else{te};
      qv[i]=qv_mech*RHO_CP*(20.0-ts).max(0.0)*h/1e6+inf[m]*RHO_CP*dt*h/1e6;
      wf[i]=SFP*2.0*qv_mech*h*3600.0/1e6;
      wr[i]=qv_mech*RHO_CP*ETA_HR*dt*h/1e6;
    }
    VentilationResult{monthly_q_v:MonthlyProfile::new(qv),annual_q_v:qv.iter().sum(),monthly_w_fan:MonthlyProfile::new(wf),annual_w_fan:wf.iter().sum(),monthly_wtw_recovery:MonthlyProfile::new(wr),annual_wtw_recovery:wr.iter().sum()}
}

fn qh(zone:&Rekenzone,climate:&nta8800_model::ClimateData,tr:&nta8800_transmission::TransmissionResult,qv10:f64,qv_mech:f64)->(f64,f64){
    let v=ventilation(climate,qv10,qv_mech);let w=windows();let wr:Vec<&Window>=w.iter().collect();
    let d=calculate_demand(zone,tr,&v,qv_mech*1212.23/3600.0,&wr,climate,HeatingSetpoint::new(MonthlyProfile::from_constant(20.0)),CoolingSetpoint::new(MonthlyProfile::from_constant(24.0)),&InternalGains::forfaitair(UsageFunction::Woonfunctie),ThermalMassInput::zwaar_massief(),1.0).unwrap();
    (d.annual_heating_demand/3.6/AG,v.annual_q_v)
}

#[test]
fn epw001a_infiltration_differential_probe(){
    let z=zone();let c=de_bilt_climate_data();let indoor=MonthlyProfile::from_constant(20.0);let tr=calculate_transmission(&z,&elements(),&[],&[],&indoor,&c,ground_hg(),&HashMap::new(),&HashMap::new()).unwrap();
    let flows=[("required",0.50*0.40*1.05*AG*3.6),("installed-0.9",0.90*AG*3.6)];
    for (flow_name,qflow) in flows{for (name,qv10,reference) in [("EPW007c",0.20,37.98),("EPW001a",0.98,42.69),("EPW007a-input",2.94,f64::NAN)]{let(q,e)=qh(&z,&c,&tr,qv10,qflow);let err=if reference.is_finite(){(q-reference).abs()/reference*100.0}else{f64::NAN};eprintln!("flow={flow_name} qv_mech={qflow:.3} m3/h; {name}: Q_V;an={e:.1} MJ; Q_H;nd;net={q:.6} kWh/m²; EDR={reference:.2}; error={err:.3}%");}}
    let (baseline,_) = qh(&z,&c,&tr,0.98,0.90*AG*3.6);assert!(baseline.is_finite()&&baseline>0.0);
}
