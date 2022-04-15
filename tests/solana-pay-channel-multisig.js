const anchor = require('@project-serum/anchor');
const { BN } = require('bn.js');
const { clusterApiUrl, Connection, PublicKey } = require("@solana/web3.js");
const connection = new Connection(clusterApiUrl('devnet'), 'confirmed');


describe('solana_payment_channel_multisig', async () => {

  // Configure the client to use the devnet cluster.
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);
  
  const multiSigWallet= anchor.web3.Keypair.generate();
  const treasuryWallet = anchor.web3.Keypair.generate();
  const alice = anchor.web3.Keypair.generate();
  const bob = anchor.web3.Keypair.generate();
  const alicePaymentUser = anchor.web3.Keypair.generate();
  const bobPaymentUser = anchor.web3.Keypair.generate();
  const program = anchor.workspace.SolanaPayChannelMultisig;

  it('Creates alice user', async () => {
 
    const signature = await connection.requestAirdrop(alice.publicKey, 2000000000);
    await connection.confirmTransaction(signature);
    console.log(signature);
    let balanceOfAlice = await connection.getBalance(alice.publicKey);
    console.log("Alice Received the Airdrop :", balanceOfAlice);

    await program.rpc.createPaymentUser("Alice",{
      accounts: {
        paymentUser: alicePaymentUser.publicKey,
        user: alice.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [alice,alicePaymentUser]
    });
  });

  it('Creates bob user', async () => {

    const signature = await connection.requestAirdrop(bob.publicKey, 2000000000);
    await connection.confirmTransaction(signature);
    let balanceOfBob = await connection.getBalance(bob.publicKey);
    console.log("Bob Received the Airdrop :", balanceOfBob);

    await program.rpc.createPaymentUser("Bob",{
      accounts: {
        paymentUser: bobPaymentUser.publicKey,
        user: bob.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [bob,bobPaymentUser]
    });
  });

  it('Creates multi-sig wallet', async () => {
    
    const signature = await connection.requestAirdrop(treasuryWallet.publicKey, 1000000000);
    await connection.confirmTransaction(signature);
    let balanceOfTreasury = await connection.getBalance(treasuryWallet.publicKey);
    console.log("Bob Received the Airdrop :", balanceOfTreasury);

    let user_1_contribution = new BN(1000000)
    let user_2_contribution = new BN(2000000)
    
    await program.rpc.createMultisigWallet(user_1_contribution,user_2_contribution,{
      accounts: {
        multisigWallet: multiSigWallet.publicKey,
        owner: treasuryWallet.publicKey,
        user1: alice.publicKey,
        user2: bob.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [alice,bob,treasuryWallet,multiSigWallet]
    });

  });

  it('Updates wallet contributions', async () => {
    let user_1_contribution = new BN(1500000)
    let user_2_contribution = new BN(1500000)
    await program.rpc.updateBalance(user_1_contribution,user_2_contribution,{
      accounts: {
        multisigWallet: multiSigWallet.publicKey,
        user1: alice.publicKey,
        user2: bob.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [alice,bob]
    });
    const multiSigWalletObj = await program.account.multiSigWallet.fetch(multiSigWallet.publicKey);
    console.log(multiSigWalletObj)
  });

  it('withdraws wallet money', async () => {
    await program.rpc.closeChannel({
      accounts: {
        multisigWallet: multiSigWallet.publicKey,
        owner: treasuryWallet.publicKey,
        signer: treasuryWallet.publicKey,
        user1: alice.publicKey,
        user2: bob.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [treasuryWallet]
    });
    const multiSigWalletObj = await program.account.multiSigWallet.fetch(multiSigWallet.publicKey);
    console.log(multiSigWalletObj)
  });
        
});