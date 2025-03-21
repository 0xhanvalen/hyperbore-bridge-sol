use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

declare_id!("2J13KUMb3sQi6PwUkvx1hiytx7jUATXH92AMCQpdRYmT");

pub const AUTHORIZED_LAUNCHER: Pubkey = pubkey!("9FEDyP1t345xFKVrJPN2TgQvQEJGz8KXE2xPV6TVXYY6");

#[program]
pub mod bridge_sol {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, tax: u16) -> Result<()> {
        let validator_account = &mut ctx.accounts.validator_account;
        validator_account.validator_pubkey = ctx.accounts.owner.key();
        validator_account.times_validated = 0;
        validator_account.bump = ctx.bumps.validator_account;
        let pool_state = &mut ctx.accounts.pool_state;
        pool_state.owner = ctx.accounts.owner.key();
        pool_state.proposed_owner = pool_state.owner;
        pool_state.usdc_mint = ctx.accounts.usdc_mint.key();
        pool_state.paused = false;
        pool_state.required_signatures = 1; // Example: Require 2 validators
        pool_state.total_volume = 0;
        pool_state.tax = tax;
        pool_state.treasury = ctx.accounts.treasury.key();
        pool_state.bump = ctx.bumps.pool_state;
        validator_account.pool = pool_state.key();
        pool_state.validators = [Pubkey::default(); 10];
        pool_state.validators[0] = validator_account.key();
        emit!(PoolCreated {
            address: pool_state.key(),
            treasury: pool_state.treasury,
            tax: pool_state.tax,
            timestamp: Clock::get()?.unix_timestamp,
        });
        emit!(ValidatorAdded {
            address: validator_account.key(),
            pool_state: pool_state.key(),
            validator_address: validator_account.validator_pubkey,
            required_validators: pool_state.required_signatures,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn update_state(ctx: Context<UpdateStateContext>, args: ConfigUpdateArgs) -> Result<()> {
        let state = &mut ctx.accounts.pool_state;
        if let Some(new_treasury) = args.treasury {
            state.treasury = new_treasury;
        };
        if let Some(proposed_owner) = args.owner {
            state.proposed_owner = proposed_owner;
        };
        if let Some(new_tax) = args.tax {
            state.tax = new_tax;
        };
        if let Some(new_paused) = args.paused {
            state.paused = new_paused;
        };
        Ok(())
    } 

    pub fn add_validator(ctx: Context<AddValidatorContext>) -> Result<()> {
        let state = &mut ctx.accounts.pool_state;
        let mut count = 0;
        for i in 0..10 {
            if state.validators[i] != Pubkey::default() {
                count += 1;
            }
        }
        if count >= 10 {
            return Err(error!(ErrorCode::TooManyValidators));
        }
        let validator_account = &mut ctx.accounts.validator_account;
        validator_account.pool = state.key();
        validator_account.validator_pubkey = ctx.accounts.new_validator.key();
        validator_account.times_validated = 0;
        validator_account.bump = ctx.bumps.validator_account;
        state.validators[count] = validator_account.key();
        state.required_signatures = (count as u8) + 1; // zero indexing, idk, if validators[0] is filled, this would be 1, if validators[0] and validators[1], this would be 2, etc
        emit!(ValidatorAdded {
            address: validator_account.key(),
            pool_state: state.key(),
            validator_address: validator_account.validator_pubkey,
            required_validators: state.required_signatures,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn remove_validator(ctx: Context<RemoveValidatorContext>) -> Result<()> {
        let state = &mut ctx.accounts.pool_state;
        let old_validator_account = &ctx.accounts.validator_account;
        // find the index of the old validator
        let mut index = 0;
        let mut found_index = false;
        for i in 0..10 {
            if state.validators[i] == old_validator_account.key() {
                index = i;
                found_index = true;
            }
        }
        if found_index == false {
            return Err(error!(ErrorCode::ValidatorDoesNotExist))
        }
        // oh god, shuffling an index array smdh
        // Shift all validators above the found index down by one
        for i in index..9 {
            state.validators[i] = state.validators[i + 1];
        }
        // Set the last index to the default value
        state.validators[9] = Pubkey::default();

        // Update the required_signatures to reflect the new count of validators
        state.required_signatures -= 1;

        // Emit an event for the removed validator
        emit!(ValidatorRemoved {
            address: old_validator_account.key(),
            pool_state: state.key(),
            validator_address: old_validator_account.validator_pubkey,
            required_validators: state.required_signatures,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }
}

//  ========================================================================================================  //
//  Account Contexts                                                                                          //
//    ▄████████  ▄████████  ▄████████     ███           ▄████████     ███     ▀████    ▐████▀    ▄████████    //
//    ███    ███ ███    ███ ███    ███ ▀█████████▄      ███    ███ ▀█████████▄   ███▌   ████▀    ███    ███   //
//    ███    ███ ███    █▀  ███    █▀     ▀███▀▀██      ███    █▀     ▀███▀▀██    ███  ▐███      ███    █▀    //
//    ███    ███ ███        ███            ███   ▀      ███            ███   ▀    ▀███▄███▀      ███          //
//  ▀███████████ ███        ███            ███          ███            ███        ████▀██▄     ▀███████████   //
//    ███    ███ ███    █▄  ███    █▄      ███          ███    █▄      ███       ▐███  ▀███             ███   //
//    ███    ███ ███    ███ ███    ███     ███          ███    ███     ███      ▄███     ███▄     ▄█    ███   //
//    ███    █▀  ████████▀  ████████▀     ▄████▀        ████████▀     ▄████▀   ████       ███▄  ▄████████▀    //
//  ========================================================================================================  //    

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        mut,
        constraint = owner.key() == AUTHORIZED_LAUNCHER,
    )]
    pub owner: Signer<'info>,

    #[account(
        init, 
        payer = owner, 
        space = 8 + PoolState::INIT_SPACE,
        seeds = [b"pool_state"],
        bump
    )]
    pub pool_state: Account<'info, PoolState>,

    /// CHECK: The treasury where fees go
    pub treasury: AccountInfo<'info>,

    /// CHECK: This is the USDC mint address
    pub usdc_mint: AccountInfo<'info>,

    #[account(
        init,
        payer = owner,
        space = 8 + ValidatorAccount::INIT_SPACE,
        seeds = [b"validator_account", owner.key().as_ref()],
        bump
    )]
    pub validator_account: Account<'info, ValidatorAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateStateContext<'info> {
    #[account(
        mut,
    )]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool_state"],
        bump = pool_state.bump,
        constraint = pool_state.owner == owner.key(),
    )]
    pub pool_state: Account<'info, PoolState>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddValidatorContext<'info> {
    #[account(
        mut,
    )]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool_state"],
        bump = pool_state.bump,
        constraint = pool_state.owner == owner.key(),
    )]
    pub pool_state: Account<'info, PoolState>,

    /// CHECK: The validator being added
    pub new_validator: AccountInfo<'info>,

    #[account(
        init,
        payer = owner,
        space = 8 + ValidatorAccount::INIT_SPACE,
        seeds = [b"validator_account", new_validator.key().as_ref()],
        bump
    )]
    pub validator_account: Account<'info, ValidatorAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveValidatorContext<'info> {
    #[account(
        mut,
    )]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool_state"],
        bump = pool_state.bump,
        constraint = pool_state.owner == owner.key(),
    )]
    pub pool_state: Account<'info, PoolState>,

    /// CHECK: The validator being removed
    pub old_validator: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"validator_account", old_validator.key().as_ref()],
        bump = validator_account.bump,
        close = owner
    )]
    pub validator_account: Account<'info, ValidatorAccount>,
    pub system_program: Program<'info, System>,
}

