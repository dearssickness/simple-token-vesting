import * as anchor from "@coral-xyz/anchor";
import { Program, Idl } from "@coral-xyz/anchor";
import idl from "../target/idl/simple_token_vesting.json";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import { assert } from "chai";
import { findProgramAddressSync } from "@project-serum/anchor/dist/cjs/utils/pubkey";

describe("simple_token_vesting", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const programId = new PublicKey("9Dt3WPawaT6Jf2aTxauKRhsmrBAn84zA3Mi5uitaWZs3");
  const program = new Program(idl as Idl, provider);
  const user = Keypair.generate();

  let config: PublicKey;
  let escrow_wallet: PublicKey;
  let authority: PublicKey;
  let admin_token_account: PublicKey;
  let token_mint: PublicKey;
  let beneficiary_data: PublicKey;
  let beneficiary_wallet: PublicKey;
  
  const decimals = 6;
  const amount = 1000;

  const percent = 10;

  before(async () => {
    const airdropSignature = await provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    const latestBlockHash = await provider.connection.getLatestBlockhash()

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: airdropSignature,
    })

    token_mint= await createMint(provider.connection, user, user.publicKey, null, 9);

    admin_token_account = await createAccount(provider.connection, user, token_mint, user.publicKey);
    beneficiary_wallet = await createAccount(provider.connection, user, token_mint, user.publicKey);

    const [configPda] = findProgramAddressSync([Buffer.from("config_vesting")],programId)
    config = configPda;
    
    const [escrowPda] = findProgramAddressSync([Buffer.from("escrow_wallet"), config.toBuffer()],programId)
    escrow_wallet = escrowPda;

    const [authorityPda] = findProgramAddressSync([Buffer.from("authority")],programId)
    authority = authorityPda;

    const [beneficiaryDataPda] = findProgramAddressSync(
      [
      Buffer.from("beneficiary_data"), 
      user.publicKey.toBuffer()
      ],
      programId
    )

    beneficiary_data = beneficiaryDataPda;

    await mintTo(provider.connection, user, token_mint, admin_token_account, user, 1500);
  });

  it("Initialize vesting", async () => {

    const adminTokenAccountBefore = await getAccount(provider.connection, admin_token_account);
    const escrowWalletBefore = await getAccount(provider.connection, escrow_wallet);

    await program.methods
      .initialize(
      new anchor.BN(amount),
      new anchor.BN(decimals)
      )
      .accounts({
          config: config,
          escrowWallet: escrow_wallet,
          authority: authority,
          user: user,
          adminTokenAccount: admin_token_account,
          tokenMint: token_mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

    const adminTokenAccountAfter = await getAccount(provider.connection, admin_token_account);
    const escrowWalletAfter = await getAccount(provider.connection, escrow_wallet);

    assert.equal(
      Number(adminTokenAccountBefore.amount) - amount,
      Number(adminTokenAccountAfter.amount),
      "Admin token account should decrease"
    );

    assert.equal(
      Number(escrowWalletBefore.amount) + amount,
      Number(escrowWalletAfter.amount),
      "Escrow wallet should increase"
    );

  });

  it("Set percent for beneficiary", async () => {

    await program.methods
      .release(
      new anchor.BN(percent)
      )
      .accounts({
          config: config,
          authority: authority,
        })
        .signers([user])
        .rpc();

  });

  it("Claim vesting", async () => {

    await program.methods
      .claim()
      .accounts({
          config: config,
          beneficiary_data: beneficiary_data,
          beneficiary_wallet: beneficiary_wallet,
          escrowWallet: escrow_wallet,
          authority: authority,
          user: user,
          tokenMint: token_mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

    });

});