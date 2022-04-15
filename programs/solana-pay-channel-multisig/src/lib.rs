use anchor_lang::prelude::Pubkey;
use anchor_lang::prelude::*;
use anchor_lang::prelude::{Key, Signer};
use anchor_lang::solana_program::system_program;

declare_id!("4STy3H6Nw4bqFE48STm3aoDnhqu3DLhhHvoRN1s5WLko");

#[program]
pub mod solana_pay_channel_multisig {
    use super::*;

    use anchor_lang::solana_program::{
        program::{invoke},
        system_instruction::{transfer}
    };

    
    pub fn create_payment_user(ctx: Context<CreatePaymentUser>,user_name:String) -> Result<()> {
        let multi_sig_wallet = &mut ctx.accounts.payment_user;
        multi_sig_wallet.user_name = user_name;
        multi_sig_wallet.key = ctx.accounts.user.key();
        Ok(())
    }

    pub fn create_multisig_wallet(ctx: Context<CreateMultiSigWallet>,user_1_contribution:u64,user_2_contribution:u64) -> Result<()> {
        let multi_sig_wallet = &mut ctx.accounts.multisig_wallet;
        let user_1_obj = &mut ctx.accounts.user_1;
        let user_2_obj = &mut ctx.accounts.user_2;

        let user_1_balance = user_1_obj.to_account_info().lamports();
        let user_2_balance = user_2_obj.to_account_info().lamports();

        if user_1_contribution > user_1_balance {
            return Err(ErrorCode::NotEnoughLamports.into());
        }

        if user_2_contribution > user_2_balance {
            return Err(ErrorCode::NotEnoughLamports.into());
        }

        multi_sig_wallet.user_1 = *user_1_obj.to_account_info().unsigned_key();
        multi_sig_wallet.user_2 = *user_2_obj.to_account_info().unsigned_key();

        multi_sig_wallet.user_1_balance = user_1_contribution;
        multi_sig_wallet.user_2_balance = user_2_contribution;

        let transfer_instruction_user_1 = &transfer(
            &multi_sig_wallet.user_1,
            &ctx.accounts.owner.to_account_info().key,
            user_1_contribution,
        );

        invoke(
            transfer_instruction_user_1,
            &[
                ctx.accounts.user_1.to_account_info(),
                ctx.accounts.owner.to_account_info(),       
            ]
        )?;

        let transfer_instruction_user_2 = &transfer(
            &multi_sig_wallet.user_2,
            &ctx.accounts.owner.to_account_info().key,
            user_2_contribution,
        );

        invoke(
            transfer_instruction_user_2,
            &[
                ctx.accounts.user_2.to_account_info(),
                ctx.accounts.owner.to_account_info(),       
            ]
        )?;

        Ok(())

    }

    pub fn update_balance(ctx: Context<UpdateBalance>,new_user_1_balance: u64, new_user_2_balance: u64) -> Result<()>{
        let multi_sig_wallet = &mut ctx.accounts.multisig_wallet;
        let previous_balance = multi_sig_wallet.user_1_balance + multi_sig_wallet.user_2_balance;
        if previous_balance != (new_user_1_balance+new_user_2_balance) {
            return Err(ErrorCode::InvalidBalances.into());
        }
        if *ctx.accounts.user_1.to_account_info().key != multi_sig_wallet.user_1 &&  *ctx.accounts.user_1.to_account_info().key != multi_sig_wallet.user_2{
            return Err(ErrorCode::InvalidSigner.into());
        }
        if *ctx.accounts.user_2.to_account_info().key != multi_sig_wallet.user_1 &&  *ctx.accounts.user_2.to_account_info().key != multi_sig_wallet.user_2{
            return Err(ErrorCode::InvalidSigner.into());
        }
        multi_sig_wallet.user_1_balance = new_user_1_balance;
        multi_sig_wallet.user_2_balance = new_user_2_balance;
        Ok(())
    }

    pub fn close_channel(ctx: Context<WithdrawBalance>) -> Result<()>  {
        let multi_sig_wallet = &mut ctx.accounts.multisig_wallet;

        if *ctx.accounts.signer.to_account_info().key != multi_sig_wallet.user_1 &&  *ctx.accounts.signer.to_account_info().key != multi_sig_wallet.user_2{
            return Err(ErrorCode::InvalidSigner.into());
        }

        let user_1_balance = multi_sig_wallet.user_1_balance;
        let user_2_balance = multi_sig_wallet.user_2_balance;

        let transfer_instruction_user_1 = &transfer(
            &ctx.accounts.owner.to_account_info().key,
            &multi_sig_wallet.user_1,
            user_1_balance,
        );

        invoke(
            transfer_instruction_user_1,
            &[
                ctx.accounts.owner.to_account_info(),
                ctx.accounts.user_1.to_account_info()
            ]
        )?;


        let transfer_instruction_user_2 = &transfer(
            &ctx.accounts.owner.to_account_info().key,
            &multi_sig_wallet.user_2,
            user_2_balance,
        );

        invoke(
            transfer_instruction_user_2,
            &[
                ctx.accounts.owner.to_account_info(),
                ctx.accounts.user_2.to_account_info()
            ]
        )?;

        multi_sig_wallet.user_1_balance = 0;
        multi_sig_wallet.user_2_balance = 0;

        Ok(())
    }
    
}

#[derive(Accounts)]
pub struct CreatePaymentUser<'info> {
    #[account(init, payer = user, space = 8 + 64 + 64 + 64 + 64)]
    pub payment_user: Account<'info, PaymentUser>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateBalance<'info> {
    #[account(mut)]
    pub multisig_wallet: Account<'info, MultiSigWallet>,
    #[account(mut)]
    pub user_1: Signer<'info>,
    #[account(mut)]
    pub user_2: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct WithdrawBalance<'info> {
    #[account(mut)]
    pub multisig_wallet: Account<'info, MultiSigWallet>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub signer: Signer<'info>,
    ///CHECK: Multisig Wallet User 1 is not unsafe because of fn contraints
    #[account(mut)]
    pub user_1: AccountInfo<'info>,
     ///CHECK: Multisig Wallet User 2 is not unsafe because of fn contraints
    #[account(mut)]
    pub user_2: AccountInfo<'info>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateMultiSigWallet<'info> {
    #[account(init, payer = owner, space = 8 + 64 + 64 + 64 + 64)]
    pub multisig_wallet: Account<'info, MultiSigWallet>,
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut)]
    pub user_1: Signer<'info>,
    #[account(mut)]
    pub user_2: Signer<'info>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

#[account]
pub struct PaymentUser {
    pub user_name: String,
    pub key: Pubkey
}

#[account]
pub struct MultiSigWallet {
    pub user_1: Pubkey,
    pub user_2: Pubkey,
    pub user_1_balance: u64,
    pub user_2_balance: u64
}

#[error_code]
pub enum ErrorCode {
    #[msg("Not enough lamports in wallet")]
    NotEnoughLamports,
    #[msg("previous contribution sum doesnot match with new contribution sum")]
    InvalidBalances,
    #[msg("Not a valid signer")]
    InvalidSigner,
}
