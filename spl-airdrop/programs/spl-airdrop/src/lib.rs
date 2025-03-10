use anchor_lang::prelude::*;
use anchor_spl::token::{
    self, 
    MintTo, 
    Token,
    Mint,
    TokenAccount,
    mint_to,
};
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{
    create_metadata_accounts_v3, CreateMetadataAccountsV3, Metadata, mpl_token_metadata::types::DataV2,
};

// 프로그램 ID, 실제 배포 시 이 값과 deploy 결과가 일치해야함
declare_id!("4gjGVTWDi19sBVS6oj4oskGyvennpyoxYju3rtUt4Qdo");

// 프로그램 모듈, 에어드랍 프로그램 로직을 정의
#[program]
pub mod spl_airdrop {
    // 상위 스코프의 모든 구조체와 함수 등을 가져옴
    use super::*;

    /// create_mint 함수
    /// 새로운 SPL 토큰(Mint)를 생성(ctx.accounts.mint)하고, 해당 Mint의 권한을 PDA(프로그램 파생 주소)가 갖도록 설정합니다.
    /// 토큰을 생성할 때는 Metaplex 메타데이터를 생성합니다. 이 때 create_metadata_account_v3를 CPI로 호출하여 이름, 심볼, Uri를 저장합니다.
    pub fn create_mint(
        ctx: Context<CreateMint>, 
        token_name: String, 
        token_symbol: String, 
        token_uri: String) -> Result<()> {
        // PDA 서명 시드
        let signer_seeds: &[&[&[u8]]] = &[&[b"mint", &[ctx.bumps.mint_account]]];

        // Metaplex Token Metada Program CPI 호출하여 메타데이터 Account를 생성성
        create_metadata_accounts_v3(
            CpiContext::new(
                ctx.accounts.token_metadata_program.to_account_info(),
                CreateMetadataAccountsV3 {
                    metadata: ctx.accounts.metadata_account.to_account_info(),
                    mint: ctx.accounts.mint_account.to_account_info(),
                    mint_authority: ctx.accounts.mint_account.to_account_info(),
                    update_authority: ctx.accounts.mint_account.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                },
                // &signer
            ).with_signer(signer_seeds),
            DataV2 {
                name: token_name.clone(),
                symbol: token_symbol.clone(),
                uri: token_uri.clone(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            false,  // is mutable (메타데이터를 수정할 수 있는지? false이므로 수정 불가)
            true,   // update authoirty is signer (업데이터가 signer인지, 즉 메타데이터를 만든 사람만이 업데이트 가능)
            None,   // collection details (컬렉션, Uses 등의 추가 기능이 없으므로 None)
        )?;

        // msg! 매크로를 이용하면 anchor test 명령어로 테스트 시 로그가 남아서 유용하게 사용할 수 있습니다.
        msg!("Token mint created!!");
        // 성공 시 Ok(())를 반환
        Ok(())
    }

    /// Airdrop
    /// create_mint를 통해 만든 Mint(mint_account)에서 토큰을 발행하고
    /// 특정 지갑(payer)의 ATA(associated_token_account)에 amount만큼 전송합니다.
    /// PDA seeds = [b"mint"]를 이용하여 서명하고, amount는 decimals=9로 계산하여 전달합니다. (예를 들어 토큰 1개면 amount=1_000_000_000)
    pub fn airdrop(ctx: Context<Airdrop>, mint_pubkey: Pubkey, amount: u64) -> Result<()> {
        // mint_to에 필요한 cpi_accounts를 생성합니다.
        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint_account.to_account_info(),
            to: ctx.accounts.associated_token_account.to_account_info(),
            authority: ctx.accounts.mint_account.to_account_info(),
        };

        // PDA 서명 시드
        let signer_seeds: &[&[&[u8]]] = &[&[b"mint", &[ctx.bumps.mint_account]]];

        // 이제 mint_to CPI를 호출합니다. signer로 PDA를 사용하기 위해서 with_signer를 호출합니다.
        mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(), 
                cpi_accounts, 
            )
            .with_signer(signer_seeds),
            amount,
        )?;

        msg!("Airdropped {} tokens of mint {} to {}", amount, mint_pubkey, ctx.accounts.associated_token_account.key());

        Ok(())
    }
}

// CreateMint 계정(Account) 구조체
// program의 create_mint 함수를 호출할 때 필요한 계정을 정의합니다.
// <'info>는 러스트의 중요한 개념 중 하나인 라이프타임으로, CreateMint가 동작하는 스코프를 'info로 이름지은 것입니다.
#[derive(Accounts)]
pub struct CreateMint<'info> {
    // 트랜잭션 수수료 및 rent 등을 지불하는 지갑
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK:
    /// mint_account는 새로 생성할 Mint 계정을 뜻합니다.
    /// PDA seeds = [b"mint"], bump는 Anchor가 자동 계산합니다.
    /// deciamls = 9, authority는 자기 자신(mint_account), freeze_authority 또한 자기 자신을 뜻합니다.
    #[account(
        init,
        seeds = [b"mint"],
        bump,
        payer = payer, 
        mint::decimals = 9, 
        mint::authority = mint_account.key(), 
        mint::freeze_authority = mint_account.key(),
    )]
    pub mint_account: Account<'info, Mint>,

    /// CHECK:
    /// metadata_account는 토큰의 Metaplex Token Metadata 계정(metadata PDA)을 뜻합니다.
    #[account(
        mut,
        seeds = [b"metadata", token_metadata_program.key().as_ref(), mint_account.key().as_ref()],
        bump,
        seeds::program = token_metadata_program.key(),
    )]
    pub metadata_account: UncheckedAccount<'info>,

    // 민팅과 같은 CPI호출을 위해 필요한 SPL Token Program입니다.
    pub token_program: Program<'info, Token>,
    // Metaplex 메타데이터를 이용하기 위한 Metaplex Token Metadata Program입니다.
    pub token_metadata_program: Program<'info, Metadata>,
    // 계정 생성과 같은 기본 기능을 이용하기 위한 시스템 프로그램입니다.
    pub system_program: Program<'info, System>,
    // 임대료를 뜻하는 Sysvar
    pub rent: Sysvar<'info, Rent>,
}

// Airdrop 계정(Account) 구조체
// airdrop 함수 호출 시 필요한 계정을 정의합니다.
#[derive(Accounts)]
pub struct Airdrop<'info> {
    // 마찬가지로 트랜잭션 수수료 및 rent 등을 지불할 사용자 지갑
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK:
    /// 이미 생성된 SPL Mint 계정
    #[account(
        mut,
        seeds = [b"mint"],
        bump,
    )]
    pub mint_account: Account<'info, Mint>,

    /// CHECK:
    /// recipient_token_account는 수령자의 토큰 계정을 의미합니다.
    /// init_if_needed를 통해, 만약 계정이 없으면 새로 생성합니다.
    /// init_if_needed는 Cargo.toml에서 features enable을 해줘야합니다. (자세한 내용은 Cargo.toml 참조)
    /// associated_token_account는 "토큰"과 "유저 잔고"를 매핑해주는 계정이므로, 어떤 토큰인지는 mint가 뜻하고, authority는 잔고 계정을 뜻합니다.
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_account,
        associated_token::authority = payer,
    )]
    pub associated_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}