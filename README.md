# Conditional SOL Transfer (≥ 0.1 SOL) — Solana / Anchor

A minimal Solana program (Anchor) that transfers SOL **from address A to address B if and only if the amount is _at least_ a configurable threshold**. The included scripts default that threshold to **0.1 SOL**.

> ⚠️ **Solana behavior**: Programs run only when explicitly invoked inside a transaction.  
> This contract does **not** auto-run on generic wallet-to-wallet transfers. It enforces the “≥ 0.1 SOL” rule **when you call it** and then performs the SOL transfer A → B via CPI to the System Program.

## What’s included
- ✅ Anchor program with a PDA config storing: `authority`, `from` (A), `to` (B), `thresholdLamports`
- ✅ `initialize(from, to, thresholdLamports)`
- ✅ `send_if_over_threshold(amountLamports)` → **now enforces _amount ≥ threshold_**
- ✅ Admin updates: `update_threshold(newThresholdLamports)` and `update_addresses(newFrom, newTo)`
- ✅ TypeScript scripts to initialize config and to send funds through the program
- ✅ Step-by-step instructions and inline code comments

---

## Prerequisites
- Rust & Cargo
- Solana CLI (`solana --version`)
- Node 18+ and npm or yarn
- Anchor CLI (`anchor --version`) — code targets modern Anchor (0.28+)

```bash
solana config set --url https://api.devnet.solana.com
```

## Quickstart

1. **Install deps & build**
   ```bash
   npm install
   anchor build
   ```

2. **Set your Program ID**
   - Generate or locate your program keypair:
     ```bash
     anchor keys list
     ```
   - Copy the **program id** and update it in **both**:
     - `programs/conditional_transfer/src/lib.rs` in `declare_id!(...)`
     - `Anchor.toml` under `[programs.devnet]` as `conditional_transfer = "<PROGRAM_ID>"`
   - Rebuild:
     ```bash
     anchor build
     ```

3. **Deploy to Devnet**
   ```bash
   anchor deploy
   ```

4. **Initialize config**
   - **A** is your current wallet (`~/.config/solana/id.json`).
   - **B** is the recipient pubkey (base58).
   - Threshold defaults to **0.1 SOL = 100,000,000 lamports** if not provided.
   ```bash
   npx ts-node scripts/init.ts B_PUBKEY [thresholdLamports]
   ```

5. **Send (A → B) via program (≥ threshold)**
   ```bash
   npx ts-node scripts/send.ts 0.25
   ```
   - The script will fetch the on-chain threshold and **fail fast** client-side if your amount is below it.
   - On-chain validation enforces **amount ≥ threshold** as well.

## Notes
- 1 SOL = 1,000,000,000 lamports. 0.1 SOL = 100,000,000 lamports.
- The **authority** set at `initialize` can update threshold or A/B addresses.
- For mainnet: change the cluster in `Anchor.toml`, adjust RPC/wallet, rebuild, and redeploy.

## Security tips
- Verify the **Program ID** and cluster prior to sending real funds.
- Consider a multisig for the authority in production.
- Keep the `from` key safe; the program requires **A** to sign the transfer call.

---

MIT License
