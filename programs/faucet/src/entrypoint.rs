//! Program entrypoint
use solana_program::entrypoint;

use crate::processor::process_instruction;

entrypoint!(process_instruction);
