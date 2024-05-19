mod prompt;
mod theme;
mod select;

use dirs::home_dir;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::process::Command;

use crate::select::select;

#[derive(Debug)]
struct SSHConfigEntry {
    hostname: String,
    username: Option<String>,
}

fn parse_ssh_config() -> io::Result<HashMap<String, SSHConfigEntry>> {
    let home = home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;
    let path = home.join(".ssh/config");
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    let mut hosts = HashMap::new();
    let mut current_host = None;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.starts_with("Host ") {
            if let Some(host) = line.strip_prefix("Host ") {
                current_host = Some(host.to_string());
                hosts.insert(host.to_string(), SSHConfigEntry { hostname: host.to_string(), username: None });
            }
        } else if line.starts_with("  User ") {
            if let Some(user) = line.strip_prefix("User ") {
                if let Some(current_host) = &current_host {
                    if let Some(entry) = hosts.get_mut(current_host) {
                        entry.username = Some(user.to_string());
                    }
                }
            }
        }
    }

    Ok(hosts)
}

fn select_host_cli(hosts: &HashMap<String, SSHConfigEntry>) -> Option<String> {
    let mut sorted_hosts: Vec<&String> = hosts.keys().collect();
    sorted_hosts.sort();
    let mut selector = select("✨ Select a host to SSH into ✨");

    for (index, hostname) in sorted_hosts.iter().enumerate() {
        let entry = hosts.get(*hostname).unwrap();
        let description = match &entry.username {
            Some(username) => format!("User: {}", username),
            None => String::from(""),
        };
        let label = format!("({}) {}", index, *hostname);
        selector = selector.item(*hostname, label, &description);
    }

    match selector.interact() {
        Ok(selection) => Some(selection.clone()),
        Err(_) => None,
    }
}
    
fn ssh_into_host(entry: &SSHConfigEntry) -> Result<(), Box<dyn std::error::Error>> {
    let user_host = match &entry.username {
        Some(username) => format!("{}@{}", username, entry.hostname),
        None => format!("{}", entry.hostname),
    };

    Command::new("ssh")
        .arg(user_host)
        .status()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let hosts = parse_ssh_config()?;
    println!("No host selected");

    if hosts.is_empty() {
        eprintln!("No hosts found in ~/.ssh/config");
        return Ok(());
    }

    if let Some(selected_host) = select_host_cli(&hosts) {
        let selected_entry = hosts.get(&selected_host).unwrap();
        ssh_into_host(&selected_entry)?;
    } else {
        println!("No host selected");
    }

    Ok(())
}

