use std::{
    io::{Error, Read, Write}, 
    process::{Command, Output, Stdio}
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

    let status_output: Vec<String> = out.lines().map(|line| {
        line.split_whitespace().skip(1).next().expect("Device name not found").to_string()
    }).collect();

    status_output
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
pub fn get_available_devices() -> String {
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

pub fn tailscale_send(file_paths: Vec<Option<String>>, target: &str) -> Vec<Option<String>> {
    let mut status = Vec::<Option<String>>::new();

    for path in file_paths.iter() {
        let mut this_file_status = String::new();
        let _ = match path {
            Some(p) => {
                let cmd = Command::new("tailscale")
                .args(["file", "cp", p, &format!("{target}:")])
                .spawn();

                let _ = match cmd.unwrap().stderr {
                    Some(mut err) => err.read_to_string(&mut this_file_status),
                    None => {
                        this_file_status = String::from("The file was successfully sent!");
                        Ok(0)
                    },
                };

                status.push(Some(this_file_status.clone()));
            }
            None => status.push(Some(String::from("Something went wrong sending the file..."))),
        };
    }

    status
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