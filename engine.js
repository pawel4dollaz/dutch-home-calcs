export const LABELS = [
  { label:'A++++', max:0 }, { label:'A+++', max:50 }, { label:'A++', max:75 },
  { label:'A+', max:105 }, { label:'A', max:160 }, { label:'B', max:190 },
  { label:'C', max:250 }, { label:'D', max:290 }, { label:'E', max:335 },
  { label:'F', max:380 }, { label:'G', max:Infinity }
];

const MONTH_DAYS=[31,28,31,30,31,30,31,31,30,31,30,31];
// Transparent screening climate: representative NL monthly outdoor temperatures (°C).
const MONTH_T=[3.5,3.7,6.0,9.0,12.5,15.5,17.5,17.5,14.5,10.5,6.5,4.0];
const MONTH_SOLAR=[18,32,58,92,122,126,128,105,76,43,23,16]; // kWh/m²/month global-horizontal proxy
const PRIMARY_ELEC=1.45; // screening factor; official factors can differ by version/context
const GAS_PRIMARY=1.0;

function n(v,d=0){ const x=Number(v); return Number.isFinite(x)?x:d; }
function clamp(v,a,b){return Math.min(b,Math.max(a,v));}
function sum(a){return a.reduce((x,y)=>x+y,0);}

export function uFromR(r){ return 1/Math.max(0.01,n(r,0.01)); }
export function labelFor(ep2){ return LABELS.find(x=>ep2<=x.max)?.label ?? 'G'; }
export const WWS_MULTI_FAMILY_POINTS={ 'A++++':58,'A+++':53,'A++':48,'A+':43,'A':37,'B':30,'C':15,'D':11,'E':-4,'F':-9,'G':-15 };
export function wwsEnergyPoints(label){ return WWS_MULTI_FAMILY_POINTS[label] ?? null; }

function monthlyTransmission(input){
  const htr = sum((input.envelope||[]).map(s=>n(s.area)*n(s.u))) + n(input.thermalBridgeLossWk,0);
  const losses=[]; const gains=[];
  for(let m=0;m<12;m++){
    const dt=Math.max(0,20-MONTH_T[m]);
    losses.push(htr*dt*MONTH_DAYS[m]*24/1000);
    const solar=sum((input.windows||[]).map(w=>n(w.area)*n(w.gValue,0.5)))*MONTH_SOLAR[m]/1000;
    const internal=(n(input.internalGainsW,3.0)*n(input.area)*MONTH_DAYS[m]*24/1000);
    gains.push(solar+internal);
  }
  return {htr,losses,gains};
}

function monthlyVentilation(input){
  const area=n(input.area); const volume=n(input.volume,area*2.5);
  const ach=n(input.ventilation.ach,0.7); const flow=volume*ach;
  const h=0.34*flow; const eta=clamp(n(input.ventilation.heatRecovery,0),0,1);
  const losses=[];
  for(let m=0;m<12;m++){
    const dt=Math.max(0,20-MONTH_T[m]);
    losses.push(h*(1-eta)*dt*MONTH_DAYS[m]*24/1000);
  }
  return {flow,losses};
}

function dhw(input){
  const people=n(input.dhw.people,2); const litres=n(input.dhw.litresPerPersonDay,35);
  const annualLitres=people*litres*365; const dT=50;
  const useful=annualLitres*4.186*dT/3600;
  const eff=clamp(n(input.dhw.efficiency,0.75),0.2,8.0);
  return {useful,inputEnergy: useful/eff};
}

