#![allow(unexpected_cfgs)]
#![allow(deprecated)]
use std::io::Empty;

use anchor_lang::prelude::*;

declare_id!("AWBqk3mt4L33JpRWBmJ4YcV2bY7UxauTGJoAhN11AEmu");

#[error_code]
pub enum Errors {
    #[msg("Account does not have sufficient lamports")]
    InsufficientLamports,
}

#[program]
pub mod staking_anchor {
    use anchor_lang::{accounts::signer, system_program::{transfer, Transfer}};

    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        // Getting user_pda_bump from the users here isnt it risky? What if they pass an incorrect one?
        msg!("PDA Initialized {:?}", ctx.accounts.user_pda.key());
        // Calculate rent exempt
        let rent_expempt_amount = Rent::get()?.minimum_balance(8);
        // transfer that money to the newly created PDA so the account is published
        let tx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_account.to_account_info(),
                to: ctx.accounts.user_pda.to_account_info(),
            },
        );

        transfer(tx, rent_expempt_amount)?;

        let user_pda = &mut ctx.accounts.user_pda;
        user_pda.user_pda_bump = ctx.bumps.user_pda;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // Check if use have enough lamports

        let user_balance = ctx.accounts.user_account.lamports();
        msg!("Checking user lamports {:?}", user_balance);
        msg!("Checking amount {:?}", amount);
        require!(user_balance >= amount, Errors::InsufficientLamports);

        // send lamports from user to this programs PDA of that user
        let tx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.user_account.to_account_info(),
                to: ctx.accounts.user_pda.to_account_info(),
            },
        );

        anchor_lang::system_program::transfer(tx, amount)?;

        msg!("After user lamports {:?}", user_balance);
        msg!("After amount {:?}", amount);
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        // Check if user has enough funds stored in the PDA to withdraw
        let balance_on_pda = ctx.accounts.user_pda.get_lamports();
        require!(balance_on_pda >= amount, Errors::InsufficientLamports);

        let user_account = ctx.accounts.user_account.to_account_info();
        let user_pda = ctx.accounts.user_pda.to_account_info();

        // let system_program_info = ctx.accounts.system_program.to_account_info();
        // let transfer_ix = Transfer {
        //     from: user_pda.clone(),
        //     to: user_account.clone(),
        // };
        // let user_key = user_account.key();
        // let signer_seeds = &[b"vault", user_key.as_ref(), &[ctx.accounts.user_pda.user_pda_bump]];
        // let signer = &[&signer_seeds[..]];
        // let tx = CpiContext::new_with_signer(system_program_info, transfer_ix, signer);

        // transfer(tx, amount)?;

        **user_pda.try_borrow_mut_lamports()? -= amount;
        **user_account.try_borrow_mut_lamports()? += amount;

        msg!("User account : {:?}", user_account);
        msg!("User PDA : {:?}", user_pda);

        // send lamports to user
        Ok(())
    }

    pub fn close(_ctx: Context<Close>) -> Result<()> {
        msg!("Account closed successfully...");

        Ok(())
    }
}

#[account]
struct UserData {
    user_pda_bump: u8,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user_account: Signer<'info>,
    pub system_program: Program<'info, System>,

    #[account(init,payer=user_account, space=8 + 1, seeds=[b"vault", user_account.key.as_ref()], bump)]
    user_pda: Account<'info, UserData>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut, seeds=[b"vault", user_account.key.as_ref()], bump=user_pda.user_pda_bump)]
    user_pda: Account<'info, UserData>,

    #[account(mut)]
    user_account: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
// #[instruction(user_pda_bump: u8)]
pub struct Withdraw<'info> {
    #[account(mut, seeds=[b"vault", user_account.key.as_ref()], bump=user_pda.user_pda_bump)]
    user_pda: Account<'info, UserData>,

    #[account(mut)]
    user_account: Signer<'info>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
// #[instruction(user_pda_bump: u8)]
pub struct Close<'info> {
    user_account: Signer<'info>,

    #[account(mut, seeds=[b"vault", user_account.key().as_ref()], bump=user_pda.user_pda_bump, close=user_account)]
    user_pda: Account<'info, UserData>,
}
