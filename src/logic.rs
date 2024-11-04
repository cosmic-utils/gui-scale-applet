use std::{
    collections::VecDeque, io::{Error, Read}, process::{Command, Output, Stdio}, thread, time::Duration
};

use regex::RegexBuilder;

/// Get the IPv4 address assigned to this computer.
pub fn get_tailscale_ip() -> String {
    let ip_cmd = Command::new("tailscale")
        .args(["ip", "-4"])
        .output()
        .unwrap();

    match String::from_utf8(ip_cmd.stdout) {
        Ok(ip) => ip,
        Err(e) => format!("Could get tailscale IPv4 address!\n{e}"),
    }
}

/// Get Tailscale's connection status
pub fn get_tailscale_con_status() -> bool {
    let con_cmd = Command::new("tailscale")
        .args(["debug", "prefs"])
        .stdout(Stdio::piped())
        .spawn();

    let grep_cmd = Command::new("grep")
        .arg("WantRunning")
        .stdin(con_cmd.unwrap().stdout.unwrap())
        .output();

    let con_status = String::from_utf8(grep_cmd.unwrap().stdout).unwrap();

    con_status.contains("true")
}

pub fn get_tailscale_devices() -> Vec<String> {
    let ts_status_cmd = Command::new("tailscale")
        .arg("status")
        .output();

    let out = match String::from_utf8(ts_status_cmd.unwrap().stdout) {
        Ok(s) => s,
        Err(e) => format!("Error getting the status output: {e}"),
    };
    // Create a regular expression that finds all of the lines with an ipv4 address
    let reg = RegexBuilder::new(r#"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}"#)
        .build()
        .unwrap();

    let mut status_output: VecDeque<String> = out.lines()
    // Filter out the lines that don't match the ipv4 pattern.
    .filter(
        |line| reg.is_match(line)
    )
    // Map only the device names as elements of the VecDeque
    .map(|line| {
        line.split_whitespace().nth(1).expect("Device name not found").to_string()
    })
    .collect();

    // Pop this system's device name out of the VecDeque
    status_output.pop_front();
    // Add Select as the first element
    status_output.push_front("Select".to_string());

    // Return as a
    status_output.to_owned().into()
}

/// Get the current status of the SSH enablement
pub fn get_tailscale_ssh_status() -> bool {
    let ssh_cmd = Command::new("tailscale")
    .args(["debug", "prefs"])
    .stdout(Stdio::piped())
    .spawn();

    let grep_cmd = Command::new("grep")
        .arg("RunSSH")
        .stdin(ssh_cmd.unwrap().stdout.unwrap())
        .output();

    let ssh_status = String::from_utf8(grep_cmd.unwrap().stdout).unwrap();

    ssh_status.contains("true")
}

/// Get the current status of the accept-routes enablement
pub fn get_tailscale_routes_status() -> bool {
    let ssh_cmd = Command::new("tailscale")
    .args(["debug", "prefs"])
    .stdout(Stdio::piped())
    .spawn();

    let grep_cmd = Command::new("grep")
        .arg("RouteAll")
        .stdin(ssh_cmd.unwrap().stdout.unwrap())
        .output();

    let ssh_status = String::from_utf8(grep_cmd.unwrap().stdout).unwrap();

    ssh_status.contains("true")
}

/// Get available devices
pub fn _get_available_devices() -> String {
    let cmd = Command::new("tailscale")
        .args(["status", "--active"])
        .output();


    String::from_utf8(cmd.unwrap().stdout).unwrap()
}

/// Set the Tailscale connection up/down
pub fn tailscale_int_up(up_down: bool) -> bool {
    let mut ret = false;
    if up_down {
        let _ = Command::new("tailscale")
                .arg("up")
                .output();

        ret = true;
    } else {
        let _ = Command::new("tailscale")
            .arg("down")
            .output();
    }

    ret
}

/// Send files through Tail Drop
/// It's async so that it can be ran in another thread making it
/// non-blocking for the UI.
pub async fn tailscale_send(file_paths: Vec<Option<String>>, target: &str) -> Option<String> {
    // A Vec<Option<String>> that holds any error messages that may come back.
    let mut status = Vec::<Option<String>>::new();

    // Loop through the file paths
    for path in file_paths.iter() {
        // Set a error string variable to be added to the status
        let mut err_str = String::new();

        // Match on the path so Tail Drop can use it to send the file
        match path {
            // If there is path value
            Some(p) => {
                // Send the file
                let cmd = Command::new("tailscale")
                .args(["file", "cp", p, &format!("{target}:")])
                .spawn();

                // Check for errors from the tailscale command
                if let Some(mut err) = cmd.unwrap().stderr {
                    // Update the err_str variable with the error and continue
                    // to the next file.
                    let _ = err.read_to_string(&mut err_str);
                    continue;
                };                
            }
            // If the path was no good, send an error message back to the UI.
            None => return Some(String::from("Something went wrong sending the file!\nPossible bad file path!")),
        };

        // If there were an error, add it to the status Vec
        if !err_str.is_empty() { 
            status.push(Some(err_str));
        
        }
    }

    // If we got any errors, let the user know about them.
    if !status.is_empty() {
        return Some("One or more files were not sent successfully!".to_string())
    }

    None
}

/// Recieve files through Tail Drop
/// It's async so that it can be ran in another thread making it
/// non-blocking for the UI.
pub async fn tailscale_recieve() -> String {
    // Get the username of the current user.
    let whoami_cmd = Command::new("whoami")
        .output()
        .unwrap();

    // Set the username to a variable.
    let username = String::from_utf8(whoami_cmd.stdout).unwrap();

    // Create a path to the user's Downloads directory.
    let download_path = &format!("/home/{}/Downloads/", username.trim());

    // Run the tail drop recieve command, placing the file(s) in the user's Downloads directory.
    let rx_cmd = Command::new("tailscale")
        .args(["file", "get", download_path])
        .output();

    // Check to see if there were any errors during the recieve process.
    let rx_stderr = rx_cmd.unwrap().stderr.clone();

    // Either send a success or error message back to the UI.
    if rx_stderr.is_empty() {
        "Recieved file(s) in Downloads!".to_string()
    } else {
        String::from_utf8(rx_stderr).unwrap()
    }
}

pub async fn clear_status(wait_time: u64) -> Option<String> {
    thread::sleep(Duration::from_secs(wait_time));

    None
}

/// Toggle SSH on/off
pub fn set_ssh(ssh: bool) -> bool {
    let cmd: Result<Output, Error> = if ssh {
        Command::new("tailscale")
            .args(["set", "--ssh"])
            .output()
    } else {
        Command::new("tailscale")
            .args(["set", "--ssh=false"])
            .output()
    };

    match cmd {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Error occurred: {e}");
            false
        }
    }
}

