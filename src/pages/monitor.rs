use std::process::Command;

fn cpu_monitor() -> String {
    if let Ok(top) = Command::new("top").arg("-bn1").output() {
        let stdout = String::from_utf8_lossy(&top.stdout);
        if let Some(cpu) = stdout.lines().find(|line| line.contains("Cpu(s)")) {
            let percents: Vec<&str> = cpu.split_whitespace().collect();
            dbg!(&percents);
            let user_per = percents[1].parse::<f64>().expect("Parse Failed");
            let sys_per = percents[3].parse::<f64>().expect("Parse Failed");
            let ni_per = percents[5].parse::<f64>().expect("Parse Failed");
            
            return (user_per + sys_per + ni_per).to_string();
        }
        return "Error".to_string();
    }
    "Error".to_string()
}

pub fn system_monitor() -> String {
    cpu_monitor()
}