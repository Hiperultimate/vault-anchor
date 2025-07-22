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

describe("staking-anchor", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.StakingAnchor as Program<StakingAnchor>;
  const main_user = anchor.web3.Keypair.generate();

  it("Is initialized!", async () => {
    const user_account = provider.wallet.publicKey;
    await logAddressBalance(user_account, provider);
    // const [_, bump] = anchor.web3.PublicKey.findProgramAddressSync(
    //   [Buffer.from("vault"), user_account.toBuffer()],
    //   program.programId
    // );
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });

  it("Deposit SOL", async () => {
    const user_account = provider.wallet.publicKey;
    const [userPda, _bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user_account.toBuffer()],
      program.programId
    );
    const tx = await program.methods
      .deposit(new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL))
      .accounts({ userAccount: user_account }) // why do we not need a user_pda here?
      .rpc();
    console.log("Your transaction signature", tx);

    await logAddressBalance(user_account, provider);

    // get balance in PDA (should be 1 SOL)
    console.log("PDA Balance :");
    const userPDABalance = await logAddressBalance(userPda, provider);

    expect(userPDABalance).greaterThanOrEqual(1 * anchor.web3.LAMPORTS_PER_SOL);
  });

  it("Withdraw SOL from PDA", async () => {
    const user_account = provider.wallet.publicKey;
    const [user_pda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), user_account.toBuffer()],
      program.programId
    );

    const priorAccountBalance = await logAddressBalance(user_account, provider);
    // const priorPDABalance = await logAddressBalance(user_pda, provider);

    const tx = await program.methods
      .withdraw(new anchor.BN(1 * anchor.web3.LAMPORTS_PER_SOL))
      .accounts({
        userAccount: user_account,
      })
      .rpc();

    console.log("Your transaction signature : ", tx);

    const userAccountBalance = await logAddressBalance(user_account, provider);
    const userPDABalance = await logAddressBalance(user_pda, provider);

    expect(userPDABalance).lessThan(1 * anchor.web3.LAMPORTS_PER_SOL);
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
        userAccount: user_address,
      })
      .rpc();
    console.log("Your transaction signature : ", tx);

    const user_lamports_after = await provider.connection.getBalance(
      user_address
    );
    const pda_lamports_after = await provider.connection.getBalance(user_pda);


    console.log("User lamports before :", user_lamports_before);
    console.log("User lamports after :", user_lamports_after);
    console.log("PDA lamports before :", pda_lamports_before);
    console.log("PDA lamports after :", pda_lamports_after);

    // expect
  });
});
