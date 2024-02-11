use thiserror::Error;
use solana_program::program_error::ProgramError;
use borsh::DecodeError;  // Assuming DecodeError is from the borsh crate

#[derive(Debug, Error, Copy, Clone)]
pub enum CustomError {
    #[error("Invalid instruction data")]
    InvalidInstructionData,
    #[error("Other program error: {0}")]
    OtherProgramError(u32),
}

// Conversion from CustomError to ProgramError
impl From<CustomError> for ProgramError {
    fn from(e: CustomError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// Implementation of DecodeError trait for CustomError
impl<T> DecodeError<T> for CustomError {
    fn type_of() -> &'static str {
        "CustomError"
    }
}