/// Toggle accept-routes on/off
pub fn set_routes(accept_routes: bool) -> bool {
    let cmd: Result<Output, Error> = if accept_routes {
        Command::new("tailscale")
            .args(["set", "--accept-routes"])
            .output()
    } else {
        Command::new("tailscale")
            .args(["set", "--accept-routes=false"])
            .output()
    };

    match cmd {
        Ok(_) => true,
        Err(e) => {
            eprintln!("Error occurred: {e}");
            false
        }
    }
}

// Exit Node Section

/// Make current host an exit node
pub fn enable_exit_node(is_exit_node: bool) {
    let _advertise_cmd = Command::new("tailscale")
        .args(["set", &format!("--advertise-exit-node={is_exit_node}")])
        .spawn()
        .unwrap();

    let _ = tailscale_int_up(true);
}

/// Get the status of whether or not the host is an exit node
pub fn get_is_exit_node() -> bool {
    let is_exit_node_cmd = Command::new("tailscale")
    .args(["debug", "prefs"])
    .stdout(Stdio::piped())
    .spawn();

    let grep_cmd = Command::new("grep")
        .args(["-i", "advertiseroutes"])
        .stdin(is_exit_node_cmd.unwrap().stdout.unwrap())
        .output();

    let ssh_status = String::from_utf8(grep_cmd.unwrap().stdout).unwrap();

    if ssh_status.contains("null") {
        return false;
    }

    true
}

/// Add/remove exit node's access to the host's local LAN
pub fn exit_node_allow_lan_access(is_allowed: bool) -> String {
    let allow_lan_access = if is_allowed { "true" } else { "false" };

    let allow_lan_cmd = Command::new("tailscale")
        .args(["set", &format!("--exit-node-allow-lan-access={allow_lan_access}")])
        .spawn();

    match allow_lan_cmd {
        Ok(_) => String::from("Exit node access to LAN allowed!"),
        Err(e) => format!("Something went wrong: {e}"),
    }
} 

/// Get available exit nodes
pub fn get_avail_exit_nodes() -> Vec<String> {
    // Run the tailscale exit-node list command
    let exit_node_list_cmd = Command::new("tailscale")
        .args(["exit-node", "list"])
        .output();

    // Get the output String from the command
    let exit_node_list_string = String::from_utf8(exit_node_list_cmd.unwrap().stdout).unwrap();

    // Return if there are no exit nodes
    if exit_node_list_string.is_empty() {
        println!("No exit nodes found!");
        return vec!["No exit nodes found!".to_string()];
    }

    // Get all of the exit node hostnames out of the output
    let fq_hostname_reg = RegexBuilder::new(r#"\w.\w.ts.net"#).build().ok().unwrap();
    let mut exit_node_list: Vec<String> = vec!["None".to_string()];
    
    let mut exit_node_map: Vec<String> = exit_node_list_string.lines()
        .filter(|line| fq_hostname_reg.is_match(line))
        .map(|hostname| {
            hostname.split_whitespace().nth(1).expect("Could not get node fully qualified hostname!")
                .split(".").next().expect("Could not get node hostname!").to_string()
            }
        )
        .collect();

    exit_node_list.append(&mut exit_node_map);

    exit_node_list
}

/// Set selected exit node as the exit node through Tailscale CLI
pub fn set_exit_node(exit_node: String) -> bool {
    let _exit_node_set_cmd = Command::new("tailscale")
        .args(["set", &format!("--exit-node={exit_node}")])
        .spawn()
        .expect("Set exit node was not successful!");

    exit_node.is_empty()
}
