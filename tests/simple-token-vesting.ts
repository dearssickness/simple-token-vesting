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
  const beneficiary = Keypair.generate();

  let config: PublicKey;
  let escrow_wallet: PublicKey;
  let authority: PublicKey;
  let admin_token_account: PublicKey;
  let token_mint: PublicKey;
  let beneficiary_data: PublicKey;
  let beneficiary_wallet: PublicKey;
  
  const decimals = 2;
  const amount = 5;

  const percent = 10;

  before(async () => {
    const airdropSignature = await provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    const latestBlockHash = await provider.connection.getLatestBlockhash()

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: airdropSignature,
    })


    token_mint = await createMint(provider.connection, user, user.publicKey, null, 9);

    admin_token_account = await createAccount(provider.connection, user, token_mint, user.publicKey);
    beneficiary_wallet = await createAccount(provider.connection, user, token_mint, beneficiary.publicKey);

    const [configPda] = findProgramAddressSync([Buffer.from("config_vesting"), token_mint.toBuffer()],programId)
    config = configPda;
    
    const [escrowPda] = findProgramAddressSync([Buffer.from("escrow"), config.toBuffer()],programId)
    escrow_wallet = escrowPda;

    const [authorityPda] = findProgramAddressSync([Buffer.from("authority"), token_mint.toBuffer()],programId)
    authority = authorityPda;

    const [beneficiaryDataPda] = findProgramAddressSync(
      [
      Buffer.from("beneficiary_data"), 
      beneficiary_wallet.toBuffer()
      ],
      programId
    )

    beneficiary_data = beneficiaryDataPda;

    await mintTo(provider.connection, user, token_mint, admin_token_account, user, 15000000);

    await program.methods
      .initializeAccounts()
      .accounts({
          config: config,
          escrowWallet: escrow_wallet,
          beneficiaryData: beneficiary_data,
          beneficiaryWallet: beneficiary_wallet,
          authority: authority,
          user: user.publicKey,
          adminTokenAccount: admin_token_account,
          tokenMint: token_mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();
  });

  it("Add a beneficiary", async () => {
    
    const total_tokens = 5000;

    await program.methods
      .addBeneficiary(
      new anchor.BN(total_tokens),
      new anchor.BN(beneficiary_wallet.toBuffer()),
      )
      .accounts({
          beneficiaryWallet: beneficiary_wallet,
          beneficiaryData: beneficiary_data,
          user: user,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
  });

  it("Initialize vesting", async () => {

    const adminTokenAccountBefore = await getAccount(provider.connection, admin_token_account);

    await program.methods
      .initializeVesting(
      new anchor.BN(amount),
      new anchor.BN(decimals)
      )
      .accounts({
          config: config,
          escrowWallet: escrow_wallet,
          authority: authority,
          user: user.publicKey,
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
      BigInt(Number(adminTokenAccountBefore.amount)) - (BigInt(amount) * BigInt(10 ** decimals)),
      BigInt(Number(adminTokenAccountAfter.amount)),
      "Admin token account should decrease"
    );

    assert.equal(
      Number(escrowWalletAfter.amount),
      amount * 10 ** decimals,
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
          user: user.publicKey,
          tokenMint: token_mint
        })
        .signers([user])
        .rpc();
  });

  it("Claim vesting", async () => {

    await program.methods
      .claim()
      .accounts({
          config: config,
          beneficiaryData: beneficiary_data,
          beneficiaryWallet: beneficiary_wallet,
          escrowWallet: escrow_wallet,
          authority: authority,
          user: user.publicKey,
          tokenMint: token_mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

    });

});