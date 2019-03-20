pub fn log_info(event: &str, content: &str) {
    debug!("{}: {}", event, content);
}

pub fn log_xevent(content: &str) {
    debug!("{}: {}", "XEVENT", content);
}
