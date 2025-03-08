use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, 
    MintTo, 
    Token, 
    InitializeMint
};

// 프로그램 ID, 실제 배포 시 이 값과 deploy 결과가 일치해야함
declare_id!("4gjGVTWDi19sBVS6oj4oskGyvennpyoxYju3rtUt4Qdo");

// 에어드랍 프로그램 로직을 정의
#[program]
pub mod spl_airdrop {
    // 상위 스코프의 모든 구조체와 함수 등을 가져옴옴
    use super::*;

    /// create_mint 함수
    /// 새로운 SPL 토큰(Mint)를 생성(ctx.accounts.mint)하고, 해당 Mint의 권한을 PDA(프로그램 파생 주소)가 갖도록 설정합니다.
    /// `airdrop_state` 계정에 Mint 주소를 저장하고, 나중에 실제 에어드랍 시 참조할 수 있도록 설정합니다.
    pub fn create_mint(ctx: Context<CreateMint>) -> Result<()> {
        // cpi_context는 어떤 프로그램에 대한 호출인지, 어떤 계정들을 넘길지 정의합니다.
        // cpi : cross program invocation
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(), 
            InitializeMint {
                mint: ctx.accounts.mint.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            });

        // mint 계정을 초기화합니다. decimal은 9, mint_authority(권한) = payer, freeze_authority = payer로 권한을 설정합니다.
        // ?가 붙어있으므로 에러가 발생하면 상위 스코프로 에러를 전달합니다.
        token::initialize_mint(cpi_ctx, 9, &ctx.accounts.payer.key(), Some(&ctx.accounts.payer.key()))?;
        
        // 성공 시 Ok(())를 반환
        Ok(())
    }

    /// Airdrop
    /// 특정 지갑(recipient_token_account)의 토큰 어카운트에 토큰을 amount 만큼 전송합니다.
    /// 프로그램 PDA(seeds=[b"authority"])로 서명합니다.
    pub fn airdrop(ctx: Context<Airdrop>, amount: u64) -> Result<()> {
        // mint_to에 필요한 cpi_accounts를 생성합니다.
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        };
        // PDA 서명에 필요한 seeds를 정의합니다. signer seeds는 b"authority" + authority_bump입니다.
        let signer_seeds: &[&[u8]] = &[
            b"authority".as_ref(),
            &[ctx.accounts.airdrop_state.authority_bump],
        ];
        // 이제 mint_to CPI를 호출합니다. signer로 PDA를 사용하기 위해서 new_with_signer를 호출합니다.
        token::mint_to(CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), cpi_accounts, &[signer_seeds]), amount)?;

        Ok(())
    }
}

// CreateMint 계정(Account) 구조체
// program의 create_mint 함수를 호출할 때 필요한 계정을 정의합니다.
// <'info>는 러스트의 중요한 개념 중 하나인 라이프타임으로, CreateMint가 동작하는 스코프를 'info로 이름지은 것입니다.
#[derive(Accounts)]
pub struct CreateMint<'info> {
    /// CHECK:
    /// mint는 새로 생성할 Mint 계정(UncheckedAccount)
    /// SystemProgram으로 임대료(rent)만큼 할당하고, initialize_mint CPI로 SPL 구조를 입힙니다.
    /// <'info>는 마찬가지로 mint의 라이프타임을 CreateMint의 라이프타임인 'info로 명시해주는 것입니다.
    #[account(mut)]
    pub mint: UncheckedAccount<'info>,
    
    // 트랜잭션 수수료 및 임대료 면제 등을 지불하는 지갑
    #[account(mut)]
    pub payer: Signer<'info>,

    // Anchor에서 CPI 호출 시 어떤 프로그램에 전달할지를 지정합니다.
    // 에어드랍은 CPI이므로 spl-token 프로그램으로 지정합니다.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,

    // 솔라나 시스템 프로그램
    pub system_program: Program<'info, System>,

    // 임대료 관련 정보를 담는 변수
    pub rent: Sysvar<'info, Rent>,
}

// Airdrop 계정(Account) 구조체
// airdrop 함수 호출 시 필요한 계정을 정의합니다.
#[derive(Accounts)]
pub struct Airdrop<'info> {
    // airdop_state는 에어드롭 관련 전역 상태를 저장합니다.
    // 여기서는 AirdropState, 즉 mint 주소와 authority_bump 값을 저장하여 어떤 토큰을 에어드랍할지(mint), PDA 서명을 위한 값(bump)를 전역으로 저장합니다.
    #[account(mut)]
    pub airdrop_state: Account<'info, AirdropState>,

    /// CHECK:
    /// 이미 생선된 SPL Mint 계정
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK:
    /// PDA authority로 seeds=[b"authority"], bump=airdrop_state.authority.bump로 검증합니다.
    #[account(seeds=[b"authority"], bump=airdrop_state.authority_bump)]
    pub authority: AccountInfo<'info>,

    /// CHECK:
    /// recipient_token_account는 수령자의 토큰 계정을 의미합니다.
    #[account(mut)]
    pub recipient_token_account: UncheckedAccount<'info>,

    // mint_to CPI 호출에 필요한 SPL-Token 프로그램입니다.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct AirdropState {
    // 토큰 mint 주소를 통해 create_mint 시점에 생성된 Mint Pubkey 값을 저장하거나 외부에서 주입받아 설정할 수 있습니다.
    pub mint: Pubkey,
    // PDA 생성에 필요한 bump 값
    pub authority_bump: u8,
}