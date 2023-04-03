use std::collections::HashMap;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use anchor_lang::solana_program::{
    hash,
    program::{get_return_data, invoke},
};

#[derive(Debug, Clone, AnchorSerialize, AnchorDeserialize)]
pub struct IAccountMeta {
    pub pubkey: Pubkey,
    pub signer: bool,
    pub writable: bool,
}

#[derive(Debug, Clone, AnchorDeserialize, AnchorSerialize)]
pub struct PreflightAccounts {
    pub accounts: Vec<IAccountMeta>,
}

impl PreflightAccounts {
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

pub fn call_unlock() -> Result<()> {
    Ok(())
}

pub fn call_lock<'info>(ctx: CpiContext<'_, '_, '_, 'info, TILock<'info>>) -> Result<()> {
    // setup preflight
    {
        let ix_data: Vec<u8> = hash::hash("global:preflight_lock".as_bytes())
            .to_bytes()
            .to_vec();

        let ix_account_metas = ctx.accounts.to_account_metas(Some(false));
        let ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: ctx.accounts.perm_program.key(),
            accounts: ix_account_metas,
            data: ix_data,
        };

        // execute preflight
        invoke(&ix, &ctx.accounts.to_account_infos())?;
    }
    {
        // parse cpi return data
        let (program_key, program_data) = get_return_data().unwrap();
        assert_eq!(program_key, ctx.accounts.perm_program.key());
        let mut program_data = program_data.as_slice();
        let return_data: Vec<u8> = Vec::try_from_slice(&mut program_data)?;
        let additional_interface_accounts = PreflightAccounts::try_from_slice(&return_data)?;
        msg!(
            "Additional interface accounts: {:?}",
            &additional_interface_accounts
        );

        let cpi_ctx = CpiContext::new(
            ctx.accounts.perm_program.clone(),
            ILock {
                token: ctx.accounts.token.clone(),
                mint: ctx.accounts.mint.clone(),
                delegate: ctx.accounts.delegate.clone(),
                payer: ctx.accounts.payer.clone(),
                token_program: ctx.accounts.token_program.clone(),
            },
        )
        .with_remaining_accounts(ctx.remaining_accounts.to_vec());
        call_interface_function("lock".to_string(), cpi_ctx, additional_interface_accounts)?;
    }
    Ok(())
}

fn call_interface_function<'info, T: ToAccountInfos<'info> + ToAccountMetas>(
    function_name: String,
    ctx: CpiContext<'_, '_, '_, 'info, T>,
    additional_interface_accounts: PreflightAccounts,
) -> Result<()> {
    // setup
    let remaining_accounts = ctx.remaining_accounts.to_vec();

    let ix_data: Vec<u8> = hash::hash(format!("global:{}", &function_name).as_bytes())
        .to_bytes()
        .to_vec();
    let mut ix_account_metas = ctx.to_account_metas(None);
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
        program_id: ctx.program.key(),
        accounts: ix_account_metas,
        data: ix_data,
    };

    let mut ix_ais: Vec<AccountInfo> = ctx.to_account_infos();
    ix_ais.append(
        &mut additional_interface_accounts
            .match_accounts(&remaining_accounts)?
            .to_vec(),
    );

    // execute
    invoke(&ix, &ix_ais)?;
    Ok(())
}

#[derive(Accounts)]
pub struct ILock<'info> {
    #[account(mut)]
    pub token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub delegate: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct TILock<'info> {
    #[account(mut)]
    pub token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub delegate: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    /// CHECK: permission program
    pub perm_program: AccountInfo<'info>,
    // ix_accounts: Option<Account<'info, IxAccounts>>,
}

#[derive(Accounts)]
pub struct TIUnlock<'info> {
    #[account(mut)]
    pub token: InterfaceAccount<'info, TokenAccount>,
    pub mint: InterfaceAccount<'info, Mint>,
    pub delegate: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    /// CHECK: permission program
    pub perm_program: AccountInfo<'info>,
    // ix_accounts: Option<Account<'info, IxAccounts>>,
}
