use std::env;
use std::error::Error;
use rpassword::read_password_from_tty;

/// Environment variable to password to protect encrypted wallet.
static _ENV_VAR_BTC_TOOL_PWD: &str = "BTC_TOOL_PWD";

/// Help methods to read password from console
///
/// In test environments the password can be mocked by setting the
/// environment variable 'BTC_TOOL_PWD'
///
/// # Arguments
/// * prompt: Text to prompt in the console
pub fn read_password(prompt : &str) -> Result<String,Box<dyn Error>> {
   // Check Environment variable
    let retval;
    if env::var(_ENV_VAR_BTC_TOOL_PWD).is_ok() {
        retval = env::var(_ENV_VAR_BTC_TOOL_PWD).unwrap();
    }else {
        retval = read_password_from_tty(Some(prompt))?;
    }

    return Ok(retval)
}

pub fn read_verified_password() -> Result<String,Box<dyn Error>> {
    let mut retval;
    if env::var(_ENV_VAR_BTC_TOOL_PWD).is_ok() {
        retval = env::var(_ENV_VAR_BTC_TOOL_PWD).unwrap();
    }else{
        loop {
            retval = read_password_from_tty(Some("Enter Password :"))?;
            let verify  = read_password_from_tty(Some("Verify Password:"))?;
            if retval == verify {
                break;
            }else{
                println!("Entered password did not match, try again.")
            }
        }
    }
    return Ok(retval)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn read_password_using_environment_variable() {
        // Setup Remove existing file
        env::set_var(_ENV_VAR_BTC_TOOL_PWD, "foo123");

        // When
        let pwd = read_password("Enter the pwd: ").unwrap();
        // Then
        assert_eq!(pwd, "foo123".to_string());
    }
}
