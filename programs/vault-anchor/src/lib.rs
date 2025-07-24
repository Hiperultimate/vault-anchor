#![allow(unexpected_cfgs)]
#![allow(deprecated)]

use anchor_lang::{prelude::*};

declare_id!("AWBqk3mt4L33JpRWBmJ4YcV2bY7UxauTGJoAhN11AEmu");

#[error_code]
pub enum Errors {
    #[msg("Account does not have sufficient lamports")]
    InsufficientLamports,
}

fn transfer_lamports<'info>(
    system_program: AccountInfo<'info>,
    from: AccountInfo<'info>,
    to: AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let tx = CpiContext::new(
        system_program,
        anchor_lang::system_program::Transfer { from, to },
    );

    anchor_lang::system_program::transfer(tx, amount)
}

#[program]
pub mod staking_anchor {
    use anchor_lang::{
        system_program::{transfer, Transfer},
    };

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // Getting user_pda_bump from the users here isnt it risky? What if they pass an incorrect one?
        msg!("PDA Initialized {:?}", ctx.accounts.vault_state.key());

        let vault_state = &mut ctx.accounts.vault_state;
        vault_state.vault_pda_bump = ctx.bumps.vault_state;
        vault_state.vault_bump = ctx.bumps.vault;

        Ok(())
    }

    pub fn deposit(ctx: Context<Transact>, amount: u64) -> Result<()> {
        // Check if use have enough lamports

        let user_balance = ctx.accounts.signer.lamports();
        let vault_balance = ctx.accounts.vault.lamports();
        msg!("Checking user balance before {:?}", user_balance);
        msg!("Checking vault balance before {:?}", vault_balance);
        require!(user_balance >= amount, Errors::InsufficientLamports);

        transfer_lamports(
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.signer.to_account_info(),
            ctx.accounts.vault.to_account_info(),
            amount,
        )?;

        msg!("Checking user balance after {:?}", user_balance);
        msg!("Checking vault balance after {:?}", vault_balance);
        Ok(())
    }

    pub fn withdraw(ctx: Context<Transact>, amount: u64) -> Result<()> {
        // Check if user has enough funds stored in the PDA to withdraw
        let balance_on_pda = ctx.accounts.vault.get_lamports();
        require!(balance_on_pda >= amount, Errors::InsufficientLamports);

        let user_account = ctx.accounts.signer.to_account_info();
        let system_program_info = ctx.accounts.system_program.to_account_info();

        let transfer_ix = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.signer.to_account_info(),
        };
        let user_key = user_account.key();
        let signer_seeds = &[
            b"vault",
            user_key.as_ref(),
            &[ctx.accounts.vault_state.vault_bump],
        ];
        let signer = &[&signer_seeds[..]]; // what in the world is this?
        let tx = CpiContext::new_with_signer(system_program_info, transfer_ix, signer);

        transfer(tx, amount)?;

        // send lamports to user
        Ok(())
    }

    pub fn close(ctx: Context<Close>) -> Result<()> {
        // transfer all the leftover funds from the wallet
        let leftover_funds = ctx.accounts.vault.lamports();

        if leftover_funds > 0 {
            let from = ctx.accounts.vault.to_account_info();
            let to = ctx.accounts.signer.to_account_info();

            let signer_key = ctx.accounts.signer.key();
            let signer_val = &[
                b"vault",
                signer_key.as_ref(),
                &[ctx.accounts.vault_state.vault_bump],
            ];

            let signer_seeds = &[&signer_val[..]];

            let tx =CpiContext::new_with_signer(
                ctx.accounts.system_program.to_account_info(),
                Transfer {
                    from,
                    to,
                },
                signer_seeds,
            );

            transfer(tx, leftover_funds)?
        }

        // pda account is already closed thanks to anchor
        msg!("Account closed successfully...");

        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
struct UserData {
    vault_pda_bump: u8,
    vault_bump: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    signer: Signer<'info>,

    #[account(seeds=[b"vault", signer.key().as_ref()], bump)]
    vault: SystemAccount<'info>,

    #[account(init, payer=signer, space=8+UserData::INIT_SPACE, seeds=[b"state", signer.key().as_ref()], bump)]
    vault_state: Account<'info, UserData>,

    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Transact<'info> {
    #[account(mut)]
    signer: Signer<'info>,

    #[account(mut,seeds=[b"vault", signer.key().as_ref()], bump=vault_state.vault_bump)]
    vault: SystemAccount<'info>,

    #[account(seeds=[b"state", signer.key().as_ref()],bump=vault_state.vault_pda_bump)]
    vault_state: Account<'info, UserData>,

    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Close<'info> {
    #[account(mut)]
    signer: Signer<'info>,

    #[account(mut, seeds=[b"vault", signer.key().as_ref()], bump=vault_state.vault_bump)]
    vault: SystemAccount<'info>,

    #[account(mut, seeds=[b"state", signer.key().as_ref()], bump=vault_state.vault_pda_bump, close=signer)]
    vault_state: Account<'info, UserData>,

    system_program: Program<'info, System>,
}
