#![feature(generic_associated_types)]
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
        msg!("found accounts: {:?}", found_accounts.len());

        Ok(found_accounts)
    }
}

// pub fn call_unlock() -> Result<()> {
//     Ok(())
// }

pub fn get_interface_accounts(program_key: &Pubkey) -> Result<PreflightAccounts> {
    let (key, program_data) = get_return_data().unwrap();
    assert_eq!(key, *program_key);
    let mut program_data = program_data.as_slice();
    let return_data: Vec<u8> = Vec::try_from_slice(&mut program_data)?;
    let additional_interface_accounts = PreflightAccounts::try_from_slice(&return_data)?;
    msg!(
        "Additional interface accounts: {:?}",
        &additional_interface_accounts
    );
    Ok(additional_interface_accounts)
}

pub fn call<
    'info,
    C1: ToAccountInfos<'info> + ToAccountMetas + ToTargetProgram<'info, TargetCtx<'info> = C2>,
    C2: ToAccountInfos<'info> + ToAccountMetas,
>(
    ix_name: String,
    ctx: CpiContext<'_, '_, '_, 'info, C1>,
    log_info: bool,
) -> Result<()> {
    msg!("Preflight");
    // preflight
    call_preflight_interface_function(ix_name.clone(), &ctx)?;

    msg!("Parse return data");
    // parse cpi return data
    let additional_interface_accounts = get_interface_accounts(&ctx.accounts.to_target_program())?;

    // execute
    msg!("Convert into target context");
    let cpi_ctx: CpiContext<C2> = ctx
        .accounts
        .to_target_context(ctx.remaining_accounts.to_vec());
    msg!("Execute {}", &ix_name);
    call_interface_function(
        ix_name.clone(),
        cpi_ctx,
        additional_interface_accounts,
        log_info,
    )?;
    Ok(())
}

// pub fn call_lock<'info>(ctx: CpiContext<'_, '_, '_, 'info, TILock<'info>>) -> Result<()> {
//     let ix_name = "lock".to_string();
//     // preflight
//     call_preflight_interface_function(ix_name.clone(), &ctx)?;
//     // parse cpi return data
//     let additional_interface_accounts = get_interface_accounts(&ctx.accounts.perm_program.key())?;

//     let cpi_ctx = CpiContext::new(
//         ctx.accounts.perm_program.clone(),
//         ILock {
//             token: ctx.accounts.token.clone(),
//             mint: ctx.accounts.mint.clone(),
//             delegate: ctx.accounts.delegate.clone(),
//             payer: ctx.accounts.payer.clone(),
//             token_program: ctx.accounts.token_program.clone(),
//         },
//     )
//     .with_remaining_accounts(ctx.remaining_accounts.to_vec());
//     call_interface_function(ix_name.clone(), cpi_ctx, additional_interface_accounts)?;
//     Ok(())
// }

fn call_preflight_interface_function<'info, T: ToAccountInfos<'info> + ToAccountMetas>(
    function_name: String,
    ctx: &CpiContext<'_, '_, '_, 'info, T>,
) -> Result<()> {
    // setup
    let ix_data: Vec<u8> = hash::hash(format!("global:preflight_{}", &function_name).as_bytes())
        .to_bytes()
        .to_vec();
    let ix_account_metas = ctx.accounts.to_account_metas(Some(false));
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ctx.program.key(),
        accounts: ix_account_metas,
        data: ix_data,
    };

    // execute
    invoke(&ix, &ctx.accounts.to_account_infos())?;
    Ok(())
}

fn call_interface_function<'info, T: ToAccountInfos<'info> + ToAccountMetas>(
    function_name: String,
    ctx: CpiContext<'_, '_, '_, 'info, T>,
    additional_interface_accounts: PreflightAccounts,
    log_info: bool,
) -> Result<()> {
    // setup
    let remaining_accounts = ctx.remaining_accounts.to_vec();

    let ix_data: Vec<u8> = hash::hash(format!("global:{}", &function_name).as_bytes())
        .to_bytes()
        .to_vec();
    let mut ix_account_metas = ctx.accounts.to_account_metas(None);
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

    let mut ix_ais: Vec<AccountInfo> = ctx.accounts.to_account_infos();
    msg!("IX accounts: {:?}", &ix_ais.len());
    ix_ais.extend_from_slice(
        &mut additional_interface_accounts
            .match_accounts(&remaining_accounts)?
            .to_vec(),
    );
    msg!("IX accounts: {:?}", &ix_ais.len());

    if log_info {
        ix_ais.iter().into_iter().for_each(|ai| {
            msg!(
                "Account: {:?}, {:?}, {:?}, {:?}",
                ai.key,
                ai.owner,
                ai.is_signer,
                ai.is_writable
            )
        });
    } else {
        // execute
        invoke(&ix, &ix_ais)?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ILock<'info> {
    #[account(mut)]
    pub token: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub delegate: AccountInfo<'info>,
    #[account(mut)]
    pub payer: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
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

pub trait ToTargetProgram<'info> {
    type TargetCtx<'_info>: ToAccountInfos<'_info> + ToAccountMetas;

    fn to_target_program(&self) -> Pubkey;
    fn get_target_program(&self) -> AccountInfo<'info>;
    fn to_target_context(
        &self,
        remaining_accounts: Vec<AccountInfo<'info>>,
    ) -> CpiContext<'_, '_, '_, 'info, Self::TargetCtx<'info>>;
}

impl<'info> ToTargetProgram<'info> for TILock<'info> {
    type TargetCtx<'a> = ILock<'a>;

    fn to_target_program(&self) -> Pubkey {
        self.perm_program.key()
    }
    fn get_target_program(&self) -> AccountInfo<'info> {
        self.perm_program.clone()
    }

    fn to_target_context(
        &self,
        remaining_accounts: Vec<AccountInfo<'info>>,
    ) -> CpiContext<'_, '_, '_, 'info, Self::TargetCtx<'info>> {
        let inner = ILock {
            token: self.token.to_account_info(),
            mint: self.mint.to_account_info(),
            delegate: self.delegate.to_account_info(),
            payer: self.payer.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };
        CpiContext::new(self.get_target_program(), inner)
            .with_remaining_accounts(remaining_accounts)
    }
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

#[derive(Accounts)]
pub struct IUnlock<'info> {
    #[account(mut)]
    pub token: AccountInfo<'info>,
    pub mint: AccountInfo<'info>,
    pub delegate: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
}

impl<'info> ToTargetProgram<'info> for TIUnlock<'info> {
    type TargetCtx<'a> = IUnlock<'a>;

    fn to_target_program(&self) -> Pubkey {
        self.perm_program.key()
    }
    fn get_target_program(&self) -> AccountInfo<'info> {
        self.perm_program.clone()
    }

    fn to_target_context(
        &self,
        remaining_accounts: Vec<AccountInfo<'info>>,
    ) -> CpiContext<'_, '_, '_, 'info, Self::TargetCtx<'info>> {
        let inner = IUnlock {
            token: self.token.to_account_info(),
            mint: self.mint.to_account_info(),
            delegate: self.delegate.to_account_info(),
            token_program: self.token_program.to_account_info(),
        };
        CpiContext::new(self.get_target_program(), inner)
            .with_remaining_accounts(remaining_accounts)
    }
}
