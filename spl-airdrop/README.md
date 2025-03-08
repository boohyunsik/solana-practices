# SPL-Token Airdrop Program
---

이 레포지토리는 SPL(Solana Program Library) Token을 만들고 에어드랍해주는 간단한 예제 프로그램입니다,

# Flow
---

### Token Mint 생성

1. create_mint 함수 호출 : 입력 파라미터 `CreateMint`
2. system_program을 통해 mint 계정에 rent-exempt 만큼 할당하고, initialize_mint(CPI, Cross Program Invocation)를 호출합니다. 그러면 Mint 계정을 SPL-Token으로 감싸줍니다.
3. 최종적으로 SPL-Token 표준화된 Mint 계정이 생성됩니다.

### Airdrop 실행
1. airdrop 함수 호출 : 입력 파라미터 `Airdrop`, u64타입의 amount
2. CPI 호출을 위한 계정 준비(`cpi_accounts`) : `MintTo`를 통해 어떤 토큰을 누구에게 할당할지, PDA 서명을 위한 데이터(`b"authority" + authority_bump`)를 준비합니다.
3. CPI를 통해 mint_to 함수를 호출합니다.
4. 정상적으로 실행되면 `recipient_token_account`의 토큰 잔고가 `amount`만큼 증가합니다.

# Sequence Diagram
---

### create_mint()
![create_mint](./img/mint.png)

### airdrop()
![airdrop](./img/airdrop.png)