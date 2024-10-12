mod prompt;
mod theme;
mod select;

use dirs::home_dir;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::process::Command;
use std::time::Instant;

use crate::select::select;

#[derive(Debug)]
struct SSHConfigEntry {
    alias: String,
    hostname: String,
    username: Option<String>,
    identity_file: Option<String>,
    latency: Option<u128>, // Latency in milliseconds
}

// Parse ~/.ssh/config and extract host entries
fn parse_ssh_config() -> io::Result<HashMap<String, SSHConfigEntry>> {
    let home = home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
    })?;
    let path = home.join(".ssh/config");
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    let mut hosts = HashMap::new();
    let mut current_alias = None;

    for line in reader.lines() {
        let line = line?.trim().to_string();

        if line.starts_with("Host ") {
            if let Some(alias) = line.strip_prefix("Host ") {
                current_alias = Some(alias.to_string());
                hosts.insert(
                    alias.to_string(),
                    SSHConfigEntry {
                        alias: alias.to_string(),
                        hostname: alias.to_string(),
                        username: None,
                        identity_file: None,
                        latency: None,
                    },
                );
            }
        } else if line.starts_with("  Hostname ") {
            if let Some(hostname) = line.strip_prefix("Hostname ") {
                if let Some(current_alias) = &current_alias {
                    if let Some(entry) = hosts.get_mut(current_alias) {
                        entry.hostname = hostname.to_string();
                    }
                }
            }
        } else if line.starts_with("  User ") {
            if let Some(user) = line.strip_prefix("User ") {
                if let Some(current_alias) = &current_alias {
                    if let Some(entry) = hosts.get_mut(current_alias) {
                        entry.username = Some(user.to_string());
                    }
                }
            }
        } else if line.starts_with("  IdentityFile ") {
            if let Some(identity_file) = line.strip_prefix("IdentityFile ") {
                if let Some(current_alias) = &current_alias {
                    if let Some(entry) = hosts.get_mut(current_alias) {
                        entry.identity_file = Some(identity_file.to_string());
                    }
                }
            }
        }
    }

    Ok(hosts)
}

// Function to check SSH reachability and return connection latency
fn ssh_check_latency(entry: &SSHConfigEntry) -> Option<u128> {
    let user_host = match &entry.username {
        Some(username) => format!("{}@{}", username, entry.hostname),
        None => entry.hostname.clone(),
    };

    let mut ssh_command = Command::new("ssh");
    ssh_command
        .arg("-q")
        .arg("-T")
        .arg("-o").arg("BatchMode=yes")
        .arg("-o").arg("ConnectTimeout=3")
        .arg("-o").arg("PreferredAuthentications=publickey")
        .arg("-o").arg("NumberOfPasswordPrompts=0")
        .arg(user_host)
        .arg("true");

    if let Some(identity_file) = &entry.identity_file {
        ssh_command.arg("-i").arg(identity_file);
    }

    let start = Instant::now();
    let status = ssh_command.status().ok();
    let duration = start.elapsed().as_millis();

    if let Some(status) = status {
        if status.success() {
            return Some(duration);
        }
    }

    None
}

// Add latency information to each host
fn add_latency_to_hosts(hosts: &mut HashMap<String, SSHConfigEntry>) {
    for entry in hosts.values_mut() {
        entry.latency = ssh_check_latency(entry);
    }
}

// CLI for selecting a host, showing empty latency by default
fn select_host_cli(hosts: &mut HashMap<String, SSHConfigEntry>) -> Option<String> {
    loop {
        let mut sorted_hosts: Vec<&String> = hosts.keys().collect();
        sorted_hosts.sort();
        let mut selector = select("✨ Select an option ✨");

        // Add option to ping all hosts
        selector = selector.item("ping", "📡 Get latencies", "");

        // Add individual hosts to the selection menu
        for (index, alias) in sorted_hosts.iter().enumerate() {
            let entry = hosts.get(*alias).unwrap();
            let latency_info = match entry.latency {
                Some(latency) => format!("{} ms", latency),
                None => String::from(""), // Default to an empty string
            };
            let label = format!("({}) {} - {}", index, *alias, latency_info);
            selector = selector.item(*alias, label, "");
        }

        match selector.interact() {
            Ok(selection) => {
                if selection == "ping" {
                    println!("📡 Pinging all hosts...");
                    add_latency_to_hosts(hosts); // Update latencies
                } else {
                    return Some(selection.to_string().clone());
                }
            }
            Err(_) => {
                println!("No valid selection. Exiting.");
                return None;
            }
        }
    }
}

// SSH into the selected host, using the identity file if provided
fn ssh_into_host(entry: &SSHConfigEntry) -> Result<(), Box<dyn std::error::Error>> {
    let user_host = match &entry.username {
        Some(username) => format!("{}@{}", username, entry.hostname),
        None => entry.hostname.clone(),
    };

    let mut ssh_command = Command::new("ssh");
    ssh_command.arg(user_host);

    if let Some(identity_file) = &entry.identity_file {
        ssh_command.arg("-i").arg(identity_file);
    }

    ssh_command.status()?;
    Ok(())
}

// Main function
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut hosts = parse_ssh_config()?;

    if hosts.is_empty() {
        eprintln!("No hosts found in ~/.ssh/config");
        return Ok(());
    }

    // Allow user to select an option and SSH into a host if desired
    if let Some(selected_host) = select_host_cli(&mut hosts) {
        let selected_entry = hosts.get(&selected_host).unwrap();
        ssh_into_host(&selected_entry)?;
    } else {
        println!("No host selected.");
    }

    Ok(())
}
