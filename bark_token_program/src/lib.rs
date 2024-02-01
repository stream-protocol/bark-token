// lib.rs

use anchor_lang::prelude::*;
use solana_program::program::invoke_signed;
use solana_program::system_instruction;
use spl_pod::primitives::PodBool;
use spl_token::instruction::transfer_checked_with_fee;
use spl_tlv_account_resolution::account::{ExtraAccountMeta, ExtraAccountMetaList};
use spl_transfer_hook_interface::collect_extra_account_metas_signer_seeds;
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum InstructionType {
    Execute = 1,
    // Add other instructions as needed
}

declare_id!("bark8LXsP1oCtaFM2KdQpBvXgEVWPZ1nm5hecFFUFeX");

pub const EXECUTE_IX_TAG_LE: [u8; 8] = [105, 37, 101, 197, 75, 251, 102, 26];

#[program]
pub mod transferhook {
    use super::*;

    #[state]
    pub struct State {
        pub owner: Pubkey,
        pub paused: bool,
        pub fee_rate: u8,
        pub fee_wallet: Pubkey,
    }

    impl State {
        pub fn initialize(
            ctx: Context<Initialize>,
            paused: bool,
            fee_rate: u8,
            fee_wallet: Pubkey,
        ) -> Result<Self> {
            let state = &mut ctx.accounts.state;
            msg!("Initializing Bark program");
            state.owner = *ctx.accounts.authority.key;
            state.paused = paused;
            state.fee_rate = fee_rate;
            state.fee_wallet = fee_wallet;
            Ok(Self {
                owner: state.owner,
                paused: state.paused,
                fee_rate: state.fee_rate,
                fee_wallet: state.fee_wallet,
            })
        }

        pub fn change_paused_state(ctx: Context<ChangePausedState>, paused: bool) -> Result<()> {
            msg!("Changing paused state to {}", paused);

            // Check authority.
            if *ctx.accounts.authority.key != ctx.accounts.state.owner {
                msg!("Authority does not match state owner");
                return Err(ProgramError::Custom(123)); // Provide a meaningful error code
            }

            let state = &mut ctx.accounts.state;
            state.paused = paused;

            msg!("State program ID: {}", state.key());
            msg!("Change paused state to {}", state.paused);

            Ok(())
        }

        pub fn initialize_extra_account_meta_list(
            ctx: Context<InitializeExtraAccountMetaList>,
            bump_seed: u8,
        ) -> Result<()> {
            let account_metas = vec![ExtraAccountMeta {
                discriminator: 0,
                address_config: ctx.accounts.state.key().to_bytes(),
                is_signer: PodBool::from(false),
                is_writable: PodBool::from(true),
            }];

            let bump_seed = [bump_seed];
            let signer_seeds =
                collect_extra_account_metas_signer_seeds(ctx.accounts.mint.key, &bump_seed);
            let account_size = ExtraAccountMetaList::size_of(account_metas.len())?;
            invoke_signed(
                &system_instruction::allocate(ctx.accounts.extra_account.key, account_size as u64),
                &[ctx.accounts.extra_account.clone()],
                &[&signer_seeds],
            )?;
            invoke_signed(
                &system_instruction::assign(ctx.accounts.extra_account.key, ctx.program_id),
                &[ctx.accounts.extra_account.clone()],
                &[&signer_seeds],
            )?;

            let mut data = ctx.accounts.extra_account.try_borrow_mut_data()?;
            ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &account_metas)?;

            msg!(
                "Extra account meta list initialized on {}",
                ctx.accounts.extra_account.key()
            );
            Ok(())
        }
    }

    pub fn transfer_hook(ctx: Context<TransferHook>, amount: u64) -> Result<()> {
        let state = &ctx.accounts.state;

        msg!(
            "Executing transfer hook logic. State program ID: {}",
            state.key()
        );
        msg!("Transfer hook is paused: {}", state.paused);

        if state.paused {
            msg!("Transfer hook is paused");
            return Err(ProgramError::Custom(123)); // Provide a meaningful error code
        }

        // Bark-specific transfer hook logic
        handle_transfer_hook_logic(&ctx.accounts, amount)?;

        Ok(())
    }

    fn handle_transfer_hook_logic(accounts: &TransferHook, amount: u64) -> Result<()> {
        // Access relevant accounts
        let source = &accounts.source;
        let destination = &accounts.destination;
        let mint = &accounts.mint;
        let authority = &accounts.authority;
        let extra_account = &accounts.extra_account;

        // Example Bark-specific logic:
        msg!("Bark-specific transfer hook logic: Transfer amount {}", amount);

        // Calculate transfer fee
        let transfer_fee = (amount * u64::from(accounts.state.fee_rate)) / 100;

        // Transfer tokens with fee
        transfer_tokens_with_fee(
            &ctx.program_id,
            &[
                source.clone(),
                destination.clone(),
                authority.clone(),
                // Fee wallet account
                AccountInfo {
                    key: &accounts.state.fee_wallet,
                    is_signer: false,
                    is_writable: true,
                },
            ],
            amount,
            transfer_fee,
        )?;

        // Bark-specific logic ends

        Ok(())
    }

    fn transfer_tokens_with_fee(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        fee: u64,
    ) -> Result<()> {
        let cpi_accounts = transfer_checked_with_fee::TransferWithFee {
            from: accounts[0].clone(),
            to: accounts[1].clone(),
            authority: accounts[2].clone(),
            token_program: accounts[3].clone(),
        };

        let cpi_program = accounts[3].key.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        transfer_checked_with_fee::transfer_with_fee(cpi_ctx, amount, fee)?;

        Ok(())
    }

    pub fn fallback(program_id: &Pubkey, accounts: &[AccountInfo], ix_data: &[u8]) -> Result<()> {
        let mut ix_data: &[u8] = ix_data;
        let sighash: [u8; 8] = {
            let mut sighash: [u8; 8] = [0; 8];
            sighash.copy_from_slice(&ix_data[..8]);
            ix_data = &ix_data[8..];
            sighash
        };
        match sighash {
            EXECUTE_IX_TAG_LE => {
                __private::__global::transfer_hook(program_id, accounts, ix_data)
            }
            _ => Err(ProgramError::InvalidInstructionData.into()),
        }
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init,
        seeds = [authority.key().as_ref()],
        bump,
        payer = authority,
        space = 8 + 128)
    ]
    pub state: Account<'info, State>,
    pub system_program: Program<'info, System>,
    #[account(mut)]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct TransferHook<'info> {
    pub source: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub destination: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    #[account(
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump)
    ]
    pub extra_account: AccountInfo<'info>,
    pub state: Account<'info, State>,
}

#[derive(Accounts)]
pub struct InitializeExtraAccountMetaList<'info> {
    #[account(mut,
        seeds = [b"extra-account-metas", mint.key().as_ref()],
        bump)
    ]
    pub extra_account: AccountInfo<'info>,
    #[account(mut)]
    pub state: Account<'info, State>,
    pub mint: AccountInfo<'info>,
    pub authority: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(paused: bool, fee_rate: u8, fee_wallet: Pubkey)]
pub struct ChangePausedState<'info> {
    #[account(mut)]
    pub state: Account<'info, State>,
    pub authority: Signer<'info>,
}

#[account]
pub struct State {
    pub owner: Pubkey,
    pub paused: bool,
    pub fee_rate: u8,
    pub fee_wallet: Pubkey,
}
