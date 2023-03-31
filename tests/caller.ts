import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  SystemProgram,
  Keypair,
  Transaction,
} from "@solana/web3.js";
import {
  createInitializeMint2Instruction,
  TOKEN_PROGRAM_ID,
  MINT_SIZE,
  createMintToInstruction,
  createAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
  createTransferInstruction,
  TokenError,
} from "@solana/spl-token";
import { Caller } from "../target/types/caller";
import { PermissionedTokenWrapper } from "../target/types/permissioned_token_wrapper";

describe("caller-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const caller = anchor.workspace.Caller as Program<Caller>;
  const program = anchor.workspace
    .PermissionedTokenWrapper as Program<PermissionedTokenWrapper>;

  let programControl: PublicKey = PublicKey.findProgramAddressSync(
    [Buffer.from("static")],
    program.programId
  )[0];

  let payer: PublicKey = program.provider.publicKey!;
  const decimals = 9;
  let mint: PublicKey;
  let tokenAccount: PublicKey;
  let tokenRecord: PublicKey;

  let randomKp = Keypair.generate();
  let randomPerson = randomKp.publicKey;
  let randoToken: PublicKey;

  before(async () => {
    let mintKp = Keypair.generate();
    mint = mintKp.publicKey;

    tokenAccount = getAssociatedTokenAddressSync(mint, payer);

    let lamports =
      await program.provider.connection.getMinimumBalanceForRentExemption(
        MINT_SIZE,
        "confirmed"
      );
    const transaction = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: payer,
        newAccountPubkey: mint,
        space: MINT_SIZE,
        lamports,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeMint2Instruction(
        mint,
        decimals,
        payer,
        programControl,
        TOKEN_PROGRAM_ID
      ),
      createAssociatedTokenAccountInstruction(
        payer,
        tokenAccount,
        payer,
        mint,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createMintToInstruction(
        mint,
        tokenAccount,
        payer,
        1,
        [],
        TOKEN_PROGRAM_ID
      )
    );

    let txid = await program.provider.sendAndConfirm(transaction, [mintKp], {
      skipPreflight: true,
      preflightCommitment: "confirmed",
    });
    console.log("\tCreated new mint with txid: ", txid);
  });
  it("Can lock user token account", async () => {
    // Add your test here.
    tokenRecord = PublicKey.findProgramAddressSync(
      [tokenAccount.toBuffer(), Buffer.from("token_record")],
      program.programId
    )[0];

    console.log(caller, caller.methods, caller.methods.lock);
    try {
      const tx = await caller.methods
        .lock()
        .accounts({
          token: tokenAccount,
          mint,
          delegate: program.provider.publicKey!,
          payer: program.provider.publicKey!,
          tokenProgram: TOKEN_PROGRAM_ID,
          permProgram: program.programId,
        })
        .remainingAccounts([
          { pubkey: tokenRecord, isSigner: false, isWritable: true },
          { pubkey: programControl, isSigner: false, isWritable: false },
          {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
        ])
        .rpc({ skipPreflight: true });
      console.log("\tLocked", tx);
    } catch (e) {
      console.log(e);
    }
  });
  //   it("Cannot transfer locked token", async () => {
  //     randoToken = getAssociatedTokenAddressSync(mint, randomPerson);

  //     let transaction = new Transaction().add(
  //       createAssociatedTokenAccountInstruction(
  //         payer,
  //         randoToken,
  //         randomPerson,
  //         mint,
  //         TOKEN_PROGRAM_ID,
  //         ASSOCIATED_TOKEN_PROGRAM_ID
  //       ),
  //       createTransferInstruction(tokenAccount, randoToken, payer, 1, [])
  //     );

  //     try {
  //       let txid = await program.provider.sendAndConfirm(transaction, [], {
  //         skipPreflight: true,
  //         preflightCommitment: "confirmed",
  //       });
  //       throw Error("Should not be able to transfer locked token");
  //     } catch (e) {
  //       console.log("\tSuccessfully failed to transfer locked token:", e);
  //     }
  //   });
  //   it("Can unlock token", async () => {
  //     const tx = await program.methods
  //       .unlock()
  //       .accounts({
  //         token: tokenAccount,
  //         mint,
  //         programControl,
  //         tokenRecord,
  //         delegate: program.provider.publicKey!,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //       })
  //       .rpc();
  //     console.log("\tUnlocked: ", tx);
  //   });
  //   it("Can transfer unlocked token", async () => {
  //     let transaction = new Transaction().add(
  //       createAssociatedTokenAccountInstruction(
  //         payer,
  //         randoToken,
  //         randomPerson,
  //         mint,
  //         TOKEN_PROGRAM_ID,
  //         ASSOCIATED_TOKEN_PROGRAM_ID
  //       ),
  //       createTransferInstruction(tokenAccount, randoToken, payer, 1, [])
  //     );

  //     let txid = await program.provider.sendAndConfirm(transaction, [], {
  //       skipPreflight: true,
  //       preflightCommitment: "confirmed",
  //     });
  //     console.log("\tTransferred token to normie: ", txid);
  //   });
});
