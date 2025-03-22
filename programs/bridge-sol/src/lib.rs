use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use std::hash::{Hash, Hasher};
use sha2::{Digest, Sha256};
use secp256k1::{Message, PublicKey, Secp256k1};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

declare_id!("qbuMdeYxYJXBjU6C6qFKjZKjXmrU83eDQomHdrch826");

pub const AUTHORIZED_LAUNCHER: Pubkey = pubkey!("9FEDyP1t345xFKVrJPN2TgQvQEJGz8KXE2xPV6TVXYY6");
pub const MAX_SIGNATURES: usize = 16;
pub const MAX_VALIDATORS: usize = 16;

#[program]
pub mod bridge_sol {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, tax: u16) -> Result<()> {
        let pool_state = &mut ctx.accounts.pool_state;
        pool_state.owner = ctx.accounts.owner.key();
        pool_state.proposed_owner = pool_state.owner;
        pool_state.usdc_mint = ctx.accounts.usdc_mint.key();
        pool_state.paused = false;
        pool_state.required_signatures = 1;
        pool_state.total_volume = 0;
        pool_state.tax = tax;
        pool_state.accumulated_fees = 0;
        pool_state.treasury = ctx.accounts.treasury.key();
        pool_state.bump = ctx.bumps.pool_state;
        pool_state.validators = [Pubkey::default(); MAX_VALIDATORS];
        pool_state.validators[0] = ctx.accounts.owner.key();
        emit!(PoolCreated {
            address: pool_state.key(),
            treasury: pool_state.treasury,
            tax: pool_state.tax,
            timestamp: Clock::get()?.unix_timestamp,
        });
        emit!(ValidatorAdded {
            address: ctx.accounts.owner.key(),
            pool_state: pool_state.key(),
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
            // tax cannot be zero
            if new_tax == 0 {
                return Err(error!(ErrorCode::ZeroTax));
            }
            state.tax = new_tax;
        };
        if let Some(new_paused) = args.paused {
            state.paused = new_paused;
        };
        emit!(PoolStateUpdated {
            address: state.key(),
            treasury: state.treasury,
            proposed_owner: state.proposed_owner,
            tax: state.tax,
            paused: state.paused,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    } 

    pub fn accept_ownership(ctx: Context<AcceptOwnershipContext>) -> Result<()> {
        let state = &mut ctx.accounts.pool_state;
        let signer_key = ctx.accounts.signer.key();
        if signer_key == state.proposed_owner {
            state.owner = signer_key;
            state.proposed_owner = Pubkey::default();
            emit!(OwnerChanged {
                address: state.key(),
                new_owner: signer_key,
                timestamp: Clock::get()?.unix_timestamp,
            });
        } else {
            return Err(error!(ErrorCode::InvalidOwnershipChange));
        }
        Ok(())
    }

    pub fn add_validator(ctx: Context<AddValidatorContext>) -> Result<()> {
        let state = &mut ctx.accounts.pool_state;
        let new_validator_key = ctx.accounts.new_validator.key();
        let mut count = 0;
        for i in 0..MAX_VALIDATORS {
            if state.validators[i] != Pubkey::default() {
                count += 1;
            }
        }
        if count >= MAX_VALIDATORS {
            return Err(error!(ErrorCode::TooManyValidators));
        }
        state.validators[count] = new_validator_key;
        state.required_signatures = (count as u8) + 1; // zero indexing, idk, if validators[0] is filled, this would be 1, if validators[0] and validators[1], this would be 2, etc
        emit!(ValidatorAdded {
            address: new_validator_key,
            pool_state: state.key(),
            required_validators: state.required_signatures,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn remove_validator(ctx: Context<RemoveValidatorContext>) -> Result<()> {
        let state = &mut ctx.accounts.pool_state;
        let old_validator_key = &ctx.accounts.old_validator.key();
        // find the index of the old validator
        let mut index = 0;
        let mut found_index = false;
        for i in 0..MAX_VALIDATORS {
            if state.validators[i] == *old_validator_key {
                index = i;
                found_index = true;
            }
        }
        if found_index == false {
            return Err(error!(ErrorCode::ValidatorDoesNotExist))
        }
        // oh god, shuffling an index array smdh
        // Shift all validators above the found index down by one
        for i in index..(MAX_VALIDATORS - 1) {
            state.validators[i] = state.validators[i + 1];
        }
        // Set the last index to the default value
        state.validators[MAX_VALIDATORS - 1] = Pubkey::default();

        // Update the required_signatures to reflect the new count of validators
        state.required_signatures -= 1;

        // Emit an event for the removed validator
        emit!(ValidatorRemoved {
            address: *old_validator_key,
            pool_state: state.key(),
            required_validators: state.required_signatures,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn deposit_usdc(
            ctx: Context<DepositUSDCContext>,
            args: DepositUSDCArgs,
        ) -> Result<()> {
        let state = &mut ctx.accounts.pool_state;
        let depositor = &mut ctx.accounts.depositor;
        let state_ata = &mut ctx.accounts.pool_ata;
        let depositor_ata = &mut ctx.accounts.depositor_ata;
        let amount = args.amount;
        let recipient_evm_address = args.recipient_evm_address;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        depositor.key().hash(&mut hasher);
        state_ata.key().hash(&mut hasher);
        Clock::get()?.unix_timestamp.hash(&mut hasher);
        recipient_evm_address.hash(&mut hasher);
        amount.hash(&mut hasher);
        let nonce: u64 = hasher.finish();
        // calculate the tax from state.tax, a basis point fee
        let tax_amount = amount
            .checked_mul(state.tax as u64)
            .and_then(|result| result.checked_div(10000))
            .ok_or_else(|| error!(ErrorCode::TaxFailed))?;
        let deposit_amount = amount.checked_sub(tax_amount).ok_or_else(|| error!(ErrorCode::TaxFailed))?;
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: depositor_ata.to_account_info(),
                to: state_ata.to_account_info(),
                authority: depositor.to_account_info(),
            },
        );
        // NOTE: Transfers the original `amount` value - leaves the `tax` **inside** the pool!
        // tax can be recovered by accumulating the event emissions
        transfer(transfer_ctx, amount)?;
        state.accumulated_fees += tax_amount;
        if let Some(new_total) = state.total_volume.checked_add(amount) {
            state.total_volume = new_total;
        } else {
            // Log or handle the overflow case without throwing an error
            msg!("Warning: total_volume overflowed, skipping update.");
        }

        emit!(USDCDeposited {
            address: state.key(),
            depositor: depositor.key(),
            amount: deposit_amount, // Amount after fee
            recipient_evm_address,
            tax: tax_amount,
            nonce,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn withdraw_usdc(
        ctx: Context<WithdrawUSDCContext>,
        recipient: Pubkey,
        args: WithdrawUSDCArgs,
    ) -> Result<()> {
        let message = build_message(
            &args.nonce,
            &args.amount,
            &args.sender_evm_address,
            &recipient
        );
        let pool_state = &mut ctx.accounts.pool_state;
        let validators = pool_state.validators;
        let required_signatures = pool_state.required_signatures;
        // args.r, args.s, args.v need to be converted into &[Signatures], which is a struct with fields r, s, v.
        // the index of args.r, etc should match the index of &[Signatures].
        let mut signatures: [Signature; MAX_VALIDATORS] = [Signature {
            r: [0u8; 32],
            s: [0u8; 32],
            v: 0,
        }; MAX_VALIDATORS];
        for i in 0..MAX_VALIDATORS {
            signatures[i].r = args.r[i];
            signatures[i].s = args.s[i];
            signatures[i].v = args.v[i];
        }

        // **I GUESS!!!** validators[i] == signatures[i], maybe.
        // This is kind of fucked

        let verified = verify_signatures(
            &signatures,
            &validators,
            &message,
            required_signatures
        )?;
        if verified {
            // do the transfer
            let pool_ata = &mut ctx.accounts.pool_ata;
            let recipient_ata = &mut ctx.accounts.recipient_ata;

            let transfer_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: pool_ata.to_account_info(),
                    to: recipient_ata.to_account_info(),
                    authority: pool_state.to_account_info(),
                },
            );
            // NOTE: Transfers the amount **less tax** paid on the opposing chain
            // NOTE: transfers *from* EVM have already paid the tax to treasury
            transfer(transfer_ctx, args.amount)?;

            // flex the transfer
            if let Some(new_total) = pool_state.total_volume.checked_add(args.amount) {
                pool_state.total_volume = new_total;
            } else {
                // Log or handle the overflow case without throwing an error
                msg!("Warning: total_volume overflowed, skipping update.");
            }

            emit!(USDCWithdrawn {
                address: pool_state.key(),
                recipient: recipient.key(),
                amount: args.amount, // Amount after fee
                timestamp: Clock::get()?.unix_timestamp,
            });
        } else {
            return Err(error!(ErrorCode::FailedToValidate))
        }

        Ok(())
    }

    pub fn withdraw_fees(ctx: Context<WithdrawFeesContext>) -> Result<()> {
        let pool_state = &mut ctx.accounts.pool_state;
        let pool_ata = &mut ctx.accounts.pool_ata;
        let treasury_ata = &mut ctx.accounts.treasury_ata;

        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: pool_ata.to_account_info(),
                to: treasury_ata.to_account_info(),
                authority: pool_state.to_account_info(),
            },
        );
        transfer(transfer_ctx, pool_state.accumulated_fees)?;
        let emission_fees = pool_state.accumulated_fees;
        pool_state.accumulated_fees = 0;
        emit!(FeesWithdrawn {
            address: pool_state.key(),
            recipient: ctx.accounts.treasury.key(),
            amount: emission_fees,
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
pub struct AcceptOwnershipContext<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool_state"],
        bump = pool_state.bump,
        constraint = pool_state.proposed_owner == signer.key(),
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

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct DepositUSDCContext<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool_state"],
        bump = pool_state.bump,
        constraint = !pool_state.paused @ ErrorCode::PoolPaused,
    )]
    pub pool_state: Account<'info, PoolState>,

    #[account(
        mut,
        constraint = pool_state.usdc_mint == mint_account.key() @ ErrorCode::WrongToken,
    )]
    pub mint_account: Account<'info, Mint>,

    #[account(
        mut,
        constraint = depositor_ata.mint == mint_account.key(),
        constraint = depositor_ata.owner == depositor.key(),
    )]
    pub depositor_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = mint_account,
        associated_token::authority = pool_state,
    )]
    pub pool_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(recipient: Pubkey)]
