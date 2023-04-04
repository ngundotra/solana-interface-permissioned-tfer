use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked,
};

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

    pub fn tfer<'info>(
        ctx: Context<'_, '_, '_, 'info, Transfer<'info>>,
        amount: u64,
    ) -> Result<()> {
        let cvt_ctx = CpiContext::new(
            ctx.accounts.perm_program.clone(),
            _TIUnlock {
                token: ctx.accounts.source.clone(),
                mint: ctx.accounts.mint.clone(),
                delegate: ctx.accounts.delegate.clone(),
                token_program: ctx.accounts.token_program.clone(),
                perm_program: ctx.accounts.perm_program.clone(),
            },
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());
        call("unlock".to_string(), cvt_ctx, false)?;

        transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.source.to_account_info(),
                    to: ctx.accounts.dest.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    authority: ctx.accounts.delegate.to_account_info(),
                },
            ),
            amount,
            ctx.accounts.mint.decimals,
        )?;

        let cvt_ctx = CpiContext::new(
            ctx.accounts.perm_program.clone(),
            _TILock {
                token: ctx.accounts.dest.clone(),
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
}

#[derive(Accounts)]
pub struct TIUnlock<'info> {
    #[account(mut)]
    token: InterfaceAccount<'info, TokenAccount>,
    mint: InterfaceAccount<'info, Mint>,
    delegate: Signer<'info>,
    token_program: Interface<'info, TokenInterface>,
    /// CHECK: permission program
    perm_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Transfer<'info> {
    #[account(mut)]
    source: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    dest: InterfaceAccount<'info, TokenAccount>,
    mint: InterfaceAccount<'info, Mint>,
    delegate: Signer<'info>,
    #[account(mut)]
    payer: Signer<'info>,
    token_program: Interface<'info, TokenInterface>,
    /// CHECK: fuck
    #[account(seeds=["hello".as_ref()], bump)]
    penguin: AccountInfo<'info>,
    /// CHECK: permission program
    perm_program: AccountInfo<'info>,
}
