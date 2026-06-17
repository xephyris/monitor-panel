use std::{fs::read_to_string, process::Command, time::Duration};

use bytesize::ByteSize;

use crate::pages::Page;

pub struct UsagePage {
    cpu: Option<f32>,
    gpu: Option<f32>,
    memory_used: Option<String>,
    memory_total: Option<String>,
    disk_used: Option<String>,
    disk_total: Option<String>,
    fan: Option<u32>, 
}

impl UsagePage {
    pub fn new() -> Self {
        UsagePage {
            cpu: None,
            gpu: None,
            memory_used: None,
            memory_total: None,
            disk_used: None,
            disk_total: None,
            fan: None,
        }
    }

    fn cpu_monitor() -> Option<f32> {
        if let Ok(top) = Command::new("top").arg("-bn1").output() {
            let stdout = String::from_utf8_lossy(&top.stdout);
            if let Some(cpu) = stdout.lines().find(|line| line.contains("Cpu(s)")) {
                let percents: Vec<&str> = cpu.split_whitespace().collect();
                // dbg!(&percents);
                let user_per = percents[1].parse::<f64>().expect("Parse Failed");
                let sys_per = percents[3].parse::<f64>().expect("Parse Failed");
                let ni_per = percents[5].parse::<f64>().expect("Parse Failed");
                
                return Some((user_per + sys_per + ni_per) as f32);
            }
            return None;
        }
        None
    }

    fn memory_monitor() -> (Option<String>, Option<String>) {
        let file = read_to_string("/proc/meminfo").unwrap_or("".to_string());
        let mut lines = file.lines();
        let memory_total = lines.next().unwrap_or("");
        let _memory_free = lines.next();
        let memory_available = lines.next().unwrap_or("");
        let str_split:Vec<&str> = memory_total.split_whitespace().collect();
        let memory_total = str_split[1].parse::<u64>().unwrap_or(0) * 1000;
        let total_size = ByteSize::b(memory_total as u64);

        let str_split:Vec<&str> = memory_available.split_whitespace().collect();
        let memory_available = str_split[1].parse::<u64>().unwrap_or(0) * 1000;
        let memory_used = memory_total - memory_available;
        let used_size = ByteSize::b(memory_used as u64);

        // println!("file{} \n {} / {}", file, memory_total, memory_used);
        (Some(used_size.display().si().to_string()), Some(total_size.display().si().to_string()))
    }

    fn disk_monitor() -> (Option<String>, Option<String>) {
        if let Ok(output) = Command::new("df").args(["-k", "/"]).output() {

            let stdout = String::from_utf8_lossy(&output.stdout);
            
            let data_line = stdout.lines().nth(1).unwrap_or(" ");
            let mut parts = data_line.split_whitespace();

            let _filesystem = parts.next().unwrap_or(" ");
            let total_kb: u64 = parts.next().unwrap_or(" ").parse().unwrap_or(0) * 1000;
            let total_mem = ByteSize::b(total_kb);
            let used_kb: u64 = parts.next().unwrap_or(" ").parse().unwrap_or(0) * 1000;
            let used_mem = ByteSize::b(used_kb);
            let used: String = used_mem.display().si().to_string().chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
            (Some(used), Some(total_mem.display().si().to_string()))
        } else {
            (None, None)
        }
    }

    fn fan_monitor() -> Option<u32> {
        let file = read_to_string("/sys/class/hwmon/hwmon9/pwm1").unwrap_or(String::new());
        // println!("File {}", file);
        let percent = file.trim().parse::<u32>().unwrap_or(256);
        if percent == 256 {
            None
        } else {
            Some(percent)
        }

    }
}

#[repr(u32)]
enum UsageIcons {
    CPUIcon = 0xE0A9,
    MemoryIcon = 0xE445,
    DiskIcon = 0xE0ED,
    GPUIcon = 0xE66A,
    FanIcon = 0xE379,
    WindIcon = 0xE1B0,
}

fn verify_value_f32(val: Option<f32>) -> String {
    if let Some(val) = val {
        val.to_string()
    } else {
        "-".to_string()
    }
}

fn verify_string(val: &Option<String>) -> String {
    if let Some(val) = val {
        val.to_string()
    } else {
        "-".to_string()
    }
}

fn verify_value(val: Option<u32>) -> String {
    if let Some(val) = val {
        val.to_string()
    } else {
        "-".to_string()
    }
}

impl Page for UsagePage {
    fn display(&self) -> String {
        format!("{} {} % \n{} {} / {}\n{} {} / {}\n{} {} %\n{} {} %", 
            char::from_u32(UsageIcons::CPUIcon as u32).unwrap_or('-'), 
            verify_value_f32(self.cpu), 
            char::from_u32(UsageIcons::MemoryIcon as u32).unwrap_or('-'), 
            verify_string(&self.memory_used), verify_string(&self.memory_total),
            char::from_u32(UsageIcons::DiskIcon as u32).unwrap_or('-'), 
            verify_string(&self.disk_used), verify_string(&self.disk_total),
            char::from_u32(UsageIcons::GPUIcon as u32).unwrap_or('-'), 
            '-',
            char::from_u32(UsageIcons::FanIcon as u32).unwrap_or('-'), 
            verify_value(self.fan),
        )
    }

    fn refresh_rate(&self) -> Duration {
        Duration::from_millis(1000)
    }

    fn update(&mut self) {
        self.cpu = UsagePage::cpu_monitor();
        let (mem_used, mem_total) = UsagePage::memory_monitor();
        self.memory_used = mem_used;
        self.memory_total = mem_total;
        let (disk_used, disk_total) = UsagePage::disk_monitor();
        self.disk_used = disk_used;
        self.disk_total = disk_total;
        self.fan = UsagePage::fan_monitor();
    }
}