pub struct WithdrawUSDCContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool_state"],
        bump = pool_state.bump,
        constraint = !pool_state.paused @ ErrorCode::PoolPaused,
    )]
    pub pool_state: Account<'info, PoolState>,

    #[account(
        mut,
        constraint = pool_state.usdc_mint == mint_account.key() @ ErrorCode::WrongToken,
    )]
    pub mint_account: Account<'info, Mint>,

    #[account(
        mut,
        constraint = recipient_ata.mint == mint_account.key(),
        constraint = recipient_ata.owner == recipient,
    )]
    pub recipient_ata: Account<'info, TokenAccount>,

    #[account(
        associated_token::mint = mint_account,
        associated_token::authority = pool_state,
    )]
    pub pool_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawFeesContext<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"pool_state"],
        bump = pool_state.bump,
        constraint = !pool_state.paused @ ErrorCode::PoolPaused,
        constraint = pool_state.owner == owner.key()
    )]
    pub pool_state: Account<'info, PoolState>,

    /// CHECK: The treasury
    pub treasury: AccountInfo<'info>,

    #[account(
        mut,
        constraint = pool_state.usdc_mint == mint_account.key() @ ErrorCode::WrongToken,
    )]
    pub mint_account: Account<'info, Mint>,

    #[account(
        mut,
        constraint = treasury_ata.mint == mint_account.key(),
        constraint = treasury_ata.owner == treasury.key(),
    )]
    pub treasury_ata: Account<'info, TokenAccount>,

    #[account(
        associated_token::mint = mint_account,
        associated_token::authority = pool_state,
    )]
    pub pool_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
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

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositUSDCArgs {
    pub amount: u64, // total amount to deposit
    pub recipient_evm_address: [u8; 20] // who the money goes to
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawUSDCArgs {
    pub amount: u64,                                // how much to withdraw
    pub sender_evm_address: [u8; 20],               // who sent the stuff. Note that recipient is missing - it's in a separate val for access by the instruction macro
    pub nonce: [u8; 32],                            // identifying nonce (bytes32 generated at evm side);
    pub r: [[u8; 32]; MAX_VALIDATORS],              // validator r values         
    pub s: [[u8; 32]; MAX_VALIDATORS],              // validator s values
    pub v: [u8; MAX_VALIDATORS],                    // validator v values
}

// =========================================================================================  //
// Helper Functions                                                                           //
//   ▄█    █▄       ▄████████  ▄█          ▄███████▄    ▄████████    ▄████████    ▄████████   //
//   ███    ███     ███    ███ ███         ███    ███   ███    ███   ███    ███   ███    ███  // 
//   ███    ███     ███    █▀  ███         ███    ███   ███    █▀    ███    ███   ███    █▀   // 
//  ▄███▄▄▄▄███▄▄  ▄███▄▄▄     ███         ███    ███  ▄███▄▄▄      ▄███▄▄▄▄██▀   ███         // 
// ▀▀███▀▀▀▀███▀  ▀▀███▀▀▀     ███       ▀█████████▀  ▀▀███▀▀▀     ▀▀███▀▀▀▀▀   ▀███████████  // 
//   ███    ███     ███    █▄  ███         ███          ███    █▄  ▀███████████          ███  // 
//   ███    ███     ███    ███ ███▌    ▄   ███          ███    ███   ███    ███    ▄█    ███  // 
//   ███    █▀      ██████████ █████▄▄██  ▄████▀        ██████████   ███    ███  ▄████████▀   // 
//                             ▀                                     ███    ███               // 
// =========================================================================================  //  

/// Builds a standardized message that should match the EVM side.
/// This ensures consistent signature verification across chains.
pub fn build_message(
    nonce: &[u8; 32],               // nonce emitted by evm
    amount: &u64,                   // amount to withdraw
    source_address: &[u8; 20],      // must be evm
    dest_address: &Pubkey,          // must be sol
) -> [u8; 32] {
    // Create a buffer to hold all the message components
    let mut message = Vec::with_capacity(1 + 32 + 32 + 32 + 32);
    
    // Add chain identifier (1 byte)
    message.push(1);
    
    // Add nonce (32 bytes)
    message.extend_from_slice(nonce);
    
    // Add amount (convert to standard 32 bytes big-endian)
    let mut amount_bytes = [0u8; 32];
    let amount_be = amount.to_be_bytes();
    amount_bytes[32 - amount_be.len()..].copy_from_slice(&amount_be);
    message.extend_from_slice(&amount_bytes);
    
    // Add source address (padded to 32 bytes)
    let mut source_address_padded = [0u8; 32];
    let copy_len = std::cmp::min(source_address.len(), 32);
    source_address_padded[32 - copy_len..].copy_from_slice(&source_address[..copy_len]);
    message.extend_from_slice(&source_address_padded);
    
    // Add destination address (padded to 32 bytes)
    let mut dest_address_padded = [0u8; 32];
    let copy_len = std::cmp::min(dest_address.as_ref().len(), 32);
    dest_address_padded[32 - copy_len..].copy_from_slice(&dest_address.as_ref()[..copy_len]);
    message.extend_from_slice(&dest_address_padded);
    
    // Hash the message using SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&message);
    let hash_result = hasher.finalize();
    
    // Convert to fixed-size array
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash_result);
    hash_array
}