function heatingEnergy(input, usefulDemand){
  const system=input.heating||{}; const type=system.type;
  let fuel=0, elec=0, renewableHeat=0;
  if(type==='heatpump'){
    const cop=clamp(n(system.seasonalCOP,3.5),1,8);
    elec=usefulDemand/cop; renewableHeat=usefulDemand-elec;
    if(n(system.backupFraction,0)>0){
      const frac=clamp(n(system.backupFraction),0,1); const backup=usefulDemand*frac;
      const backupElec=backup/cop; elec-=backupElec; fuel+=backup; renewableHeat-=Math.max(0,backup-backupElec);
    }
  } else if(type==='gas') fuel=usefulDemand/clamp(n(system.efficiency,0.9),0.5,1.2);
  else if(type==='district') fuel=usefulDemand/clamp(n(system.efficiency,0.95),0.5,1.2);
  else if(type==='electric') elec=usefulDemand;
  else if(type==='wood') fuel=usefulDemand/clamp(n(system.efficiency,0.8),0.5,1.2);
  else elec=usefulDemand;
  const distribution=clamp(n(system.distributionLossFraction,0.05),0,0.5);
  const gross=elec+fuel;
  elec += elec*distribution; fuel += fuel*distribution;
  return {fuel,elec,renewableHeat,generationInput:gross*(1+distribution)};
}

export function calculate(input){
  const area=clamp(n(input.area),10,10000);
  const env=(input.envelope||[]).filter(s=>n(s.area)>0);
  const windows=(input.windows||[]).filter(w=>n(w.area)>0);
  const t=monthlyTransmission({...input,area,envelope:env,windows});
  const v=monthlyVentilation({...input,area});
  const usefulMonthly=t.losses.map((x,i)=>Math.max(0,x+v.losses[i]-t.gains[i]));
  const usefulHeat=sum(usefulMonthly);
  const heat=heatingEnergy(input,usefulHeat);
  const water=dhw(input);
  const lighting=area*clamp(n(input.lightingKwhM2,8),0,50);
  const aux=area*clamp(n(input.auxKwhM2,4),0,30);
  const coolUseful=area*clamp(n(input.coolingKwhM2,0),0,50);
  const coolingElec=input.cooling?.type==='heatpump' ? coolUseful/clamp(n(input.cooling.cop,3),1,8) : coolUseful;
  const pv=Math.max(0,n(input.pv.kWp,0))*900*clamp(n(input.pv.yieldFactor,1),0,1.5);
  const elecGross=heat.elec+water.inputEnergy+lighting+aux+coolingElec;
  const pvUsed=Math.min(elecGross,pv*clamp(n(input.pv.selfUse,0.35),0,1));
  const gridElec=Math.max(0,elecGross-pvUsed);
  const exported=Math.max(0,pv-pvUsed);
  const primaryFossil=(gridElec*PRIMARY_ELEC)+(heat.fuel*GAS_PRIMARY);
  const ep2=primaryFossil/area;
  const label=labelFor(ep2);
  const renewable=(heat.renewableHeat + pv + Math.max(0,pv-exported))/Math.max(1,usefulHeat+water.useful+coolUseful);
  return {
    area, usefulHeat, usefulHeatPerM2:usefulHeat/area,
    transmission:sum(t.losses), ventilation:sum(v.losses), gains:sum(t.gains), ventilationFlow:v.flow,
    heating:heat, dhw:water, lighting, auxiliary:aux, cooling:{useful:coolUseful,electricity:coolingElec},
    electricity:{gross:elecGross,pv,pvUsed,grid:gridElec,export:exported}, primaryFossil, ep2, label,
    wwsEnergyPoints:wwsEnergyPoints(label), wwsTotal:n(input.wwsBasePoints,0)+wwsEnergyPoints(label), renewableShare:clamp(renewable,0,1)
  };
}

export function validateInput(input){
  const errors=[];
  if(n(input.area)<=0) errors.push('Usable floor area must be greater than 0 m².');
  for(const s of input.envelope||[]) if(n(s.area)<0||n(s.u)<=0) errors.push(`Invalid envelope row: ${s.name||'unnamed'}.`);
  for(const w of input.windows||[]) if(n(w.area)<0||n(w.u)<=0) errors.push(`Invalid window row: ${w.name||'unnamed'}.`);
  return errors;
}
