use std::{process::Command, thread, time::Duration};
mod sources;
use sources::AppleSMCClient;

type WithError<T> = Result<T, Box<dyn std::error::Error>>;
pub struct TempMetrics {
    pub cpu_temp_avg: f32,
    pub gpu_temp_avg: f32,
}
//situationship: sketchybar graph accepts values in the range [0,1], temperature of course does not fit, so ti should be normalized
//The ideal aproach would be dinamically calculate max and min registered values
//but this is simpler. I have selected as the min 20 degress becuase i never spotted gpu/cpu temp lower than this at any load
const CPU_RANGE: (f32, f32) = (20.0, 120.0);
const GPU_RANGE: (f32, f32) = (20.0, 120.0);

//basic normalization...
fn normalise(value: f32, range: (f32, f32)) -> f32 {
    let (min, max) = range;
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}
//temperature for both cpi and gpu is calculated as average of all related sensors due to the fact that there is not
//IO report dictionary with human names of sensor names (four letter names)
//some sensors can occasionally show 0, so these case should be handled for mean calculation
fn zero_div<T: core::ops::Div<Output = T> + Default + PartialEq>(a: T, b: T) -> T {
    let zero: T = Default::default();
    return if b == zero { zero } else { a / b };
}

//initializing client defined in sources.rs
fn init_temperature_smc_client() -> WithError<(AppleSMCClient, Vec<String>, Vec<String>)> {
    let mut smc = AppleSMCClient::new()?;

    let mut cpu_sensors = Vec::new();
    let mut gpu_sensors = Vec::new();

    let names = smc.read_all_sensors().unwrap_or(vec![]);
    //println!("{:?} (len = {})", names, names.len());
    //39 cpu sensors and 18 of gpu sensors on my machine
    for name in &names {
        let key = match smc.read_sensor_code_name_info(&name) {
            Ok(key) => key,
            Err(_) => continue,
        };

        if key.data_size != 4 || key.data_type != 1718383648 {
            continue;
        }

        let _ = match smc.read_val(&name) {
            Ok(val) => val,
            Err(_) => continue,
        };

        // Unfortunately, it is not known which keys are responsible for what.
        // Basically in the code that can be found publicly "Tp" is used for CPU and "Tg" for GPU.

        match name {
            name if name.starts_with("Tp") => cpu_sensors.push(name.clone()),
            name if name.starts_with("Tg") => gpu_sensors.push(name.clone()),
            _ => (),
        }
    }

    //println!("{} {}", cpu_sensors.len(), gpu_sensors.len());
    //   println!(
    //     "{:?} (len = {})  {:?} (len = {})",
    //     cpu_sensors,
    //     cpu_sensors.len(),
    //     gpu_sensors,
    //     gpu_sensors.len()
    // );
    Ok((smc, cpu_sensors, gpu_sensors))
}

//reads actual temperature values from each cpu and gpu sensor, returns averages
fn get_temp_smc_averages(
    smc: &mut AppleSMCClient,
    cpu_keys: &[String],
    gpu_keys: &[String],
) -> WithError<TempMetrics> {
    // collect all cpu metrics into a vector
    let mut cpu_metrics = Vec::with_capacity(cpu_keys.len());
    for sensor in cpu_keys {
        let val = smc.read_val(sensor)?;
        let val = f32::from_le_bytes(val.sensor_data[0..4].try_into().unwrap());
        cpu_metrics.push(val);
    }
    //same for gpu
    let mut gpu_metrics = Vec::with_capacity(gpu_keys.len());
    for sensor in gpu_keys {
        let val = smc.read_val(sensor)?;
        let val = f32::from_le_bytes(val.sensor_data[0..4].try_into().unwrap());
        gpu_metrics.push(val);
    }
    //some trials with other sensors
    //let m_tpl_val = smc.read_val("mTPL")?;
    //let m_tpl_val_f32 = f32::from_le_bytes(m_tpl_val.sensor_data[0..4].try_into().unwrap());
    //println!("{:?}", m_tpl_val_f32);
    //let info = smc.read_key_info("mTPL")?;
    //println!(
    //    "mTPL info: size = {}, type = {:#x}",
    //    info.data_size, info.data_type
    //);
    //let raw_bytes = &&m_tpl_val.sensor_data[0..4];
    //let val_i32 = i32::from_be_bytes(raw_bytes.try_into().unwrap());
    //println!("mTPL (si32) = {}", val_i32);

    //println!("{:?}", cpu_metrics);
    //println!("{:?}", gpu_metrics);
    let cpu_temp_avg = zero_div(cpu_metrics.iter().sum::<f32>(), cpu_metrics.len() as f32);
    let gpu_temp_avg = zero_div(gpu_metrics.iter().sum::<f32>(), gpu_metrics.len() as f32);

    Ok(TempMetrics {
        cpu_temp_avg,
        gpu_temp_avg,
    })
}
//}

fn main() -> WithError<()> {
    let (mut smc, cpu_keys, gpu_keys) = init_temperature_smc_client()?;

    //loop goes and goes and pushes the sata to sketchybar.....
    //this rust binary is easily detected in ps and has pid and can be killed at any time as a process
    loop {
        let metrics = get_temp_smc_averages(&mut smc, &cpu_keys, &gpu_keys)?;
        let cpu_graph = normalise(metrics.cpu_temp_avg, CPU_RANGE);
        let gpu_graph = normalise(metrics.gpu_temp_avg, GPU_RANGE);

        //print data to see output in terminal as well, good for dev purposes only, ofc.
        // println!(
        //     "CPU avg: {:.2} °C\tGPU avg: {:.2} °C",
        //     metrics.cpu_temp_avg, metrics.gpu_temp_avg
        // );
        // push normalized values to sketchybar, both lines on the same graph...
        Command::new("sketchybar")
            .args(&[
                "--push",
                "temp.cpu",
                &format!("{:.2}", cpu_graph),
                "--push",
                "temp.gpu",
                &format!("{:.2}", gpu_graph),
            ])
            .status()?;
        //data are normalized so not very clear what are the values from graph,
        //so i add labels as well and this code will update their values as well
        Command::new("sketchybar")
            .args(&[
                "--set",
                "temp.cpu",
                &format!("label={:.0}°", metrics.cpu_temp_avg),
                "--set",
                "temp.gpu",
                &format!("label={:.0}°", metrics.gpu_temp_avg),
            ])
            .status()?;

        // sleep can be changed, will be equal to update_freq parameter of other sketchybar scripts
        thread::sleep(Duration::from_secs(1));
    }
}