#[derive(Clone, Copy)]
pub struct Signature {
    r: [u8; 32],
    s: [u8; 32],
    v: u8,
}

/// Verifies an ECDSA signature against a message using the secp256k1 library.
pub fn verify_signature(
    signature: &Signature,
    validator_pubkey: &Pubkey,
    message_hash: &[u8; 32]
) -> Result<bool> {
    // Initialize the secp256k1 context
    let secp = Secp256k1::verification_only();
    
    // Create a Message object from the message hash
    let message = Message::from_digest(*message_hash);
    
    // Construct a recoverable signature
    let recovery_id = secp256k1::ecdsa::RecoveryId::try_from(signature.v as i32 - 27)
        .map_err(|_| error!(ErrorCode::InvalidSignature))?;
    
    // Using v, r, s components to create a recoverable signature
    let recoverable_sig = secp256k1::ecdsa::RecoverableSignature::from_compact(
        &[&signature.r[..], &signature.s[..]].concat(),
        recovery_id
    ).map_err(|_| error!(ErrorCode::InvalidSignature))?;
    
    // Regular signature for verification
    let standard_sig = recoverable_sig.to_standard();
    
    // Parse the provided public key
    let pubkey = PublicKey::from_slice(validator_pubkey.as_ref())
        .map_err(|_| error!(ErrorCode::InvalidPublicKey))?;
    
    // Verify the signature
    Ok(secp.verify_ecdsa(&message, &standard_sig, &pubkey).is_ok())
}

