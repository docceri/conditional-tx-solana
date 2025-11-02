//! Conditional SOL Transfer (≥ 0.1 SOL)
//! -----------------------------------
//! This Anchor program transfers SOL from a configured `from` (A) to `to` (B)
//! **if and only if** the requested amount is **at least** a configured threshold.
//!
//! Key points:
//! - Programs execute only when invoked in a transaction; they do not auto-run on
//!   generic wallet transfers.
//! - Threshold is stored in lamports (1 SOL = 1_000_000_000 lamports).
//! - The `from` account (A) must sign calls to send funds.
//!
//! Quick usage (devnet):
//! 1) `anchor keys list` → copy your program id.
//! 2) Put it into `declare_id!(...)` below AND into `Anchor.toml` under [programs.devnet].
//! 3) `anchor build && anchor deploy`
//! 4) Initialize with TS script: `npx ts-node scripts/init.ts B_PUBKEY [thresholdLamports]`
//! 5) Send with TS script: `npx ts-node scripts/send.ts 0.25`
//!
//! Safety:
//! - On-chain check uses **amount ≥ threshold**.
//! - Authority may update threshold or addresses (consider multisig in production).

use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

// Paste your deployed program ID here and in Anchor.toml ([programs.devnet])
declare_id!("REPLACE_WITH_YOUR_PROGRAM_ID");

const CONFIG_SEED: &[u8] = b"config";

#[program]
pub mod conditional_transfer {
    use super::*;

    /// Initialize the config PDA with: authority, from, to, and threshold.
    /// - `authority`: allowed to update config
    /// - `from`: the only signer permitted to send funds
    /// - `to`: recipient
    /// - `threshold_lamports`: minimal amount (lamports) required to allow transfer
    pub fn initialize(
        ctx: Context<Initialize>,
        from: Pubkey,
        to: Pubkey,
        threshold_lamports: u64,
    ) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        cfg.authority = ctx.accounts.authority.key();
        cfg.from = from;
        cfg.to = to;
        cfg.threshold_lamports = threshold_lamports;
        cfg.bump = *ctx.bumps.get("config").unwrap();
        Ok(())
    }

    /// Transfer lamports from `from` (must sign) to `to` if `lamports ≥ threshold`.
    /// Uses a CPI to the System Program.
    pub fn send_if_over_threshold(ctx: Context<SendIfOverThreshold>, lamports: u64) -> Result<()> {
        let cfg = &ctx.accounts.config;
        // NOTE: Behavior is "≥ threshold" (at least). Adjust here if you want different rules.
        require!(
            lamports >= cfg.threshold_lamports,
            ConditionalError::BelowThreshold
        );

        // CPI to transfer SOL from `from` -> `to`
        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.from.to_account_info(),
                to: ctx.accounts.to.to_account_info(),
            },
        );
        system_program::transfer(cpi_ctx, lamports)?;
        Ok(())
    }

    /// Optional: Update threshold (authority only).
    pub fn update_threshold(ctx: Context<Update>, new_threshold_lamports: u64) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        require_keys_eq!(cfg.authority, ctx.accounts.authority.key(), ConditionalError::Unauthorized);
        cfg.threshold_lamports = new_threshold_lamports;
        Ok(())
    }

    /// Optional: Update from/to addresses (authority only).
    pub fn update_addresses(ctx: Context<Update>, new_from: Pubkey, new_to: Pubkey) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        require_keys_eq!(cfg.authority, ctx.accounts.authority.key(), ConditionalError::Unauthorized);
        cfg.from = new_from;
        cfg.to = new_to;
        Ok(())
    }
}

/// Accounts context for initialization. Creates the config PDA.
#[derive(Accounts)]
pub struct Initialize<'info> {
    /// Authority who can update the config parameters.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Program Derived Address storing config (created & paid by authority).
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 8 + 1, // discriminator + authority + from + to + threshold + bump
        seeds = [CONFIG_SEED],
        bump
    )]
    pub config: Account<'info, Config>,

    /// Built-in System Program is needed for CPI transfer & allocation.
    pub system_program: Program<'info, System>,
}

/// Accounts context for transfer call.
#[derive(Accounts)]
pub struct SendIfOverThreshold<'info> {
    /// The config PDA.
    #[account(
        seeds = [CONFIG_SEED],
        bump = config.bump
    )]
    pub config: Account<'info, Config>,

    /// The `from` account must match config.from AND must sign the transaction.
    #[account(mut, address = config.from)]
    pub from: Signer<'info>,

    /// The `to` account must match config.to and will receive lamports.
    #[account(mut, address = config.to)]
    pub to: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// Accounts context for updates (authority-only).
#[derive(Accounts)]
pub struct Update<'info> {
    /// Authority as set in config; must sign.
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Config PDA with authority constraint enforced.
    #[account(
        mut,
        seeds = [CONFIG_SEED],
        bump = config.bump,
        has_one = authority
    )]
    pub config: Account<'info, Config>,
}

/// On-chain config for the program.
#[account]
pub struct Config {
    pub authority: Pubkey,
    pub from: Pubkey,
    pub to: Pubkey,
    pub threshold_lamports: u64,
    pub bump: u8,
}

/// Error types for the program.
#[error_code]
pub enum ConditionalError {
    /// Amount provided was below the configured threshold.
    #[msg("Amount must be at least the configured threshold.")]
    BelowThreshold,
    /// Caller attempted an unauthorized update.
    #[msg("Only the authority may update config.")]
    Unauthorized,
}
