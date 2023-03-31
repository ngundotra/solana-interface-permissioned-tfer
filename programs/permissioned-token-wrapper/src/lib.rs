use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface, TokenAccount, self};

use interface::{IAccountMeta, PreflightAccounts};

declare_id!("7vnNq5wAJPAoocKqwRWv6dUoZBGrZDCS3ULspFXGdGVx");

#[program]
pub mod permissioned_token_wrapper {
    use super::*;

    pub mod admin {
        use super::*;
        pub fn set_ix_accounts<'info>(
            ctx: Context<SetIxAccounts>,
            accounts: Vec<Pubkey>
        ) -> Result<()> {
            // let mut ix_accounts = vec![];
            // for account in accounts {
            //     ix_accounts.push(AccountMeta {
            //         pubkey: *account.key,
            //         is_signer: account.is_signer,
            //         is_writable: account.is_writable,
            //     });
            // }
            // ix.accounts = ix_accounts;
            Ok(())
        }
    }

    pub fn preflight_lock(ctx: Context<ILock>) -> Result<Vec<u8>> {
        let token = ctx.accounts.token.key();
        let (program_control, _) = Pubkey::find_program_address(&[STATIC_PREFIX.as_bytes()], &crate::id());
        let (token_record, _) = Pubkey::find_program_address(&[token.key().as_ref(), TOKEN_RECORD_PREFIX.as_bytes()], &crate::id());
        let system_program = System::id();
        Ok(PreflightAccounts { accounts: vec![
            IAccountMeta { pubkey: program_control, signer: false, writable: false }, 
            IAccountMeta { pubkey: token_record, signer: false, writable: true }, 
            IAccountMeta { pubkey: system_program, signer: false, writable: false }
        ] }.try_to_vec()?)
    }

    pub fn preflight_unlock(ctx: Context<IUnlock>) -> Result<Vec<u8>> {
        let token = ctx.accounts.token.key();
        let (program_control, _) = Pubkey::find_program_address(&[STATIC_PREFIX.as_bytes()], &crate::id());
        let (token_record, _) = Pubkey::find_program_address(&[token.key().as_ref(), TOKEN_RECORD_PREFIX.as_bytes()], &crate::id());
        let system_program = System::id();
        Ok(PreflightAccounts { accounts: vec![
            IAccountMeta { pubkey: program_control, signer: false, writable: false }, 
            IAccountMeta { pubkey: token_record, signer: false, writable: true }, 
            IAccountMeta { pubkey: system_program, signer: false, writable: false }
        ] }.try_to_vec()?)
    }

    pub fn lock(ctx: Context<Lock>) -> Result<()> {
        ctx.accounts.token_record.locked = 1;

        let static_seeds: &[u8] = &STATIC_PREFIX.as_bytes();
        let bump_seed = ctx.bumps.get("program_control").unwrap();
        let seeds = &[&static_seeds[..], &[*bump_seed]];
        let binding = [&seeds[..]];
        let ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::FreezeAccount {
                account: ctx.accounts.token.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.program_control.to_account_info(),
            },
            &binding,
        );
        token_interface::freeze_account(ctx)?;
        Ok(())
    }

    pub fn unlock(ctx: Context<Unlock>) -> Result<()> {
        ctx.accounts.token_record.locked = 0;

        let static_seeds: &[u8] = &STATIC_PREFIX.as_bytes();
        let bump_seed = ctx.bumps.get("program_control").unwrap();
        let seeds = &[&static_seeds[..], &[*bump_seed]];
        let binding = [&seeds[..]];
        let ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token_interface::ThawAccount {
                account: ctx.accounts.token.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.program_control.to_account_info(),
            },
            &binding,
        );
        token_interface::thaw_account(ctx)?;
        Ok(())
    }
}

pub const STATIC_PREFIX: &'static str = "static";
pub const TOKEN_RECORD_PREFIX: &'static str = "token_record";
#[account]
pub struct TokenRecord {
    pub locked: u8,
}

#[derive(Accounts)]
pub struct SetIxAccounts {
}

#[derive(Accounts)]
pub struct ILock<'info> {
    token: InterfaceAccount<'info, TokenAccount>,
    mint: InterfaceAccount<'info, Mint>,
    /// CHECK: nil
    delegate: AccountInfo<'info>,
    /// CHECK: nil
    payer: AccountInfo<'info>,
    token_program: Interface<'info, TokenInterface>,
    // #[account(seeds=["lock".as_bytes()], bump)]
    // ix_accounts: Option<Account<'info, IxAccounts>>
}

#[derive(Accounts)]
pub struct Lock<'info> {
    #[account(mut)]
    token: InterfaceAccount<'info, TokenAccount>,
    mint: InterfaceAccount<'info, Mint>,
    #[account(constraint = token.owner == delegate.key() || token.delegate.is_some() && token.delegate.unwrap() == delegate.key())]
    delegate: Signer<'info>,
    #[account(mut)]
    payer: Signer<'info>,
    token_program: Interface<'info, TokenInterface>,
    /// CHECK: nothing
    #[account(
        seeds=[STATIC_PREFIX.as_bytes()], 
        bump,
        constraint = mint.freeze_authority.is_some() && mint.freeze_authority.unwrap() == program_control.key()
    )]
    program_control: AccountInfo<'info>,
    #[account(init_if_needed, payer=payer, space=8 + std::mem::size_of::<TokenRecord>(), seeds=[token.key().as_ref(), TOKEN_RECORD_PREFIX.as_bytes()], bump)]
    token_record: Account<'info, TokenRecord>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct IUnlock<'info> {
    token: InterfaceAccount<'info, TokenAccount>,
    mint: InterfaceAccount<'info, Mint>,
    /// CHECK: nil
    owner: AccountInfo<'info>,
    token_program: Interface<'info, TokenInterface>,
    // #[account(seeds=["unlock".as_bytes()], bump)]
    // ix_accounts: Option<Account<'info, IxAccounts>>
}

#[derive(Accounts)]
pub struct Unlock<'info> {
    #[account(mut)]
    token: InterfaceAccount<'info, TokenAccount>,
    mint: InterfaceAccount<'info, Mint>,
    #[account(constraint = token.owner == delegate.key() || token.delegate.is_some() && token.delegate.unwrap() == delegate.key())]
    delegate: Signer<'info>,
    token_program: Interface<'info, TokenInterface>,
    /// CHECK: nothing
    #[account(
        seeds=[STATIC_PREFIX.as_bytes()], 
        bump,
        constraint = mint.freeze_authority.is_some() && mint.freeze_authority.unwrap() == program_control.key()
    )]
    program_control: AccountInfo<'info>,
    #[account(mut, seeds=[token.key().as_ref(), b"token_record"], bump)]
    token_record: Account<'info, TokenRecord>,
    system_program: Program<'info, System>,
}