/**
 * Send SOL from A → B THROUGH the program if amount ≥ threshold.
 * The signer must be A (the `from` in config).
 *
 * Usage:
 *   npx ts-node scripts/send.ts AMOUNT_SOL
 *
 * Example:
 *   npx ts-node scripts/send.ts 0.25
 */
import * as anchor from "@coral-xyz/anchor";
import {LAMPORTS_PER_SOL, PublicKey, SystemProgram} from "@solana/web3.js";
import idl from "../target/idl/conditional_transfer.json" assert { type: "json" };

(async () => {
  try {
    const [, , amountSolArg] = process.argv;
    if (!amountSolArg) {
      console.error("Usage: npx ts-node scripts/send.ts AMOUNT_SOL");
      process.exit(1);
    }
    const amountSol = Number(amountSolArg);
    if (!Number.isFinite(amountSol) || amountSol <= 0) {
      throw new Error("AMOUNT_SOL must be a positive number");
    }
    const lamports = BigInt(Math.floor(amountSol * LAMPORTS_PER_SOL));

    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const programId = new PublicKey((idl as any).address ?? process.env.PROGRAM_ID!);
    const program = new anchor.Program(idl as anchor.Idl, programId, provider);

    const [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    // Fetch config and provide helpful client-side check
    const cfg = await program.account.config.fetch(configPda);
    const from = provider.wallet.publicKey;
    const to = new PublicKey(cfg.to);
    const threshold: bigint = BigInt(cfg.thresholdLamports?.toString?.() ?? cfg.thresholdLamports);

    console.log("Program ID :", program.programId.toBase58());
    console.log("Config PDA :", configPda.toBase58());
    console.log("From (A)   :", from.toBase58());
    console.log("To (B)     :", to.toBase58());
    console.log("Threshold  :", threshold.toString(), "lamports");
    console.log("Amount     :", lamports.toString(), "lamports");

    if (lamports < threshold) {
      console.error("Amount is below the configured threshold; required ≥ threshold.");
      process.exit(1);
    }

    const txSig = await program.methods
      .sendIfOverThreshold(new anchor.BN(lamports.toString())) // on-chain enforces ≥ threshold
      .accounts({
        config: configPda,
        from,                 // must sign and match config.from
        to,                   // must equal config.to
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Send tx signature:", txSig);
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
})();
