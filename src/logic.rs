use std::{
    collections::VecDeque, 
    io::{Error, Read}, 
    process::{Command, Output, Stdio},
};

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

    if con_status.contains("true") {
        return true;
    }

    false
}

pub fn get_tailscale_devices() -> Vec<String> {
    let ts_status_cmd = Command::new("tailscale")
        .arg("status")
        .output();

    let out = match String::from_utf8(ts_status_cmd.unwrap().stdout) {
        Ok(s) => s,
        Err(e) => format!("Error getting the status output: {e}"),
    };

    let mut status_output: VecDeque<String> = out.lines().map(|line| {
        line.split_whitespace().skip(1).next().expect("Device name not found").to_string()
    }).collect();

    status_output.pop_front();

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

    if ssh_status.contains("true") {
        return true;
    }

    false
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

    if ssh_status.contains("true") {
        return true;
    }

    false
}

/// Get available devices
pub fn _get_available_devices() -> String {
    let cmd = Command::new("tailscale")
        .args(["status", "--active"])
        .output();


    String::from_utf8(cmd.unwrap().stdout).unwrap()
}

/// Set the Tailscale connectio up/down
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
        let _ = match path {
            // If there is path value
            Some(p) => {
                // Send the file
                let cmd = Command::new("tailscale")
                .args(["file", "cp", p, &format!("{target}:")])
                .spawn();

                // Check for errors from the tailscale command
                let _ = match cmd.unwrap().stderr {
                    Some(mut err) => {
                        // Update the err_str variable with the error and continue
                        // to the next file.
                        let _ = err.read_to_string(&mut err_str);
                        continue;
                    }
                    // If there's no error, we don't need to do anything.
                    None => {}
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
    let rx_status = if rx_stderr.is_empty() {
        "Recieved file(s) in Downloads!".to_string()
    } else {
        String::from_utf8(rx_stderr).unwrap()
    };

    // Return the recieve status
    rx_status
}

/// Toggle SSH on/off
pub fn set_ssh(ssh: bool) -> bool {
    let cmd: Result<Output, Error>;
    
    if ssh {
        cmd = Command::new("tailscale")
        .args(["set", "--ssh"])
        .output();
    } else {
        cmd = Command::new("tailscale")
            .args(["set", "--ssh=false"])
            .output();
    }

    match cmd {
        Ok(_) => true,
        Err(e) => {
            println!("Error occurred: {e}");
            false
        }
    }
}

/// Toggle accept-routes on/off
pub fn set_routes(accept_routes: bool) -> bool {
    let cmd: Result<Output, Error>;
    
    if accept_routes {
        cmd = Command::new("tailscale")
        .args(["set", "--accept-routes"])
        .output();
    } else {
        cmd = Command::new("tailscale")
            .args(["set", "--accept-routes=false"])
            .output();
    }

    match cmd {
        Ok(_) => true,
        Err(e) => {
            println!("Error occurred: {e}");
            false
        }
    }
}