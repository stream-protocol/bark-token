// state.rs

use anchor_lang::prelude::*;
use spl_pod::primitives::PodBool;
use spl_tlv_account_resolution::account::ExtraAccountMetaList;

#[derive(Accounts)]
pub struct State<'info> {
    pub owner: Pubkey,
    pub paused: bool,
    pub fee_rate: u8,  // New field for storing the fee rate
    pub fee_wallet: AccountInfo<'info>,  // New field for storing the fee wallet
}

impl<'info> State<'info> {
    pub fn new(ctx: Context<Initialize>, paused: bool, fee_rate: u8, fee_wallet: AccountInfo<'info>) -> Result<Self> {
        let state = &mut ctx.accounts.state;
        msg!("Initializing Bark program");
        state.owner = *ctx.accounts.authority.key;
        state.paused = paused;
        state.fee_rate = fee_rate;
        state.fee_wallet = fee_wallet;
        Ok(Self { owner: state.owner, paused: state.paused, fee_rate: state.fee_rate, fee_wallet: state.fee_wallet })
    }
}