// ================================================================================================================================  //
// Arg Structs - Function Argument Definitions                                                                                       //
//   ▄████████    ▄████████    ▄██████▄          ▄████████     ███        ▄████████ ███    █▄   ▄████████     ███        ▄████████   // 
//   ███    ███   ███    ███   ███    ███        ███    ███ ▀█████████▄   ███    ███ ███    ███ ███    ███ ▀█████████▄   ███    ███  // 
//   ███    ███   ███    ███   ███    █▀         ███    █▀     ▀███▀▀██   ███    ███ ███    ███ ███    █▀     ▀███▀▀██   ███    █▀   // 
//   ███    ███  ▄███▄▄▄▄██▀  ▄███               ███            ███   ▀  ▄███▄▄▄▄██▀ ███    ███ ███            ███   ▀   ███         // 
// ▀███████████ ▀▀███▀▀▀▀▀   ▀▀███ ████▄       ▀███████████     ███     ▀▀███▀▀▀▀▀   ███    ███ ███            ███     ▀███████████  // 
//   ███    ███ ▀███████████   ███    ███               ███     ███     ▀███████████ ███    ███ ███    █▄      ███              ███  // 
//   ███    ███   ███    ███   ███    ███         ▄█    ███     ███       ███    ███ ███    ███ ███    ███     ███        ▄█    ███  // 
//   ███    █▀    ███    ███   ████████▀        ▄████████▀     ▄████▀     ███    ███ ████████▀  ████████▀     ▄████▀    ▄████████▀   // 
//                ███    ███                                              ███    ███                                                 // 
// ================================================================================================================================  //

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ConfigUpdateArgs {
    pub treasury: Option<Pubkey>,
    pub owner: Option<Pubkey>,
    pub tax: Option<u16>,
    pub paused: Option<bool>,
}

