use solana_program::program_error::ProgramError;

#[derive(Debug)]
pub enum ReclaimError {
    InvalidOwner,
    NotEnoughFractions,
    AlreadyReclaimed,
}

impl From<ReclaimError> for ProgramError {
    fn from(e: ReclaimError) -> ProgramError {
        ProgramError::Custom(e as u32)
    }
}
