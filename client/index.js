const { Account, clusterApiUrl, Connection, Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, Transaction, sendAndConfirmTransaction } = require('@solana/web3.js');
const { TOKEN_PROGRAM_ID, Token, AccountLayout, MintLayout, TokenAmount } = require('@solana/spl-token');

//const url = clusterApiUrl('devnet'); // Use 'mainnet-beta' for the mainnet
const url = "http://localhost:8899";


async function createAccount(connection, payer, programId, initialData) {
  // Create a new account for the data
  const account = new Account();

  // Get the rent fee
  const rent = await connection.getMinimumBalanceForRentExemption(initialData.length);

  // Transfer lamports to the new account
  const transferIx = SystemProgram.transfer({
    fromPubkey: payer.publicKey,
    toPubkey: account.publicKey,
    lamports: rent,
  });

  let transaction = new Transaction().add(transferIx);
  await sendAndConfirmTransaction(connection, transaction, [payer, account]);

  // Create the account
  const createAccountIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: account.publicKey,
    lamports: rent,
    space: initialData.length,
    programId,
  });

  transaction = new Transaction().add(createAccountIx);
  await sendAndConfirmTransaction(connection, transaction, [payer, account]);

  return account;
}

// async function sendAndConfirmTransaction(connection, transaction, signers) {
//   const signature = await connection.sendTransaction(transaction, signers);
//   await connection.confirmTransaction(signature);
//   return signature;
// }

(async () => {
  // Connect to the local Solana devnet
  const connection = new Connection(url, 'confirmed');

// Payer's secret key
// const payerSecretKey = [86,114,146,108,104,139,39,148,157,246,159,83,31,164,245,8,142,187,134,191,219,164,219,8,148,154,166,230,40,247,141,166,230,182,176,154,152,140,128,88,176,186,148,234,214,186,11,137,197,131,142,141,166,196,5,81,175,241,50,229,27,117,147,156];

// Create a new Account object
// const payer = new Account(payerSecretKey);
const { Keypair } = require('@solana/web3.js');
const payer = Keypair.generate();

  // airdrop to payer
  // Amount to airdrop, in SOL
const amount = 1;

// Airdrop
coconnection.requestAirdrop(payer.publicKey, amount * LAMPORTS_PER_SOL)
  .then((transactionSignature) => {
    console.log(`Airdropped ${amount} SOL to ${payer.publicKey.toString()}`);
    console.log(`Transaction Signature: ${transactionSignature}`);
  });
  
  // Program ID (replace with your own)
  const programId = new PublicKey('2HYuZEeKt7qicGKqM1q1CoLd7r7HGjCo8N5KM7mnF1FS');

  // Initial data for the TokenData structure
  const initialData = Buffer.from([0, 0, 0, 0]); // Adjust as needed

  // Create an account for the data
  const dataAccount = await createAccount(connection, payer, programId, initialData);

  // Call the SetTokenToAccount instruction
  const instructionData = Buffer.from([1, 0, 0, 0]); // Adjust as needed
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: false },
      { pubkey: dataAccount.publicKey, isSigner: false, isWritable: true },
    ],
    programId,
    data: instructionData,
  });

  const transaction = new Transaction().add(instruction);
  await sendAndConfirmTransaction(connection, transaction, [payer]);

  console.log('Transaction confirmed!');
})();
