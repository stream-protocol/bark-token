import {
    Connection,
    Keypair,
    SystemProgram,
    Transaction,
    PublicKey,
    sendAndConfirmTransaction,
    clusterApiUrl,
  } from "@solana/web3.js";
  import {
    TOKEN_2022_PROGRAM_ID,
    createAccount,
    createAssociatedTokenAccount,
    createInitializeMintInstruction,
    createInitializeTransferFeeConfigInstruction,
    getMintLen,
    mintTo,
    transferCheckedWithFee,
    unpackAccount,
    withdrawWithheldTokensFromAccounts,
    ExtensionType,
  } from "@solana/spl-token";
  
  // Constants
  const CLUSTER_API_URL = clusterApiUrl("devnet");
  const FEE_ACCOUNT_PUBLIC_KEY = new PublicKey("FEESaM9hVHPQE1tcvH3tvP6VtGfa9gz9y5qmqd4Fi2fv");
  const payer = pg.wallet.keypair;
  const mintAuthority = payer.publicKey;
  const transferFeeConfigAuthority = pg.wallet.keypair;
  const withdrawWithheldAuthority = pg.wallet.keypair;
  const feeBasisPoints = 300; // 3%
  const maxFeeBasisPoints = 800; // 8%
  const mintLen = getMintLen([ExtensionType.TransferFeeConfig]);
  
  // Initialize connection to Solana cluster
  const connection = new Connection(CLUSTER_API_URL, 'confirmed');
  
  // Configuration
  const config = {
    clusterUrl: clusterApiUrl("devnet"),
    feeBasisPoints: 300,
    maxFee: BigInt(800),
    mintAmount: 20_000_000_000_000n,
    transferAmount: BigInt(10_0000),
  };
  
  // Utility function for BigInt arithmetic
  const calculateFee = (amount: bigint, basisPoints: number, maxFee: bigint): bigint => {
    const fee = (amount * BigInt(basisPoints)) / BigInt(10000);
    return fee > maxFee ? maxFee : fee;
  };
  
  // Function to create a new account and return its address
  const createNewAccount = async (space: number, lamports: number): Promise<Keypair> => {
    const keypair = Keypair.generate();
    const createAccountInstruction = SystemProgram.createAccount({
      fromPubkey: payer.publicKey,
      newAccountPubkey: keypair.publicKey,
      space,
      lamports,
      programId: TOKEN_2022_PROGRAM_ID,
    });
  
    const transaction = new Transaction().add(createAccountInstruction);
    await sendAndConfirmTransaction(connection, transaction, [payer, keypair]);
    return keypair;
  };
  
  // Function to mint BARK tokens
  const mintTokens = async (
    mint: Keypair,
    destinationTokenAccount: Keypair,
    mintAuthority: Keypair,
    amount: bigint
  ): Promise<string> => {
    const transactionSignature = await mintTo(
      connection,
      payer,
      mint.publicKey,
      destinationTokenAccount.publicKey,
      mintAuthority.publicKey,
      amount,
      undefined,
      undefined,
      TOKEN_2022_PROGRAM_ID
    );
    return transactionSignature;
  };
  
  // Function to transfer BARK tokens with a fee
  const transferTokensWithFee = async (
    sourceTokenAccount: Keypair,
    mint: Keypair,
    destinationTokenAccount: Keypair,
    transferAmount: bigint,
    decimals: number,
    feeBasisPoints: number,
    maxFeeBasisPoints: number
  ): Promise<string> => {
    try {
      const transferFee = calculateFee(transferAmount, feeBasisPoints, maxFeeBasisPoints);
      const actualTransferFee = transferFee > maxFeeBasisPoints ? maxFeeBasisPoints : transferFee;
      const totalAmount = transferAmount + actualTransferFee;
  
      const transactionSignature = await transferCheckedWithFee(
        connection,
        payer,
        sourceTokenAccount.publicKey,
        mint.publicKey,
        destinationTokenAccount.publicKey,
        payer.publicKey,
        totalAmount,
        decimals,
        actualTransferFee,
        undefined,
        undefined,
        TOKEN_2022_PROGRAM_ID
      );
  
      return transactionSignature;
    } catch (error) {
      console.error("Transfer with fee failed:", error);
      throw new Error(`Transfer with fee failed: ${error.message}`);
    }
  };
  
  // Function to burn BARK tokens
  const burnTokens = async (mintAccount: Keypair, amount: bigint): Promise<void> => {
    try {
      const burnAddress = new PublicKey("burneaBWzG1hbV9uMLsnjYTDJ8H5WgeoFhe5tM4eUzX");
  
      const burnTransaction = new Transaction().add(
        transferTokensWithFee(
          mintAccount,
          mintAccount,
          burnAddress,
          amount,
          decimals,
          0, // No fee for burning
          0
        )
      );
  
      await sendAndConfirmTransaction(connection, burnTransaction, [payer]);
  
      console.log(`Burned ${amount} BARK tokens`);
    } catch (error) {
      console.error("Burn BARK tokens failed:", error);
      throw new Error(`Burn BARK tokens failed: ${error.message}`);
    }
  };
  
  // Function to display Solana Explorer URL
  const displaySolanaExplorerUrl = (publicKey: PublicKey): void => {
    console.log("Solana Explorer URL:", getExplorerUrl(publicKey));
  };
  
  // Main logic
  try {
    // Step 1: Create Mint BARK Account
    const mintAccount = await createNewAccount(mintLen, lamports);
    console.log("Mint BARK Account created:", mintAccount.publicKey.toBase58());
  
    // Step 2: Initialize TransferFeeConfig Extension and Mint BARK Account data
    const initTransaction = new Transaction()
      .add(
        createInitializeTransferFeeConfigInstruction(
          mintAccount.publicKey,
          transferFeeConfigAuthority.publicKey,
          withdrawWithheldAuthority.publicKey,
          feeBasisPoints,
          maxFeeBasisPoints,
          TOKEN_2022_PROGRAM_ID
        ),
        createInitializeMintInstruction(
          mintAccount.publicKey,
          decimals,
          mintAuthority,
          updateAuthority.publicKey, // Replace with actual update authority
          TOKEN_2022_PROGRAM_ID
        )
      );
  
    await sendAndConfirmTransaction(connection, initTransaction, [payer]);
    console.log("Mint BARK Account initialized with TransferFeeConfig");
  
    // Step 4: Create Accounts
    const sourceTokenAccount = await createAssociatedTokenAccount(
      payer.publicKey,
      payer.publicKey,
      mintAccount.publicKey
    );
  
    const destinationTokenAccount = await createAssociatedTokenAccount(
      Keypair.generate().publicKey,
      payer.publicKey,
      mintAccount.publicKey
    );
  
    console.log("Accounts created for wallet and a random keypair");
  
    // Step 5: Mint to sourceTokenAccount
    const mintAmount = BigInt(20_000_000_000);
    await mintTokens(mintAccount, sourceTokenAccount, mintAuthority, mintAmount);
  
    console.log("Minted to sourceTokenAccount");
  
    // Step 6: Transfer with fee
    const transferAmount = BigInt(1000_00);
    const feeCharged = await transferTokensWithFee(
      sourceTokenAccount,
      mintAccount,
      destinationTokenAccount,
      transferAmount,
      decimals,
      feeBasisPoints,
      maxFeeBasisPoints
    );
  
    console.log("Transferred with fee");
  
    // Step 7: Burn tokens - 2% quarterly on the 15th day
    const currentDate = new Date();
    const currentDay = currentDate.getDate();
    if (currentDay === 15) {
      const burnAmount = (mintAmount * BigInt(2)) / BigInt(100);
      await burnTokens(mintAccount, burnAmount);
    } else {
      console.log("Not the 15th day of the quarter, skipping burning");
    }
  
    // Step 8: Withdraw withheld fees
    const accountsToWithdrawFrom: PublicKey[] = [];
    const allAccounts = await connection.getProgramAccounts(TOKEN_2022_PROGRAM_ID, {
      commitment: "confirmed",
      filters: [{ memcmp: { offset: 0, bytes: mintAccount.publicKey.toBytes() } }],
    });
  
    for (const accountInfo of allAccounts) {
      const account = unpackAccount(accountInfo.pubkey, accountInfo.account, TOKEN_2022_PROGRAM_ID);
      const transferFeeAmount = getTransferFeeAmount(account);
  
      if (transferFeeAmount !== null && transferFeeAmount.withheldAmount > 0) {
        accountsToWithdrawFrom.push(accountInfo.pubkey);
      }
    }
  
    if (accountsToWithdrawFrom.length > 0) {
      await withdrawWithheldTokensFromAccounts(
        connection,
        payer,
        mintAccount.publicKey,
        destinationTokenAccount,
        withdrawWithheldAuthority,
        accountsToWithdrawFrom,
        TOKEN_2022_PROGRAM_ID
      );
  
      console.log("Withdrawn fees from Accounts");
    } else {
      console.log("No fees to withdraw");
    }
  
    // Step 9: Display Solana Explorer URL
    displaySolanaExplorerUrl(mintAccount.publicKey);
  } catch (error) {
    console.error("Transaction failed:", error.message);
  }
  
  // Function to get Solana Explorer URL
  function getExplorerUrl(publicKey: PublicKey): string {
    return `https://explorer.solana.com/address/${publicKey.toBase58()}?cluster=devnet`;
  }  