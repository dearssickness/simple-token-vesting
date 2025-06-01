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
  const admin = Keypair.generate();
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
  const total_tokens = 500;
  const startTime = Math.floor(Date.now() / 1000);
  const cliffDuration = 1; // 1 Seconds obviously for tests to pass!
  const vestingDuration = 20; // 20 Seconds ^^^^^^^^^^^^^^^^^^^^^^^^^

  before(async () => {
    const airdropSignature = await provider.connection.requestAirdrop(admin.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    const latestBlockHash = await provider.connection.getLatestBlockhash()

    await provider.connection.confirmTransaction({
      blockhash: latestBlockHash.blockhash,
      lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      signature: airdropSignature,
    })


    token_mint = await createMint(provider.connection, admin, admin.publicKey, null, 9);

    admin_token_account = await createAccount(provider.connection, admin, token_mint, admin.publicKey);
    beneficiary_wallet = await createAccount(provider.connection, admin, token_mint, beneficiary.publicKey);

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

    await mintTo(provider.connection, admin, token_mint, admin_token_account, admin, 15000000);

    await program.methods
      .initializeAccounts()
      .accounts({
          config: config,
          escrowWallet: escrow_wallet,
          beneficiaryData: beneficiary_data,
          beneficiaryWallet: beneficiary_wallet,
          authority: authority,
          admin: admin.publicKey,
          adminTokenAccount: admin_token_account,
          tokenMint: token_mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();
  });

  it("Add a beneficiary", async () => {
    
    await program.methods
      .addBeneficiary(
      new anchor.BN(total_tokens),
      new anchor.BN(beneficiary_wallet.toBuffer()),
      )
      .accounts({
          beneficiaryWallet: beneficiary_wallet,
          beneficiaryData: beneficiary_data,
          admin: admin,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
  });

  it("Initialize vesting", async () => {

    const adminTokenAccountBefore = await getAccount(provider.connection, admin_token_account);

    await program.methods
      .initializeVesting(
      new anchor.BN(amount),
      new anchor.BN(decimals),
      new anchor.BN(startTime),
      new anchor.BN(cliffDuration),
      new anchor.BN(vestingDuration)
      )
      .accounts({
          config: config,
          escrowWallet: escrow_wallet,
          authority: authority,
          admin: admin.publicKey,
          adminTokenAccount: admin_token_account,
          tokenMint: token_mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
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
    const autoVesting = false;
    const vestingInvoked = false;

    await program.methods
      .release(
      new anchor.BN(percent),
      autoVesting,
      vestingInvoked
      )
      .accounts({
          config: config,
          authority: authority,
          admin: admin.publicKey,
          tokenMint: token_mint
        })
        .signers([admin])
        .rpc();
  });

  it("Claim vesting", async () => {

    const beneficiaryWalletBefore = await getAccount(provider.connection, beneficiary_wallet);

    await program.methods
      .claim()
      .accounts({
          config: config,
          beneficiaryData: beneficiary_data,
          beneficiaryWallet: beneficiary_wallet,
          escrowWallet: escrow_wallet,
          authority: authority,
          user: admin.publicKey,
          tokenMint: token_mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin])
        .rpc();

    const beneficiaryWalletAfter = await getAccount(provider.connection, beneficiary_wallet);

    assert.equal(
        Number(beneficiaryWalletBefore.amount) + (total_tokens * percent) / 100 ,
        Number(beneficiaryWalletAfter.amount),
        "Beneficiary wallet should increase by percent of total_tokens"
    )

    });

it("Invoke vesting", async () => {

    await program.methods
      .invokeVesting()
      .accounts({
          config: config,
          escrowWallet: escrow_wallet,
          authority: authority,
          admin: admin.publicKey,
          adminTokenAccount: admin_token_account,
          tokenMint: token_mint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();

    });

it("Reconfigure vesting", async () => {
    const autoVesting = true;
    const vestingInvoked = false;

    await program.methods
      .reconfigureVesting(
      autoVesting,
      vestingInvoked
      )
      .accounts({
          config: config,
          authority: authority,
          admin: admin.publicKey,
          tokenMint: token_mint,
        })
        .signers([admin])
        .rpc();
});

});