use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use interface::{call_lock, PreflightAccounts, TILock as _TILock};
declare_id!("6Dmq9ijrYZio9ny6PezemaWe3kcs7qbJ8sB78LHgQDeY");

#[program]
pub mod caller {
    use anchor_lang::solana_program::{
        hash,
        program::{get_return_data, invoke},
    };

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
        call_lock(cvt_ctx)?;
        Ok(())
    }

    pub fn unlock<'info>(ctx: Context<'_, '_, '_, 'info, TIUnlock<'info>>) -> Result<()> {
        let token = &ctx.accounts.token;
        let mint = &ctx.accounts.mint;
        let delegate = &ctx.accounts.delegate;
        let token_program = &ctx.accounts.token_program;

        // setup preflight
        {
            let ix_data: Vec<u8> = hash::hash("global:preflight_unlock".as_bytes())
                .to_bytes()
                .to_vec();
            let ix_account_metas = vec![
                AccountMeta::new_readonly(ctx.accounts.token.key(), false),
                AccountMeta::new_readonly(ctx.accounts.mint.key(), false),
                AccountMeta::new_readonly(ctx.accounts.delegate.key(), false),
                AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
            ];
            let ix = anchor_lang::solana_program::instruction::Instruction {
                program_id: ctx.accounts.perm_program.key(),
                accounts: ix_account_metas,
                data: ix_data,
            };

            // execute preflight
            invoke(
                &ix,
                &[
                    token.to_account_info(),
                    mint.to_account_info(),
                    delegate.to_account_info(),
                    token_program.to_account_info(),
                ],
            )?;
        }
        {
            // parse cpi return data
            let (program_key, program_data) = get_return_data().unwrap();
            assert_eq!(program_key, ctx.accounts.perm_program.key());
            let mut program_data = program_data.as_slice();
            let additional_interface_accounts = PreflightAccounts::deserialize(&mut program_data)?;
            msg!(
                "Additional interface accounts: {:?}",
                &additional_interface_accounts
            );

            // setup lock
            let remaining_accounts = ctx.remaining_accounts.to_vec();

            let ix_data: Vec<u8> = hash::hash("global:unlock".as_bytes()).to_bytes().to_vec();
            let mut ix_account_metas = vec![
                AccountMeta::new(token.key(), false),
                AccountMeta::new_readonly(mint.key(), false),
                AccountMeta::new(delegate.key(), true),
                AccountMeta::new_readonly(token_program.key(), false),
            ];
            ix_account_metas.append(
                additional_interface_accounts
                    .accounts
                    .iter()
                    .map(|acc| {
                        if acc.writable {
                            AccountMeta::new(acc.pubkey, acc.signer)
                        } else {
                            AccountMeta::new_readonly(acc.pubkey, acc.signer)
                        }
                    })
                    .collect::<Vec<AccountMeta>>()
                    .as_mut(),
            );
            let ix = anchor_lang::solana_program::instruction::Instruction {
                program_id: ctx.accounts.perm_program.key(),
                accounts: ix_account_metas,
                data: ix_data,
            };

            let mut ix_ais: Vec<AccountInfo> = vec![
                token.to_account_info(),
                mint.to_account_info(),
                delegate.to_account_info(),
                token_program.to_account_info(),
            ];
            ix_ais.append(
                &mut additional_interface_accounts
                    .match_accounts(&remaining_accounts)?
                    .to_vec(),
            );

            // execute lock
            invoke(&ix, &ix_ais)?;
        }

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
