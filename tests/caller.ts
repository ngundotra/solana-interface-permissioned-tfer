import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {
  PublicKey,
  SystemProgram,
  Keypair,
  Transaction,
  AccountMeta,
  TransactionInstruction,
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
import { TwicePermissioned } from "../target/types/twice_permissioned";
import { IdlInstruction, idlAddress } from "@coral-xyz/anchor/dist/cjs/idl";
import { AccountsGeneric } from "@coral-xyz/anchor/dist/cjs/program/accounts-resolver";

export type LockContext = {
  token: anchor.Address;
  mint: anchor.Address;
  delegate: anchor.Address;
  payer: anchor.Address;
  tokenProgram: anchor.Address;
  permProgram: anchor.Address;
};
export type UnlockContext = {
  token: anchor.Address;
  mint: anchor.Address;
  delegate: anchor.Address;
  payer: anchor.Address;
  tokenProgram: anchor.Address;
  permProgram: anchor.Address;
};

async function resolveRemainingAccounts(
  provider: anchor.Provider,
  instructionName: string,
  accounts: AccountsGeneric
): Promise<{ accounts: AccountMeta[]; resolved: 0 }> {
  let targetProgramAddress = accounts["permProgram"] as PublicKey;
  let idlAddy = await idlAddress(targetProgramAddress);
  console.log("idlAddy", idlAddy.toBase58());
  let targetProgram = await Program.at(targetProgramAddress, provider);

  let ctx: Object;
  if (instructionName === "lock") {
    ctx = {
      token: accounts["token"],
      mint: accounts["mint"],
      delegate: accounts["delegate"],
      payer: accounts["payer"],
      token_program: accounts["token_program"],
    };
  } else if (instructionName === "unlock") {
    ctx = {
      token: accounts["token"],
      mint: accounts["mint"],
      delegate: accounts["delegate"],
      token_program: accounts["token_program"],
    };
  }

  let builder = targetProgram.methods.lock().accounts(ctx as any);
  const ix = await builder.instruction();
  let additionalAccounts = ix.keys.slice(Object.keys(ctx).length);
  return { accounts: additionalAccounts, resolved: 0 };
}

describe("caller-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  let caller = anchor.workspace.Caller as Program<Caller>;

  describe("permissioned token", () => {
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
      tokenRecord = PublicKey.findProgramAddressSync(
        [tokenAccount.toBuffer(), Buffer.from("token_record")],
        program.programId
      )[0];

      const lockCtx: LockContext = {
        token: tokenAccount,
        mint,
        delegate: program.provider.publicKey!,
        payer: program.provider.publicKey!,
        tokenProgram: TOKEN_PROGRAM_ID,
        permProgram: program.programId,
      };

      const builder = caller.methods.lock().accounts(lockCtx);
      let keys = await builder.pubkeys();
      let { accounts: remainingAccounts } = await resolveRemainingAccounts(
        program.provider,
        "lock",
        keys
      );
      const tx = builder
        .remainingAccounts(remainingAccounts)
        .rpc({ skipPreflight: true });
      console.log("\tLocked", tx);
    });
    it("Cannot transfer locked token", async () => {
      randoToken = getAssociatedTokenAddressSync(mint, randomPerson);

      let transaction = new Transaction().add(
        createAssociatedTokenAccountInstruction(
          payer,
          randoToken,
          randomPerson,
          mint,
          TOKEN_PROGRAM_ID,
          ASSOCIATED_TOKEN_PROGRAM_ID
        ),
        createTransferInstruction(tokenAccount, randoToken, payer, 1, [])
      );

      try {
        let txid = await program.provider.sendAndConfirm(transaction, [], {
          skipPreflight: true,
          commitment: "confirmed",
        });
        throw Error("Should not be able to transfer locked token");
      } catch (e) {
        console.log("\tSuccessfully failed to transfer locked token:", e);
      }
    });
    it("Can unlock user token account", async () => {
      const builder = caller.methods.unlock().accounts({
        token: tokenAccount,
        mint,
        delegate: program.provider.publicKey!,
        tokenProgram: TOKEN_PROGRAM_ID,
        permProgram: program.programId,
      });
      let keys = await builder.pubkeys();
      let { accounts: remainingAccounts } = await resolveRemainingAccounts(
        program.provider,
        "unlock",
        keys
      );
      let tx = await builder
        .remainingAccounts(remainingAccounts)
        .rpc({ skipPreflight: true });

      console.log("\tUnlocked", tx);
    });

    it("Can transfer unlocked token", async () => {
      let transaction = new Transaction().add(
        createAssociatedTokenAccountInstruction(
          payer,
          randoToken,
          randomPerson,
          mint,
          TOKEN_PROGRAM_ID,
          ASSOCIATED_TOKEN_PROGRAM_ID
        ),
        createTransferInstruction(tokenAccount, randoToken, payer, 1, [])
      );

      let txid = await program.provider.sendAndConfirm(transaction, [], {
        skipPreflight: true,
      });
      console.log("\tTransferred token to normie: ", txid);
    });
  });
  describe("Twice permissioned token", () => {
    const program = anchor.workspace
      .TwicePermissioned as Program<TwicePermissioned>;

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
      tokenRecord = PublicKey.findProgramAddressSync(
        [tokenAccount.toBuffer(), Buffer.from("token_record")],
        program.programId
      )[0];

      let lockCtx: LockContext = {
        token: tokenAccount,
        mint,
        delegate: program.provider.publicKey!,
        payer: program.provider.publicKey!,
        tokenProgram: TOKEN_PROGRAM_ID,
        permProgram: program.programId,
      };

      const builder = caller.methods.lock().accounts(lockCtx);
      let keys = await builder.pubkeys();
      let { accounts: remainingAccounts } = await resolveRemainingAccounts(
        program.provider,
        "lock",
        keys
      );
      const transaction = await builder
        .remainingAccounts(remainingAccounts)
        .transaction();

      let tx = await program.provider.sendAndConfirm(transaction, [], {
        skipPreflight: true,
      });
      tx = await program.provider.sendAndConfirm(transaction, [], {
        skipPreflight: true,
      });
      console.log("\tLocked", tx);
    });
    it("Cannot transfer locked token", async () => {
      randoToken = getAssociatedTokenAddressSync(mint, randomPerson);

      let transaction = new Transaction().add(
        createAssociatedTokenAccountInstruction(
          payer,
          randoToken,
          randomPerson,
          mint,
          TOKEN_PROGRAM_ID,
          ASSOCIATED_TOKEN_PROGRAM_ID
        ),
        createTransferInstruction(tokenAccount, randoToken, payer, 1, [])
      );

      try {
        let txid = await program.provider.sendAndConfirm(transaction, [], {
          skipPreflight: true,
          preflightCommitment: "confirmed",
        });
        throw Error("Should not be able to transfer locked token");
      } catch (e) {
        console.log("\tSuccessfully failed to transfer locked token:", e);
      }
    });
    it("Can unlock user token account", async () => {
      const builder = caller.methods.unlock().accounts({
        token: tokenAccount,
        mint,
        delegate: program.provider.publicKey!,
        tokenProgram: TOKEN_PROGRAM_ID,
        permProgram: program.programId,
      });
      let keys = await builder.pubkeys();
      let { accounts: remainingAccounts } = await resolveRemainingAccounts(
        program.provider,
        "unlock",
        keys
      );
      const transaction = await builder
        .remainingAccounts(remainingAccounts)
        .transaction();
      let tx = await program.provider.sendAndConfirm(transaction, [], {
        skipPreflight: true,
      });
      tx = await program.provider.sendAndConfirm(transaction, [], {
        skipPreflight: true,
      });
      console.log("\tUnlocked", tx);
    });

    it("Can transfer unlocked token", async () => {
      let transaction = new Transaction().add(
        createAssociatedTokenAccountInstruction(
          payer,
          randoToken,
          randomPerson,
          mint,
          TOKEN_PROGRAM_ID,
          ASSOCIATED_TOKEN_PROGRAM_ID
        ),
        createTransferInstruction(tokenAccount, randoToken, payer, 1, [])
      );

      let txid = await program.provider.sendAndConfirm(transaction, [], {
        skipPreflight: true,
        preflightCommitment: "confirmed",
      });
      console.log("\tTransferred token to normie: ", txid);
    });
  });
});
