use std::convert::TryInto;
use std::mem::size_of;

use crate::error::FaucetError;
use solana_program::program_error::ProgramError;

#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum FaucetInstruction {
    /// Initializes a faucet
    ///
    /// 0. `[]` Token Mint Account, mint authority must equal Program Derived Address
    /// 1. `[writable]` Faucet Account
    /// 2. `[]` Rent Sysvar
    InitFaucet {
        /// all other accounts may only mint this amount per ix
        amount: u64,
    },
    /// Mints Tokens
    ///
    /// 0. `[]` The mint authority - Program Derived Address
    /// 1. `[writable]` Token Mint Account
    /// 2. `[writable]` Destination Account
    /// 3. `[]` The SPL Token Program
    /// 4. `[]` The Faucet Account
    MintTokens { amount: u64 },
}

impl FaucetInstruction {
    /// Unpacks a byte buffer into a [FaucetInstruction](enum.FaucetInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input.split_first().ok_or(FaucetError::InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(FaucetError::InvalidInstruction)?;
                Self::InitFaucet { amount }
            }
            1 => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(FaucetError::InvalidInstruction)?;
                Self::MintTokens { amount }
            }
            _ => return Err(FaucetError::InvalidInstruction.into()),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            Self::InitFaucet { amount } => {
                buf.push(0);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
            Self::MintTokens { amount } => {
                buf.push(1);
                buf.extend_from_slice(&amount.to_le_bytes());
            }
        }

        buf
    }
}
