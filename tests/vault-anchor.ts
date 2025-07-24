import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StakingAnchor } from "../target/types/staking_anchor";
import { expect } from "chai";

async function logAddressBalance(
  pubKey: anchor.web3.PublicKey,
  provider: anchor.Provider
) {
  const lamports = await provider.connection.getBalance(pubKey);
  console.log(`Logging lamports for ${pubKey} : ${lamports}`);
  return lamports;
}

describe("vault-anchor", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.StakingAnchor as Program<StakingAnchor>;

  it("Is initialized!", async () => {
    const user_account = provider.wallet.publicKey;
    await logAddressBalance(user_account, provider);
    const tx = await program.methods.initialize().accounts({
      signer: user_account
    }).rpc();
    console.log("Your transaction signature", tx);
  });

  it("Deposit SOL", async () => {
    const user_account = provider.wallet.publicKey;
    const [vaultPda, _bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user_account.toBuffer()],
      program.programId
    );
    const tx = await program.methods
      .deposit(new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL))
      .accounts({ signer: user_account }) // why do we not need a user_pda here?
      .rpc();
    console.log("Your transaction signature", tx);

    await logAddressBalance(user_account, provider);

    // get balance in PDA (should be 1 SOL)
    console.log("PDA Balance :");
    const userPDABalance = await logAddressBalance(vaultPda, provider);

    expect(userPDABalance).greaterThanOrEqual(1 * anchor.web3.LAMPORTS_PER_SOL);
  });

  it("Withdraw SOL from PDA", async () => {
    const user_account = provider.wallet.publicKey;
    const [user_pda, _bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user_account.toBuffer()],
      program.programId
    );

    const priorAccountBalance = await logAddressBalance(user_account, provider);

    const tx = await program.methods
      .withdraw(new anchor.BN(500000000))
      .accounts({
        signer: user_account,
      })
      .rpc();

    console.log("Your transaction signature : ", tx);

    const userAccountBalance = await logAddressBalance(user_account, provider);
    const userPDABalance = await logAddressBalance(user_pda, provider);

    expect(userPDABalance).lessThanOrEqual(500000000);
    expect(userAccountBalance).greaterThan(priorAccountBalance);
  });

  it("Close the PDA and return the lamports to the user who made it", async () => {
    const user_address = provider.wallet.publicKey;
    const [user_pda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user_address.toBuffer()],
      program.programId
    );

    const user_lamports_before = await provider.connection.getBalance(
      user_address
    );
    const pda_lamports_before = await provider.connection.getBalance(user_pda);

    const tx = await program.methods
      .close()
      .accounts({
        signer: user_address,
      })
      .rpc();
    console.log("Your transaction signature : ", tx);

    const user_lamports_after = await provider.connection.getBalance(
      user_address
    );
    const pda_lamports_after = await provider.connection.getBalance(user_pda);

    expect(user_lamports_before).lessThan(user_lamports_after);
    expect(pda_lamports_before).greaterThan(pda_lamports_after);
    expect(pda_lamports_after).equal(0);
  });
});
