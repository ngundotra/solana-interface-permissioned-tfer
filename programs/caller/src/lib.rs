use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use interface::{call, TILock as _TILock, TIUnlock as _TIUnlock};
declare_id!("6Dmq9ijrYZio9ny6PezemaWe3kcs7qbJ8sB78LHgQDeY");

#[program]
pub mod caller {
    use super::*;

    pub fn lock<'info>(ctx: Context<'_, '_, '_, 'info, TILock<'info>>) -> Result<()> {
        let cvt_ctx = CpiContext::new(
            ctx.accounts.perm_program.clone(),
            _TILock {
                token: ctx.accounts.token.clone(),
                mint: ctx.accounts.mint.clone(),
                delegate: ctx.accounts.delegate.clone(),
                payer: ctx.accounts.payer.clone(),
                token_program: ctx.accounts.token_program.clone(),
                perm_program: ctx.accounts.perm_program.clone(),
            },
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());

        call("lock".to_string(), cvt_ctx, false)?;
        Ok(())
    }

    pub fn unlock<'info>(ctx: Context<'_, '_, '_, 'info, TIUnlock<'info>>) -> Result<()> {
        let cvt_ctx = CpiContext::new(
            ctx.accounts.perm_program.clone(),
            _TIUnlock {
                token: ctx.accounts.token.clone(),
                mint: ctx.accounts.mint.clone(),
                delegate: ctx.accounts.delegate.clone(),
                token_program: ctx.accounts.token_program.clone(),
                perm_program: ctx.accounts.perm_program.clone(),
            },
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());

        call("unlock".to_string(), cvt_ctx, false)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct TILock<'info> {
    #[account(mut)]
    token: InterfaceAccount<'info, TokenAccount>,
    mint: InterfaceAccount<'info, Mint>,
    delegate: Signer<'info>,
    #[account(mut)]
    payer: Signer<'info>,
    token_program: Interface<'info, TokenInterface>,
    /// CHECK: permission program
    perm_program: AccountInfo<'info>,
    // ix_accounts: Option<Account<'info, IxAccounts>>,
}

// impl<'info> Into<'static, _TILock<'_>> for TIUnlock<'info> {
//     fn into(self) -> _TILock<'info> {}
// }
// impl<'info> Into<&mut Context<'_, '_, '_, 'info, _TILock<'info>>>
//     for Context<'_, '_, '_, 'info, TILock<'info>>
// {
// fn convert_lock_ctx<'info>(
//     ctx: &Context<'_, '_, '_, 'info, TILock<'info>>,
// ) -> Context<'info, 'info, 'info, 'info, _TILock<'info>> {
//     let mut cpi_accounts = _TILock {
//         token: ctx.accounts.token.clone(),
//         mint: ctx.accounts.mint.clone(),
//         delegate: ctx.accounts.delegate.clone(),
//         payer: ctx.accounts.payer.clone(),
//         token_program: ctx.accounts.token_program.clone(),
//         perm_program: ctx.accounts.perm_program.clone(),
//     };

//     Context::<_TILock>::new(
//         &ctx.accounts.perm_program.key(),
//         &mut cpi_accounts,
//         ctx.remaining_accounts,
//         ctx.bumps,
//     )
// }
// fn into(accounts: &mut InitializeEscrow<'info>) -> Self {
//     let cpi_accounts = SetAuthority {
//         account_or_mint: accounts
//             .initializer_deposit_token_account
//             .to_account_info()
//             .clone(),
//         current_authority: accounts.initializer.to_account_info().clone(),
//     };
//     let cpi_program = accounts.token_program.to_account_info();
//     CpiContext::new(cpi_program, cpi_accounts)
//     // }
// }

#[derive(Accounts)]
pub struct TIUnlock<'info> {
    #[account(mut)]
    token: InterfaceAccount<'info, TokenAccount>,
    mint: InterfaceAccount<'info, Mint>,
    delegate: Signer<'info>,
    token_program: Interface<'info, TokenInterface>,
    /// CHECK: permission program
    perm_program: AccountInfo<'info>,
    // ix_accounts: Option<Account<'info, IxAccounts>>,
}