/// Verifies multiple signatures against the same message.
/// Ensures that the required number of valid signatures from validators is met.
pub fn verify_signatures(
    signatures: &[Signature],
    validator_pubkeys: &[Pubkey],
    message_hash: &[u8; 32],
    required_signatures: u8,
) -> Result<bool> {
    // Ensure we have enough signatures
    require!(
        signatures.len() >= required_signatures.into(),
        ErrorCode::NotEnoughSignatures
    );
    
    // Ensure we have the correct number of validator public keys
    require!(
        signatures.len() == validator_pubkeys.len(),
        ErrorCode::MismatchedSignaturesAndKeys
    );
    
    // Keep track of used validator indexes to prevent duplicates
    let mut used_validators = vec![false; validator_pubkeys.len()];
    let mut valid_signatures = 0;
    
    for (i, signature) in signatures.iter().enumerate() {
        // Verify this signature
        let is_valid = verify_signature(
            signature,
            &validator_pubkeys[i],
            message_hash
        )?;
        
        // If valid, mark this validator as used and increment counter
        if is_valid {
            // Make sure this validator hasn't already been used
            require!(
                !used_validators[i],
                ErrorCode::DuplicateValidator
            );
            
            used_validators[i] = true;
            valid_signatures += 1;
        }
    }
    
    // Ensure we have enough valid signatures
    Ok(valid_signatures >= required_signatures)
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
    pub validators: [Pubkey; MAX_VALIDATORS],
    pub required_signatures: u8, // should match the initialized pubkeys in validators, used as iterator
    pub tax: u16, // basis point tax on each transaction
    pub total_volume: u64,
    pub accumulated_fees: u64,
    pub treasury: Pubkey,
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
    pub pool_state: Pubkey, // the pool state this validator was added to
    pub address: Pubkey, // the actual validator
    pub required_validators: u8, // how many validators there are now
    pub timestamp: i64,
}

