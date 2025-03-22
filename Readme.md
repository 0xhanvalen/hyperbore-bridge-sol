# Hyperbore Bridge - Solana Side

One day build baby lfg.
This is a classic bridge pattern - minimal user requirements, minimal validator requirements, maximum throughput.

This is a work in progress - something like this, with meaty, grindy cryptographic verification, needs a lot of tests and a lot of eyes on it. Consider this v1.

## Features

### Admin Features

```rust
pub fn initialize(ctx: Context<Initialize>, tax: u16) -> Result<()> {}
```

Sets up the main Pool state that holds the list of approved validators, the required validators (ie for m of n validation), configures which token the bridge accepts, sets the fee, and configures the treasury account that funds may be withdrawn too.

```rust
pub fn update_state(ctx: Context<UpdateStateContext>, args: ConfigUpdateArgs) -> Result<()> {}
```

Allows the current owner to change most of the Main Pool state - they can pause the contract, change the address that funds can be withdrawn to, propose a new owner, or change the fees.

```rust
pub fn accept_ownership(ctx: Context<AcceptOwnershipContext>) -> Result<()> {}
```

If the caller of this function is marked as the proposed owner in the pool state, this makes them the new owner.

```rust
pub fn add_validator(ctx: Context<AddValidatorContext>) -> Result<()> {}
```

Adds a new validator to the next available index. While we're on validators, there's at most 16 of them at any given time.
This is set by a constant in this program, MAX_VALIDATORS. If you need more validators, you must relaunch. This is set to a constant due to the nature of Solana Accounts, which must be a fixed size in order to calculate rents.

```rust
pub fn remove_validator(ctx: Context<RemoveValidatorContext>) -> Result<()> {}
```

Removes a validator. Shifts all validators down one index, setting the final index of validators to be the default Pubkey.

### Token Features

```rust
    pub fn deposit_usdc(
            ctx: Context<DepositUSDCContext>,
            args: DepositUSDCArgs,
        ) -> Result<()> {}
```

The main event - allows a user to deposit their USDC on Solana, to brige over to the HyperEVM. Emits an event that gets validated by the validators.

```rust
    pub fn withdraw_usdc(
        ctx: Context<WithdrawUSDCContext>,
        recipient: Pubkey,
        args: WithdrawUSDCArgs,
    ) -> Result<()> {}
```

Call this to recover your funds. You have to supply your own signatures, though, so probably use a structured front end that keeps track of the validation signatures for calling this.

```rust
pub fn withdraw_fees(ctx: Context<WithdrawFeesContext>) -> Result<()> {}
```

The owner of the Pool state can call this to withdraw the accumulated fees, tracked in the Pool state. Resets accumulated fees to zero.

## License

This project is licensed under the MIT License. Please remix it for your own needs and make beautiful, co-operative things.

## Sponsorship

This project is sponsored by [HyperBoreDAO](https://www.hyperboredao.ai/)
