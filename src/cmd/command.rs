use std::error::Error;

/// Trait all Commands have to implement
pub trait Command {
    /// Command to run the sub command.
    fn execute(self : &Self) -> Result<(), Box<dyn Error>>;
}