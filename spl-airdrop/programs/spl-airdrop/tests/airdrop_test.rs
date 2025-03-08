use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Token, Mint, TokenAccount, InitializeMint};
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use spl_airdrop::spl_airdrop;

#[tokio::test]
async fn test_create_mint() {
    let mut program_test = ProgramTest::new(
        "airdrop_program",
        spl_airdrop::ID,
        processor!(spl_airdrop::entry),
    );

    let payer = Keypair::new();
    program_test.add_account(
        payer.pubkey(),
        Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: system_program::ID,
            ..Account::default()
        },
    );

    let (mut banks_client, payer_pubkey, recent_blockhash) = program_test.start().await;

    // mint 계정 준비
    let mint_key = Keypair::new();
    let mint_pubkey = mint_key.pubkey();

    let create_mint_tx = anchor_lang::InstructionData::data(&spl_airdrop::instruction::CreateMint {});

    let accounts = spl_airdrop::accounts::CreateMint {
        mint: mint_pubkey,
        payer: payer_pubkey,
        token_program: anchor_spl::token::ID,
        system_program: system_program::ID,
        rent: sysvar::rent::id(),
    }.to_account_metas(None);
    
    banks_client.process_transaction(tx).await.unwrap();

    let mint_account_data = banks_client
        .get_account(mint_pubkey)
        .await
        .expect("get_account")
        .expect("mint not found");

    assert_eq!(mint_account_data.owner, anchor_spl::token::ID);

    println!("Mint가 성공적으로 생성됨: {}", mint_pubkey);
}