use std::io;
use termion::color;
pub fn get_input(prompt: &str) -> Result<String, String> {
    println!("{}{prompt}{}", color::Fg(color::Green), color::Fg(color::Reset));
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => Ok(input.trim().to_string()),
        Err(error) => Err(format!("Error reading input: {}", error)),
    }
}
