use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};

declare_id!("HXYeNBMbc5SXSsqgeJDiKZZ3wsG6i8VPjZn3NGr4aHXw");

#[program]
pub mod aixblock_contribution {
    use super::*;

    pub fn log_contribution(
        ctx: Context<LogContribution>,
        category: ContributionType,
        impact: u8,
    ) -> Result<()> {
        let contributor = &mut ctx.accounts.contributor;
        require!(impact <= 10, CustomError::InvalidImpact);

        let points = category.assign_points(impact)?;
        contributor.total_points += points;
        contributor.contributions.push(Contribution {
            category,
            impact,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn distribute_rewards(ctx: Context<DistributeRewards>, monthly_pool: u64) -> Result<()> {
        let contributors = &mut ctx.accounts.global_state.contributors;
        let total_points: u64 = contributors.iter().map(|c| c.total_points).sum();

        require!(total_points > 0, CustomError::NoContributions);
        let distributed_pool = if total_points < 500 {
            monthly_pool / 2
            //We have to send the 50% to a reserve fund
        } else {
            monthly_pool
        };


        //Naive and easy implementation to transfer tokens

        for contributor in contributors.iter_mut() {
            let reward = (contributor.total_points as u128 * distributed_pool as u128)
                / total_points as u128;
            token::transfer(ctx.accounts.into_transfer_context(), reward as u64)?;
        }

        

        //Better approach would be to pre-compute everything at once hence saving time and CU
        let mut precomputed_rewards = vec![];

        for contributor in contributors.iter() {
            let reward = (contributor.total_points as u128 * distributed_pool as u128)
                / total_points as u128;
            precomputed_rewards.push((contributor.wallet, reward as u64));
        }

        for (wallet, amount) in precomputed_rewards {
            token::transfer(ctx.accounts.into_transfer_context(), amount)?;
        }


        Ok(())
    }
}

#[derive(Accounts)]
pub struct LogContribution<'info> {
    #[account(mut)]
    pub contributor: Account<'info, Contributor>,
}

#[derive(Accounts)]
pub struct DistributeRewards<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Contributor {
    pub wallet: Pubkey,
    pub total_points: u64,
    pub contributions: Vec<Contribution>, //Maybe we can only keep the count instead of while contributions
}

#[account]
pub struct GlobalState {
    pub contributors: Vec<Contributor>,
    pub reward_mint: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Contribution {
    pub category: ContributionType,
    pub impact: u8,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum ContributionType {
    BugFix,
    FeatureDev,
    CodeOptimization,
    BugReport,
    TestContribution,
}

impl ContributionType {
    pub fn assign_points(&self, impact: u8) -> Result<u64> {
        let points = match self {
            ContributionType::BugFix => match impact {
                1..=2 => 1,
                3..=5 => 3,
                6..=8 => 6,
                9..=10 => 9,
                _ => return Err(CustomError::InvalidImpact.into()),
            },
            ContributionType::FeatureDev => match impact {
                2..=4 => 2,
                5..=7 => 5,
                8..=10 => 8,
                _ => return Err(CustomError::InvalidImpact.into()),
            },
            ContributionType::CodeOptimization => match impact {
                1..=2 => 1,
                3..=5 => 3,
                6..=8 => 6,
                _ => return Err(CustomError::InvalidImpact.into()),
            },
            ContributionType::BugReport => match impact {
                1..=2 => 1,
                3..=5 => 3,
                6..=8 => 6,
                9..=10 => 9,
                _ => return Err(CustomError::InvalidImpact.into()),
            },
            ContributionType::TestContribution => match impact {
                2..=4 => 2,
                5..=7 => 5,
                _ => return Err(CustomError::InvalidImpact.into()),
            },
        };
        Ok(points as u64)
    }
}

#[error_code]
pub enum CustomError {
    #[msg("Invalid impact level.")]
    InvalidImpact,
    #[msg("No contributions to distribute rewards.")]
    NoContributions,
}