// ========================================================================================================== //
// Account Definitions                                                                                        //
//   ▄████████  ▄████████  ▄████████     ███          ████████▄     ▄████████    ▄████████    ▄████████       //
//   ███    ███ ███    ███ ███    ███ ▀█████████▄      ███   ▀███   ███    ███   ███    ███   ███    ███      //
//   ███    ███ ███    █▀  ███    █▀     ▀███▀▀██      ███    ███   ███    █▀    ███    █▀    ███    █▀       //
//   ███    ███ ███        ███            ███   ▀      ███    ███  ▄███▄▄▄      ▄███▄▄▄       ███             //
// ▀███████████ ███        ███            ███          ███    ███ ▀▀███▀▀▀     ▀▀███▀▀▀     ▀███████████      //
//   ███    ███ ███    █▄  ███    █▄      ███          ███    ███   ███    █▄    ███                 ███      //
//   ███    ███ ███    ███ ███    ███     ███          ███   ▄███   ███    ███   ███           ▄█    ███      //
//   ███    █▀  ████████▀  ████████▀     ▄████▀        ████████▀    ██████████   ███         ▄████████▀       //
// ========================================================================================================== //  

#[account]
#[derive(InitSpace)]
pub struct PoolState {
    pub owner: Pubkey,
    pub proposed_owner: Pubkey,
    pub usdc_mint: Pubkey,
    pub paused: bool,
    pub validators: [Pubkey; 10],
    pub required_signatures: u8, // should match the initialized pubkeys in validators, used as iterator
    pub tax: u16, // basis point tax on each transaction
    pub total_volume: u64,
    pub treasury: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct ValidatorAccount {
    pub pool: Pubkey,  // parent pool
    pub validator_pubkey: Pubkey,
    pub times_validated: u64,
    pub bump: u8,
}
// ========================================================================= //
// Events                                                                    //
//   ▄████████   ▄█    █▄     ▄████████ ███▄▄▄▄       ███        ▄████████   //
//   ███    ███ ███    ███   ███    ███ ███▀▀▀██▄ ▀█████████▄   ███    ███   //
//   ███    █▀  ███    ███   ███    █▀  ███   ███    ▀███▀▀██   ███    █▀    //
//  ▄███▄▄▄     ███    ███  ▄███▄▄▄     ███   ███     ███   ▀   ███          //
// ▀▀███▀▀▀     ███    ███ ▀▀███▀▀▀     ███   ███     ███     ▀███████████   //
//   ███    █▄  ███    ███   ███    █▄  ███   ███     ███              ███   //
//   ███    ███ ███    ███   ███    ███ ███   ███     ███        ▄█    ███   //
//   ██████████  ▀██████▀    ██████████  ▀█   █▀     ▄████▀    ▄████████▀    //
// ========================================================================  // 

#[event]
pub struct PoolCreated {
    pub address: Pubkey, // the pool state/config address
    pub treasury: Pubkey, // the treasury address
    pub tax: u16, // bps fee for transfers
    pub timestamp: i64, // when the shit happened
}

#[event]
pub struct ValidatorAdded {
    pub address: Pubkey, // the account for the validator
    pub pool_state: Pubkey, // the pool state this validator was added to
    pub validator_address: Pubkey, // the actual validator
    pub required_validators: u8, // how many validators there are now
    pub timestamp: i64,
}

#[event]
pub struct ValidatorRemoved {
    pub address: Pubkey, // the account for the validator
    pub pool_state: Pubkey, // the pool state this validator was removed from
    pub validator_address: Pubkey, // the actual validator that got canned
    pub required_validators: u8, // how many validators there are now
    pub timestamp: i64,
}

//  ==========================================================================  //
//  Error Codes / Errors                                                        //  
//   ▄████████    ▄████████    ▄████████  ▄██████▄     ▄████████    ▄████████   //
//   ███    ███   ███    ███   ███    ███ ███    ███   ███    ███   ███    ███  // 
//   ███    █▀    ███    ███   ███    ███ ███    ███   ███    ███   ███    █▀   // 
//  ▄███▄▄▄      ▄███▄▄▄▄██▀  ▄███▄▄▄▄██▀ ███    ███  ▄███▄▄▄▄██▀   ███         // 
// ▀▀███▀▀▀     ▀▀███▀▀▀▀▀   ▀▀███▀▀▀▀▀   ███    ███ ▀▀███▀▀▀▀▀   ▀███████████  // 
//   ███    █▄  ▀███████████ ▀███████████ ███    ███ ▀███████████          ███  // 
//   ███    ███   ███    ███   ███    ███ ███    ███   ███    ███    ▄█    ███  // 
//   ██████████   ███    ███   ███    ███  ▀██████▀    ███    ███  ▄████████▀   // 
//                ███    ███   ███    ███              ███    ███               //  
//  ==========================================================================  //  

#[error_code]
pub enum ErrorCode {
    #[msg("Too many validators. Max is 10. Remove one before adding another one.")]
    TooManyValidators,

    #[msg("This validator key doesn't exist")]
    ValidatorDoesNotExist,
}