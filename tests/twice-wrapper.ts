// import * as anchor from "@coral-xyz/anchor";
// import { Program } from "@coral-xyz/anchor";
// import {
//   PublicKey,
//   SystemProgram,
//   Keypair,
//   Transaction,
// } from "@solana/web3.js";
// import {
//   createInitializeMint2Instruction,
//   TOKEN_PROGRAM_ID,
//   MINT_SIZE,
//   createMintToInstruction,
//   createAssociatedTokenAccount,
//   createAssociatedTokenAccountInstruction,
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   getAssociatedTokenAddressSync,
//   createTransferInstruction,
//   TokenError,
// } from "@solana/spl-token";
// import { PermissionedTokenWrapper } from "../target/types/permissioned_token_wrapper";
// import { assert } from "chai";
// import { TwicePermissioned } from "../target/types/twice_permissioned";
//
// describe("twice-permissioned", () => {
//   // Configure the client to use the local cluster.
//   anchor.setProvider(anchor.AnchorProvider.env());
//
//   const program = anchor.workspace
//     .TwicePermissioned as Program<TwicePermissioned>;
//
//   let programControl: PublicKey = PublicKey.findProgramAddressSync(
//     [Buffer.from("static")],
//     program.programId
//   )[0];
//
//   let payer: PublicKey = program.provider.publicKey!;
//   const decimals = 9;
//   let mint: PublicKey;
//   let tokenAccount: PublicKey;
//   let tokenRecord: PublicKey;
//
//   let randomKp = Keypair.generate();
//   let randomPerson = randomKp.publicKey;
//   let randoToken: PublicKey;
//
//   before(async () => {
//     let mintKp = Keypair.generate();
//     mint = mintKp.publicKey;
//
//     tokenAccount = getAssociatedTokenAddressSync(mint, payer);
//
//     let lamports =
//       await program.provider.connection.getMinimumBalanceForRentExemption(
//         MINT_SIZE,
//         "confirmed"
//       );
//     const transaction = new Transaction().add(
//       SystemProgram.createAccount({
//         fromPubkey: payer,
//         newAccountPubkey: mint,
//         space: MINT_SIZE,
//         lamports,
//         programId: TOKEN_PROGRAM_ID,
//       }),
//       createInitializeMint2Instruction(
//         mint,
//         decimals,
//         payer,
//         programControl,
//         TOKEN_PROGRAM_ID
//       ),
//       createAssociatedTokenAccountInstruction(
//         payer,
//         tokenAccount,
//         payer,
//         mint,
//         TOKEN_PROGRAM_ID,
//         ASSOCIATED_TOKEN_PROGRAM_ID
//       ),
//       createMintToInstruction(
//         mint,
//         tokenAccount,
//         payer,
//         1,
//         [],
//         TOKEN_PROGRAM_ID
//       )
//     );
//
//     let txid = await program.provider.sendAndConfirm(transaction, [mintKp], {
//       skipPreflight: true,
//       preflightCommitment: "confirmed",
//     });
//     console.log("\tCreated new mint with txid: ", txid);
//   });
//   it("Can lock user token account", async () => {
//     // Add your test here.
//     tokenRecord = PublicKey.findProgramAddressSync(
//       [tokenAccount.toBuffer(), Buffer.from("token_record")],
//       program.programId
//     )[0];
//
//     let tx = await program.methods
//       .lock()
//       .accounts({
//         token: tokenAccount,
//         mint,
//         programControl,
//         tokenRecord,
//         payer: program.provider.publicKey!,
//         delegate: program.provider.publicKey!,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc({ commitment: "confirmed" });
//
//     let record = await program.account.tokenRecord.fetch(
//       tokenRecord,
//       "confirmed"
//     );
//     assert(record.locked == 0x0, "Token record should be unlocked");
//
//     tx = await program.methods
//       .lock()
//       .accounts({
//         token: tokenAccount,
//         mint,
//         programControl,
//         tokenRecord,
//         payer: program.provider.publicKey!,
//         delegate: program.provider.publicKey!,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc({ commitment: "confirmed" });
//
//     record = await program.account.tokenRecord.fetch(tokenRecord, "confirmed");
//     assert(
//       record.locked == 0x1,
//       "Token record should be locked on 2nd invocation"
//     );
//
//     console.log("\tLocked", tx);
//   });
//   it("Cannot transfer locked token", async () => {
//     randoToken = getAssociatedTokenAddressSync(mint, randomPerson);
//
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
//
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
//     let tx = await program.methods
//       .unlock()
//       .accounts({
//         token: tokenAccount,
//         mint,
//         programControl,
//         tokenRecord,
//         delegate: program.provider.publicKey!,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc({ commitment: "confirmed" });
//
//     let record = await program.account.tokenRecord.fetch(
//       tokenRecord,
//       "confirmed"
//     );
//     assert(
//       record.locked == 0x1,
//       "Token record should be locked after 1st invocation"
//     );
//
//     tx = await program.methods
//       .unlock()
//       .accounts({
//         token: tokenAccount,
//         mint,
//         programControl,
//         tokenRecord,
//         delegate: program.provider.publicKey!,
//         tokenProgram: TOKEN_PROGRAM_ID,
//       })
//       .rpc({ commitment: "confirmed" });
//
//     record = await program.account.tokenRecord.fetch(tokenRecord, "confirmed");
//     assert(
//       record.locked == 0x0,
//       "Token record should be unlocked after 2nd invocation"
//     );
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
//
//     let txid = await program.provider.sendAndConfirm(transaction, [], {
//       skipPreflight: true,
//       preflightCommitment: "confirmed",
//     });
//     console.log("\tTransferred token to normie: ", txid);
//   });
// });
//
