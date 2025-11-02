/**
 * Initialize program config on-chain.
 * - A = provider wallet (from)
 * - B = CLI argument (to)
 * - threshold = second arg in lamports (defaults to 0.1 SOL = 100_000_000 lamports)
 *
 * Usage:
 *   npx ts-node scripts/init.ts B_PUBKEY [thresholdLamports]
 */
import * as anchor from "@coral-xyz/anchor";
import {PublicKey, SystemProgram} from "@solana/web3.js";
import idl from "../target/idl/conditional_transfer.json" assert { type: "json" };

(async () => {
  try {
    const [, , toArg, thresholdArg] = process.argv;
    if (!toArg) {
      console.error("Usage: npx ts-node scripts/init.ts B_PUBKEY [thresholdLamports]");
      process.exit(1);
    }

    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const programId = new PublicKey((idl as any).address ?? process.env.PROGRAM_ID!);
    const program = new anchor.Program(idl as anchor.Idl, programId, provider);

    // PDA for singleton config
    const [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );

    const from = provider.wallet.publicKey; // A is the provider wallet
    const to = new PublicKey(toArg);
    const thresholdLamports = thresholdArg ? BigInt(thresholdArg) : 100_000_000n; // 0.1 SOL

    console.log("Program ID   :", program.programId.toBase58());
    console.log("Authority    :", provider.wallet.publicKey.toBase58());
    console.log("From (A)     :", from.toBase58());
    console.log("To (B)       :", to.toBase58());
    console.log("Threshold    :", thresholdLamports.toString(), "lamports");
    console.log("Config PDA   :", configPda.toBase58());

    const txSig = await program.methods
      .initialize(from, to, new anchor.BN(thresholdLamports.toString()))
      .accounts({
        authority: provider.wallet.publicKey,
        config: configPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Initialize transaction signature:", txSig);
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
})();
