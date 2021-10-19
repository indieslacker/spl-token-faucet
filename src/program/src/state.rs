use solana_program::program_error::ProgramError;
use solana_program::program_pack::{IsInitialized, Pack, Sealed};
use solana_program::pubkey::Pubkey;

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

use crate::error::FaucetError;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Faucet {
    pub is_initialized: bool,
    pub mint: Pubkey,
    pub amount: u64,
}

impl Sealed for Faucet {}

impl IsInitialized for Faucet {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for Faucet {
    const LEN: usize = 41;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < Faucet::LEN {
            return Err(FaucetError::IncorrectInitializationData.into());
        }
        let src = array_ref![src, 0, Faucet::LEN];
        let (is_initialized, amount, mint) = array_refs![src, 1, 8, 32];

        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(FaucetError::IncorrectInitializationData.into()),
        };
        Ok(Self {
            is_initialized,
            amount: u64::from_le_bytes(*amount),
            mint: Pubkey::new_from_array(*mint),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Faucet::LEN];
        let (is_initialized_dst, amount_dst, mint_dst) =
            mut_array_refs!(dst, 1, 8, 32);
        let &Faucet {
            is_initialized,
            ref mint,
            amount,
        } = self;

        is_initialized_dst[0] = is_initialized as u8;
        *amount_dst = amount.to_le_bytes();
        *mint_dst = mint.to_bytes();
    }
}
