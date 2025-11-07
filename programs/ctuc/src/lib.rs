use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, SetAuthority};
use spl_token::instruction::AuthorityType;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod owner_mint_token {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, owner: Pubkey) -> Result<()> {
        let cfg = &mut ctx.accounts.config;
        cfg.owner = owner;
        cfg.mint = ctx.accounts.mint.key();
        cfg.bump = ctx.bumps.mint_authority_pda;

        // Move mint authority from current signer to the PDA
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = token::SetAuthority {
            current_authority: ctx.accounts.current_mint_authority.to_account_info(),
            account_or_mint: ctx.accounts.mint.to_account_info(),
        };
        token::set_authority(
            CpiContext::new(cpi_program, cpi_accounts),
            AuthorityType::MintTokens,
            Some(ctx.accounts.mint_authority_pda.key()),
        )?;
        Ok(())
    }

    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.owner.key(),
            ctx.accounts.config.owner,
            CustomError::Unauthorized
        );

        let mint_key = ctx.accounts.mint.key();
        let seeds = &[
            b"authority",
            mint_key.as_ref(),
            &[ctx.accounts.config.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.destination.to_account_info(),
            authority: ctx.accounts.mint_authority_pda.to_account_info(),
        };
        token::mint_to(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, signer),
            amount,
        )?;
        Ok(())
    }

    pub fn disable_minting(ctx: Context<DisableMinting>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.owner.key(),
            ctx.accounts.config.owner,
            CustomError::Unauthorized
        );

        let mint_key = ctx.accounts.mint.key();
        let seeds = &[
            b"authority",
            mint_key.as_ref(),
            &[ctx.accounts.config.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_accounts = SetAuthority {
            current_authority: ctx.accounts.mint_authority_pda.to_account_info(),
            account_or_mint: ctx.accounts.mint.to_account_info(),
        };
        token::set_authority(
            CpiContext::new_with_signer(cpi_program, cpi_accounts, signer),
            AuthorityType::MintTokens,
            None,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(owner: Pubkey)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(seeds = [b"authority", mint.key().as_ref()], bump)]
    pub mint_authority_pda: UncheckedAccount<'info>,
    pub current_mint_authority: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + 32 + 32 + 1,
        seeds = [b"config", mint.key().as_ref()],
        bump
    )]
    pub config: Account<'info, MintConfig>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    pub owner: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(seeds = [b"authority", mint.key().as_ref()], bump = config.bump)]
    pub mint_authority_pda: UncheckedAccount<'info>,
    #[account(mut)]
    pub destination: Account<'info, TokenAccount>,
    #[account(seeds = [b"config", mint.key().as_ref()], bump)]
    pub config: Account<'info, MintConfig>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DisableMinting<'info> {
    pub owner: Signer<'info>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    #[account(seeds = [b"authority", mint.key().as_ref()], bump = config.bump)]
    pub mint_authority_pda: UncheckedAccount<'info>,
    #[account(seeds = [b"config", mint.key().as_ref()], bump)]
    pub config: Account<'info, MintConfig>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct MintConfig {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub bump: u8,
}

#[error_code]
pub enum CustomError {
    #[msg("Unauthorized: only the configured owner may perform this action.")]
    Unauthorized,
}