use std::collections::HashMap;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

declare_id!("6Dmq9ijrYZio9ny6PezemaWe3kcs7qbJ8sB78LHgQDeY");

#[program]
pub mod caller {
    use anchor_lang::solana_program::{
        hash,
        program::{get_return_data, invoke},
    };

    use super::*;

    pub fn lock<'info>(ctx: Context<'_, '_, '_, 'info, TILock<'info>>) -> Result<()> {
        let token = &ctx.accounts.token;
        let mint = &ctx.accounts.mint;
        let delegate = &ctx.accounts.delegate;
        let payer = &ctx.accounts.payer;
        let token_program = &ctx.accounts.token_program;

        // setup preflight
        {
            let ix_data: Vec<u8> = hash::hash("global:preflight_lock".as_bytes())
                .to_bytes()
                .to_vec();
            let ix_account_metas = vec![
                AccountMeta::new_readonly(ctx.accounts.token.key(), false),
                AccountMeta::new_readonly(ctx.accounts.mint.key(), false),
                AccountMeta::new_readonly(ctx.accounts.delegate.key(), false),
                AccountMeta::new_readonly(ctx.accounts.payer.key(), false),
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
                    payer.to_account_info(),
                    token_program.to_account_info(),
                ],
            )?;
        }
        {
            // parse cpi return data
            let (program_key, program_data) = get_return_data().unwrap();
            assert_eq!(program_key, ctx.accounts.perm_program.key());
            let mut program_data = program_data.as_slice();
            let additional_interface_accounts = IxAccounts::deserialize(&mut program_data)?;
            msg!(
                "Additional interface accounts: {:?}",
                &additional_interface_accounts
            );

            // setup lock
            let remaining_accounts = ctx.remaining_accounts.to_vec();

            let ix_data: Vec<u8> = hash::hash("global:lock".as_bytes()).to_bytes().to_vec();
            let mut ix_account_metas = vec![
                AccountMeta::new(token.key(), false),
                AccountMeta::new_readonly(mint.key(), false),
                AccountMeta::new(delegate.key(), true),
                AccountMeta::new(payer.key(), true),
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
                payer.to_account_info(),
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

    pub fn unlock<'info>(ctx: Context<'_, '_, '_, 'info, TIUnlock<'info>>) -> Result<()> {
        let token = &ctx.accounts.token;
        let mint = &ctx.accounts.mint;
        let delegate = &ctx.accounts.delegate;
        let token_program = &ctx.accounts.token_program;

        // setup preflight
        {
            let ix_data: Vec<u8> = hash::hash("global:preflight_lock".as_bytes())
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
            let additional_interface_accounts = IxAccounts::deserialize(&mut program_data)?;
            msg!(
                "Additional interface accounts: {:?}",
                &additional_interface_accounts
            );

            // setup lock
            let remaining_accounts = ctx.remaining_accounts.to_vec();

            let ix_data: Vec<u8> = hash::hash("global:lock".as_bytes()).to_bytes().to_vec();
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

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct IAccountMeta {
    pub pubkey: Pubkey,
    pub signer: bool,
    pub writable: bool,
}

#[derive(Debug)]
#[account]
pub struct IxAccounts {
    accounts: Vec<IAccountMeta>,
}

impl IxAccounts {
    pub fn match_accounts<'info>(
        &self,
        accounts: &[AccountInfo<'info>],
    ) -> Result<Vec<AccountInfo<'info>>> {
        let mut map = HashMap::<Pubkey, AccountInfo>::new();

        for acc in accounts {
            map.insert(acc.key(), acc.clone());
        }

        let mut found_accounts = Vec::<AccountInfo>::new();
        for acc in self.accounts.iter() {
            let found_acc = map.get(&acc.pubkey);
            if found_acc.is_none() {
                msg!(&format!("account not found: {:?}", acc.pubkey));
                return Err(ProgramError::NotEnoughAccountKeys.into());
            }
            found_accounts.push(found_acc.unwrap().clone());
        }

        Ok(found_accounts)
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

#[derive(Accounts)]
pub struct Initialize {}
