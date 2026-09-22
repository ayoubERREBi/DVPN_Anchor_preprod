import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Dvpn } from "../target/types/dvpn";
import { assert } from "chai";

describe("dvpn", () => {
  // local provider config
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Dvpn as Program<Dvpn>;
  const user = provider.wallet;

  // node and client PDA derivation
  const [nodePda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("node"), user.publicKey.toBuffer()],
    program.programId
  );

  const [clientPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("client"), user.publicKey.toBuffer()],
    program.programId
  );

  const sessionId = "sess_001";
  const [sessionPda] = anchor.web3.PublicKey.findProgramAddressSync(
    [
      Buffer.from("session"),
      user.publicKey.toBuffer(),
      Buffer.from(sessionId),
    ],
    program.programId
  );

  it("Create Node", async () => {
    const wgKey = "wg_key_node_12345678901234567890123456789012";
    const priceHour = new anchor.BN(1000);

    await program.methods
      .createNode(wgKey, priceHour)
      .accounts({
        node: nodePda,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const nodeAccount = await program.account.node.fetch(nodePda);
    assert.equal(nodeAccount.wgKey, wgKey);
    assert.equal(nodeAccount.priceHour.toNumber(), 1000);
    assert.isTrue(nodeAccount.isActive);
  });

  it("Create client", async () => {
    const wgKey = "wg_key_client_12345678901234567890123456789";

    await program.methods
      .createClient(wgKey)
      .accounts({
        client: clientPda,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const clientAccount = await program.account.client.fetch(clientPda);
    assert.equal(clientAccount.wgKey, wgKey);
    assert.isFalse(clientAccount.inSession);
  });

  it("Start session", async () => {
    await program.methods
      .startSession(sessionId)
      .accounts({
        session: sessionPda,
        node: nodePda,
        client: clientPda,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const sessionAccount = await program.account.session.fetch(sessionPda);
    const nodeAccount = await program.account.node.fetch(nodePda);

    assert.equal(sessionAccount.sessionId, sessionId);
    assert.isTrue(nodeAccount.inSession);
  });

  it("End session", async () => {
    await program.methods
      .stopSession()
      .accounts({
        session: sessionPda,
        client: clientPda,
        node: nodePda,
        user: user.publicKey,
      })
      .rpc();

    const sessionAccount = await program.account.session.fetch(sessionPda);
    const nodeAccount = await program.account.node.fetch(nodePda);

    assert.isAbove(sessionAccount.endTime.toNumber(), 0);
    assert.isFalse(nodeAccount.inSession);
  });
});