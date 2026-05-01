use notify_rust::{Notification, Timeout};

const APP_NAME: &str = "GUI Scale Applet";
const ICON: &str = "tailscale-icon";

/// Send a desktop notification
fn send_notification(summary: &str, body: &str) {
    if let Err(e) = Notification::new()
        .appname(APP_NAME)
        .summary(summary)
        .body(body)
        .icon(ICON)
        .timeout(Timeout::Milliseconds(5000))
        .show()
    {
        eprintln!("Failed to show notification: {e}");
    }
}

pub fn notify_connection_change(connected: bool) {
    let body = if connected {
        "Tailscale connected"
    } else {
        "Tailscale disconnected"
    };
    send_notification("Tailscale", body);
}

pub fn notify_new_device(device_name: &str) {
    send_notification("New Tailscale device", device_name);
}

pub fn notify_incoming_files() {
    send_notification("TailDrop", "Incoming files are waiting");
}

pub fn notify_files_sent(device: &str, count: usize) {
    let body = format!("Sent {count} file(s) to {device}");
    send_notification("TailDrop", &body);
}

pub fn notify_files_received(path: &str) {
    let body = format!("Saved incoming files to {path}");
    send_notification("TailDrop", &body);
}

pub fn notify_account_switched(account: &str) {
    let body = format!("Switched to account: {account}");
    send_notification("Tailscale", &body);
}