#[event]
pub struct ValidatorRemoved {
    pub pool_state: Pubkey, // the pool state this validator was removed from
    pub address: Pubkey, // the actual validator that got canned
    pub required_validators: u8, // how many validators there are now
    pub timestamp: i64,
}

#[event]
pub struct PoolStateUpdated {
    pub address: Pubkey, // the pool state being updated
    pub proposed_owner: Pubkey, // the new proposed_owner. If not default, a new owner has been proposed
    pub treasury: Pubkey, // the current treasury after the update. May not have changed.
    pub tax: u16, // the tax in bps after the update. May not have changed.
    pub paused: bool, // the pause state. May not have changed.
    pub timestamp: i64,
}

#[event]
pub struct OwnerChanged {
    pub address: Pubkey, // the state account that had it's ownership changed
    pub new_owner: Pubkey, // da boss
    pub timestamp: i64,
}

#[event]
pub struct USDCDeposited {
    pub address: Pubkey, // The state account the USDC ATA is derived from,
    pub depositor: Pubkey, // the Solana account that deposited,
    pub recipient_evm_address: [u8; 20], // the EVM account getting the stuff,
    pub amount: u64, // the amount of USDC deposited,
    pub tax: u64, // the amount of USDC taxed,
    pub nonce: u64, // a unique hash for this transaction
    pub timestamp: i64
}

#[event]
pub struct USDCWithdrawn {
    pub address: Pubkey, // the state account the USDC ATA is derived from,
    pub recipient: Pubkey, // who got they money
    pub amount: u64, // how much they got paid
    pub timestamp: i64,
}

#[event]
pub struct FeesWithdrawn {
    pub address: Pubkey, // the state account that the fees were withdrawn from,
    pub recipient: Pubkey, // who got the money
    pub amount: u64, // how many monies
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

    #[msg("This validator key doesn't exist.")]
    ValidatorDoesNotExist,

    #[msg("Invalid Ownership Change, you are not the proposed owner.")]
    InvalidOwnershipChange,

    #[msg("Pool's Closed.")]
    PoolPaused,

    #[msg("USDC Only.")]
    WrongToken,

    #[msg("Tax must be at least 1 basis point.")]
    ZeroTax,

    #[msg("Tax could not be applied.")]
    TaxFailed,

    #[msg("Verification failed - failed to hash message.")]
    InvalidMessageHash,

    #[msg("Verification failed - one tested signature failed.")]
    InvalidSignature,

    #[msg("Verification failed - one tested validator pubkey failed.")]
    InvalidPublicKey,

    #[msg("Verification failed - not enough signatures present.")]
    NotEnoughSignatures,

    #[msg("Verification failed - wrong number of sigs/validators.")]
    MismatchedSignaturesAndKeys,

    #[msg("Verification failed - attempted to pass duplicate validators.")]
    DuplicateValidator,

    #[msg("Invalid validator account supplied")]
    InvalidValidatorAccount,

    #[msg("Failed to validate withdrawal")]
    FailedToValidate,
}