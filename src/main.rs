use clap::Parser;
use notify_rust::Notification;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::enums::device::UsedGpuMemory;
use nvml_wrapper::struct_wrappers::device::ProcessInfo;
use nvml_wrapper::Nvml;
use sysinfo::{Pid, PidExt, ProcessExt, System, SystemExt};

fn send_notification(name: &str, body: &str, icon: &str) {
    Notification::new()
        .summary(name)
        .body(body)
        .icon(icon)
        .show()
        .unwrap();
}

fn get_gpu_usage() -> GpuInfo {
    let nvml = Nvml::init().unwrap();
    let device = nvml.device_by_index(0).unwrap();

    let name = device.name().unwrap();
    let total_utilization = device.utilization_rates().unwrap();
    let memory_usage = device.memory_info().unwrap();
    let temperature = device.temperature(TemperatureSensor::Gpu).unwrap();
    let graphics_processes: Vec<ProcessInfo> = device.running_graphics_processes_v2().unwrap();

    GpuInfo {
        name,
        total_utilization: format!("{}%", total_utilization.gpu),
        memory_usage: (memory_usage.used >> 20, memory_usage.total >> 20),
        temperature,
        graphics_processes,
    }
}

fn get_target_process_info(gpu_info: GpuInfo, target_process: &str) -> SingleProcessInfo {
    gpu_info
        .graphics_processes
        .iter()
        .filter(|process| {
            let sys = System::new_all();
            let process_name = get_process_name(&sys, process.pid);
            process_name == target_process
        })
        .map(|process| SingleProcessInfo {
            name: get_process_name(&System::new_all(), process.pid),
            memory_usage: match process.used_gpu_memory {
                UsedGpuMemory::Used(used) => used >> 20,
                _ => 0,
            },
        })
        .collect::<Vec<SingleProcessInfo>>()
        .pop()
        .unwrap()
}

fn get_process_name(sys: &sysinfo::System, pid: u32) -> String {
    if let Some(process) = sys.process(Pid::from_u32(pid)) {
        return process.name().to_string();
    }
    String::from("")
}
#[derive(Debug, Clone)]
struct GpuInfo {
    name: String,
    total_utilization: String,
    memory_usage: (u64, u64),
    temperature: u32,
    graphics_processes: Vec<ProcessInfo>,
}

#[derive(Debug)]
struct SingleProcessInfo {
    name: String,
    memory_usage: u64,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    name: String,
}

fn main() {
    let args = Args::parse();
    let target_process = args.name;

    let gpu_info = get_gpu_usage();
    let target_process_info = get_target_process_info(gpu_info.clone(), target_process.as_str());

    println!(
        "Name: {:#?}\nTotal utilization: {:#?}\nMemory usage: {:#?}/{:#?} MB\nTemperature: {:#?}°C\n{} memory usage: {:#?} MB",
        gpu_info.name,
        gpu_info.total_utilization,
        gpu_info.memory_usage.0,
        gpu_info.memory_usage.1,
        gpu_info.temperature,
        target_process_info.name,
        target_process_info.memory_usage
    );
    send_notification(
        "GPU Usage",
        format!(
            "{} Is Utilizing {} MB of Memory",
            target_process_info.name, target_process_info.memory_usage
        )
        .as_str(),
        "dialog-information",
    );
}
