use colored::*;

pub fn log_info(type_: &str, content: &str) {
    println!("{}: {}", type_.yellow(), content);
}

pub fn log_xevent(content: &str) {
    //println!("{}: {}", "XEVENT".cyan(), content);
}
