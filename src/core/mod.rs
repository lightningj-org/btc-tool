use std::error::Error;
use std::io::stdin;

pub mod settings;
pub mod walletdata;
pub mod password;
pub mod walletcontainer;

/// Help method that prompts string and reads input from stdin and
/// expects 'yes' and 'no'.
pub fn get_confirmation(prompt : &str) -> Result<bool, Box<dyn Error>>{
    let retval :bool;
    loop {
        println!("{}",prompt);
        let mut word = String::new();
        let _ = stdin().read_line(&mut word)?;
        let answer = word.to_lowercase().trim().to_string();
        if answer.eq("yes") {
            retval = true;
            break;
        }
        if answer.eq("no") {
            retval = false;
            break;
        }
        println!("Invalid input, enter 'yes' or 'no'.")
    }
    return Ok(retval);
}
