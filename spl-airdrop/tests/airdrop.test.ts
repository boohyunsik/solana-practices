import * as anchor from "@coral-xyz/anchor";
import { SplAirdrop } from "../target/types/spl_airdrop";
import { PublicKey, Keypair } from "@solana/web3.js";
import { getAccount, getAssociatedTokenAddressSync } from "@solana/spl-token";

describe("create_mint_test", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program = anchor.workspace.SplAirdrop as anchor.Program<SplAirdrop>;
    const wallet = provider.wallet as anchor.Wallet;
    const connection = provider.connection;

    let mintPDA: PublicKey;

    before(async() => {
        console.log("Global State Initialized.");
    });

    it("Scenario 1: Mint 1 Token and Airdrop to 3 wallets", async () => {
        console.log('mint token');
        const mint = await createMint("Alpha Token", "AT", "test_uri_1");

        console.log('airdrop!');
        const userA = Keypair.generate();
        const userB = Keypair.generate();
        const userC = Keypair.generate();

        await airdropToken(mint, userA.publicKey);
        await airdropToken(mint, userB.publicKey);
        await airdropToken(mint, userC.publicKey);
    });

    // utils
    async function createMint(name: string, symbol: string, uri: string) {
        console.log('createMint');
        [mintPDA] = PublicKey.findProgramAddressSync([Buffer.from('mint')], program.programId);
        const txSig = await program.methods
            .createMint(name, symbol, uri)
            .accounts({
                payer: wallet.publicKey,
            })
            .rpc();
        console.log('mintPDA', mintPDA.toBase58());
        console.log('sig: ', txSig);
        return mintPDA;
    }

    async function airdropToken(mintPubkey: PublicKey, recipient: PublicKey) {
        // get ATA
        const ata = getAssociatedTokenAddressSync(
            mintPDA,
            wallet.publicKey,
        );
        const amount = new anchor.BN(1_000_000_000);

        await program.methods
            .airdrop(mintPubkey, amount)
            .accounts({
                associatedTokenAccount: ata,
            })
            .rpc();
        
            const accInfo = await getAccount(connection, ata);
            console.log(
                `Airdrop success: recipient=${recipient.toBase58()}, tokenAmount=${accInfo.amount}`
            );
            return ata;
    }
})