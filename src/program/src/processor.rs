use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::state::Mint;

use crate::error::FaucetError;
use crate::instruction::FaucetInstruction;
use crate::state::Faucet;

pub struct Processor;

impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = FaucetInstruction::unpack(input)?;
        match instruction {
            FaucetInstruction::InitFaucet { amount } => {
                msg!("Instruction: InitFaucet");
                Self::process_init_faucet(accounts, amount, program_id)?
            }
            FaucetInstruction::MintTokens { amount } => {
                msg!("Instruction: MintTokens");
                Self::process_mint_tokens(accounts, amount, program_id)?
            }
        }
        Ok(())
    }

    pub fn process_init_faucet(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let (pda, _nonce) = Pubkey::find_program_address(&[b"faucet"], program_id);

        let mint_account = next_account_info(account_info_iter)?;
        let mint_state = Mint::unpack(&mint_account.data.borrow())?;

        if pda
            != mint_state
                .mint_authority
                .ok_or(ProgramError::InvalidAccountData)?
        {
            return Err(FaucetError::IncorrectMintAuthority.into());
        }

        let faucet_account = next_account_info(account_info_iter)?;

        let mut faucet = Faucet::unpack_unchecked(&faucet_account.data.borrow())?;
        if faucet.is_initialized {
            return Err(FaucetError::AccountAlreadyInUse.into());
        }

        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if !rent.is_exempt(faucet_account.lamports(), faucet_account.data_len()) {
            return Err(FaucetError::AccountNotRentExempt.into());
        }

        faucet.is_initialized = true;
        faucet.amount = amount;
        faucet.mint = *mint_account.key;

        Faucet::pack(faucet, &mut faucet_account.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_mint_tokens(
        accounts: &[AccountInfo],
        amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let pda_account = next_account_info(account_info_iter)?;
        let (pda, nonce) = Pubkey::find_program_address(&[b"faucet"], program_id);

        if pda != *pda_account.key {
            return Err(ProgramError::InvalidAccountData);
        }

        let mint_acc = next_account_info(account_info_iter)?;
        let token_dest_acc = next_account_info(account_info_iter)?;
        let token_program = next_account_info(account_info_iter)?;

        let faucet_acc = next_account_info(account_info_iter)?;

        let faucet = Faucet::unpack_from_slice(&faucet_acc.data.borrow())?;

        if faucet.mint != *mint_acc.key {
            return Err(FaucetError::InvalidMint.into());
        }

        if amount > faucet.amount {
            return Err(FaucetError::RequestingTooManyTokens.into());
        }

        let ix = spl_token::instruction::mint_to(
            token_program.key,
            mint_acc.key,
            token_dest_acc.key,
            &pda,
            &[],
            amount,
        )?;

        solana_program::program::invoke_signed(
            &ix,
            &[
                mint_acc.clone(),
                token_dest_acc.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"faucet"[..], &[nonce]]],
        )?;
        Ok(())
    }
}
