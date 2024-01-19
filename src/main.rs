use chrono::{DateTime, Utc};
use clap::{Args, Parser};
use notify_rust::Notification;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::enums::device::UsedGpuMemory;
use nvml_wrapper::struct_wrappers::device::ProcessInfo;
use nvml_wrapper::Nvml;
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
};
use sysinfo::{Pid, System};

fn capitalize(input: String) -> String {
    let mut chars = input.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first
            .to_uppercase()
            .chain(chars.map(|c| c.to_ascii_lowercase()))
            .collect(),
    }
}

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

fn print_info(gpu_info: GpuInfo, target_process_info: SingleProcessInfo) {
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
}

fn init_log(path: &str) -> File {
    if Path::new(path).exists() {
        OpenOptions::new().append(true).open(path).unwrap()
    } else {
        File::create(path).unwrap()
    }
}

fn log_info(
    gpu_info: GpuInfo,
    log_path: &str,
    target_process_info: Option<SingleProcessInfo>,
    delimiter: &str,
) {
    let mut file = init_log(log_path);

    let mut log = String::new();
    let now: DateTime<Utc> = Utc::now();

    log.push_str(format!("{} | ", now.format("%a %b %e %T %Y")).as_str());
    log.push_str(format!("{}{}", gpu_info.name, delimiter).as_str());
    log.push_str(
        format!(
            "Total utilization: {}{}",
            gpu_info.total_utilization, delimiter
        )
        .as_str(),
    );
    log.push_str(
        format!(
            "Memory usage: {}/{} MB{}",
            gpu_info.memory_usage.0, gpu_info.memory_usage.1, delimiter
        )
        .as_str(),
    );
    log.push_str(format!("Temperature: {}°C{}", gpu_info.temperature, delimiter).as_str());

    match target_process_info {
        Some(target_process_info) => {
            log.push_str(
                format!(
                    "{} memory usage: {} MB",
                    target_process_info.name, target_process_info.memory_usage
                )
                .as_str(),
            );
        }
        None => {
            for process in gpu_info.graphics_processes {
                let process_name = capitalize(get_process_name(&System::new_all(), process.pid));
                log.push_str(
                    format!(
                        "{} memory usage: {} MB",
                        process_name,
                        match process.used_gpu_memory {
                            UsedGpuMemory::Used(used) => used >> 20,
                            _ => 0,
                        }
                    )
                    .as_str(),
                );
            }
        }
    }

    log.push('\n');

    file.write_all(log.as_bytes()).unwrap();
}

#[derive(Debug, Clone)]
struct GpuInfo {
    name: String,
    total_utilization: String,
    memory_usage: (u64, u64),
    temperature: u32,
    graphics_processes: Vec<ProcessInfo>,
}

#[derive(Debug, Clone)]
struct SingleProcessInfo {
    name: String,
    memory_usage: u64,
}

/// Simple program to get the GPU usage of a process
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[command(flatten)]
    name_or_loging: NameOrLoging,

    /// Print info about the GPU and the process
    #[arg(short, long, requires = "name")]
    print_info: bool,

    /// Disable the notification
    #[arg(short, long, requires = "name")]
    disable_notification: bool,

    /// Path to the log file
    #[arg(
        short = 'L',
        long,
        default_value = "/tmp/gpu-usage.log",
        requires = "loging"
    )]
    log_path: String,

    /// Log delimiter
    #[arg(short, long, default_value = ", ", requires = "loging")]
    delimiter: String,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = true)]
struct NameOrLoging {
    /// Name of the a process
    #[arg(short, long)]
    name: Option<String>,

    /// Log the GPU usage
    #[arg(short, long)]
    loging: bool,
}

fn main() {
    let args: Arguments = Arguments::parse();

    let gpu_info: GpuInfo = get_gpu_usage();

    if args.name_or_loging.name.is_some() {
        let target_process_info: SingleProcessInfo =
            get_target_process_info(gpu_info.clone(), args.name_or_loging.name.unwrap().as_str());

        if !args.disable_notification {
            send_notification(
                "GPU Usage",
                format!(
                    "{} is utilizing {} MB of memory",
                    capitalize(target_process_info.name.clone()),
                    target_process_info.memory_usage
                )
                .as_str(),
                "dialog-information",
            );
        }
        if args.print_info {
            print_info(gpu_info.clone(), target_process_info.clone());
        }
        if args.name_or_loging.loging {
            log_info(
                gpu_info,
                &args.log_path,
                Some(target_process_info),
                args.delimiter.as_str(),
            );
        }
    } else if args.name_or_loging.loging {
        log_info(gpu_info, &args.log_path, None, args.delimiter.as_str());
    } else {
        println!("Please provide a process name or enable loging")
    }
}
