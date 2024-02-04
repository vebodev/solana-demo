use borsh::{BorshDeserialize, BorshSerialize};
use chrono;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
};
#[allow(unused_imports)]
use std::rc::Rc;

#[derive(Debug, PartialEq, BorshDeserialize, BorshSerialize)]
struct ChangeDetail {
    amount: u32,
    from: Pubkey,
    to: Pubkey,
    timestamp: i64,
}

#[derive(Debug, PartialEq, BorshSerialize, BorshDeserialize)]
/// Custom data structure to store u32 value
struct TokenData {
    amount: u32,
    history: Vec<ChangeDetail>,
}

#[derive(Debug, PartialEq, BorshSerialize, BorshDeserialize)]
/// All custom program instructions
pub enum ProgramInstruction {
    // Set token to an account by Adminstrator ONLY
    SetTokenToAccount{ amount: u32 },

    // Save token by account owner 
    SaveTokenByOwner { amount: u32 },

    // Withdraw token by account owner
    WithdrawTokenByOwner { amount: u32 },

    // Check blanace token
    CheckBalanceToken,
}

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("program_id: {:?}", program_id);
    // decode instruction data
    let instruction: ProgramInstruction = ProgramInstruction::try_from_slice(instruction_data)
        .map_err(|err| {
            msg!("Error: Failed to deserialize instruction data: {:?}", err);
            ProgramError::InvalidInstructionData
        })?;

    match instruction {
        ProgramInstruction::SetTokenToAccount { amount } => {
            msg!("Instruction: SetTokenToAccount");
            set_token_to_account(program_id, accounts, amount)
        }
        ProgramInstruction::SaveTokenByOwner { amount } => {
            msg!("Instruction: SaveTokenByOwner");
            save_token_by_owner(program_id, accounts, amount)
        }
        ProgramInstruction::WithdrawTokenByOwner { amount } => {
            msg!("Instruction: WithdrawTokenByOwner");
            withdraw_token_by_owner(program_id, accounts, amount)
        }
        ProgramInstruction::CheckBalanceToken => {
            msg!("Instruction: CheckBalanceToken");
            check_balance_token(program_id, accounts)
        }
    }
}

/**
 * Set token to an account by Adminstrator ONLY
 */
fn set_token_to_account(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u32,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let admin_account = next_account_info(account_iter)?;

    dbg!("admin_account: {:?}", admin_account);

    // validate admin account
    if !admin_account.is_signer {
        msg!("Error: Admin account does not sign the transaction");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let data_account = next_account_info(account_iter)?;

    dbg!("data_account: {:?}", data_account);

    // Check if data_account is writable
    if !data_account.is_writable {
        msg!("Error: Data account is not writable");
        return Err(ProgramError::InvalidArgument);
    }


    // if data len is zero, then create a new token_data
    if data_account.data.borrow().len() == 0 {
        let history = ChangeDetail {
            amount: amount,
            from: *admin_account.key,
            to: *data_account.key,
            timestamp: chrono::Utc::now().timestamp(),
        };
        let token_data = TokenData {
            amount: amount,
            history: vec![history],
        };

        dbg!("empty token_data: {:?}", &token_data);

        // serialize data
        let serialized_data = token_data.try_to_vec()?;

        // Copy the serialized data into the data account
        data_account.try_borrow_mut_data()?.copy_from_slice(&serialized_data);
    } else {
        // Borsh serialize amount
        let mut token_data = TokenData::try_from_slice(&data_account.data.borrow())?;

        // set amount
        token_data.amount = amount;

        // append a history
        token_data.history.push(ChangeDetail {
            amount: amount,
            from: *admin_account.key,
            to: *data_account.key,
            timestamp: chrono::Utc::now().timestamp(),
        });

        dbg!("nonempty token_data: {:?}", &token_data);

        // serialize data
        let serialized_data = token_data.try_to_vec()?;

        // Copy the serialized data into the data account
        data_account.try_borrow_mut_data()?.copy_from_slice(&serialized_data);

    }

    Ok(())
}

/**
 * Save token by account owner 
 */
fn save_token_by_owner(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u32,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner_account = next_account_info(account_iter)?;

    // get token of owner account
    let mut token_data = TokenData::try_from_slice(&owner_account.data.borrow())?;
    
    // add amount to token
    token_data.amount += amount;

    // append history
    token_data.history.push(ChangeDetail {
        amount: amount,
        from: *owner_account.key,
        to: *owner_account.key,
        timestamp: chrono::Utc::now().timestamp(),
    });

    // put back token to owner account
    let serialized_data = token_data.try_to_vec()?;
    owner_account.try_borrow_mut_data()?.copy_from_slice(&serialized_data);

    Ok(())
}

fn withdraw_token_by_owner(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u32,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner_account = next_account_info(account_iter)?;

    // get token of owner account
    let mut token_data = TokenData::try_from_slice(&owner_account.data.borrow())?;

    if token_data.amount < amount {
        msg!("Error: Not enough token to withdraw");
        return Err(ProgramError::InvalidArgument);
    }

    // substract amount
    token_data.amount -= amount;
    // append history
    token_data.history.push(ChangeDetail {
        amount: amount,
        from: *owner_account.key,
        to: *owner_account.key,
        timestamp: chrono::Utc::now().timestamp(),
    });

    let serialized_data = token_data.try_to_vec()?;
    owner_account.try_borrow_mut_data()?.copy_from_slice(&serialized_data);

    Ok(())
}

fn check_balance_token(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner_account = next_account_info(account_iter)?;

    let _token_data = TokenData::try_from_slice(&owner_account.data.borrow())?;

    Ok(())
}
#[cfg(test)]
mod test {
    use super::*;
    use solana_program_test::*;
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use solana_sdk::{message::Message, signature::{Keypair, Signer}, system_instruction, system_transaction, transaction::Transaction};

    #[tokio::test]
    async fn test_set_token_to_account() {
        // Set up a mock environment
        let program_id = Pubkey::new_unique();
        let (mut banks_client, payer, recent_blockhash) = ProgramTest::new(
            "solana_demo", // Replace with the name of your program
            program_id,
            processor!(process_instruction),
        )
        .start()
        .await;

        // Create a new account for the admin
        let admin_keypair = Keypair::new();
        let admin_account = Keypair::new();
        let admin_account_data = TokenData {
            amount: 42,
            history: vec![ChangeDetail {
                amount: 42,
                from: Pubkey::new_unique(),
                to: Pubkey::new_unique(),
                timestamp: 0,
            }],
        };
        create_account(
            &mut banks_client,
            &payer,
            &admin_account,
            program_id,
            admin_account_data,
        )
        .await;

        // Create a new account for the data with initial data
        let data_keypair = Keypair::new();
        let data_account = Keypair::new();
        let data_account_data = TokenData {
            amount: 0,
            history: Vec::new(),
        };
        create_account(
            &mut banks_client,
            &payer,
            &data_account,
            program_id,
            data_account_data,
        )
        .await;

        // Craft the transaction
        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &ProgramInstruction::SetTokenToAccount {
                    amount: 100,
                }
                .try_to_vec()
                .unwrap(),
                vec![
                    AccountMeta::new_readonly(admin_account.pubkey(), true),
                    AccountMeta::new(data_account.pubkey(), false),
                ],
            )],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[&payer, &admin_keypair], recent_blockhash);

        // Execute the transaction
        banks_client.process_transaction(transaction).await.unwrap();

        // Fetch the updated data account
        let updated_data_account = banks_client
            .get_account(data_keypair.pubkey())
            .await
            .expect("Failed to get data account");

        // Deserialize the data account to verify the changes
        let updated_token_data: TokenData =
            TokenData::try_from_slice(&updated_data_account.unwrap().data)
                .expect("Failed to deserialize data account");

        // Add your assertions based on the expected behavior
        assert_eq!(updated_token_data.amount, 100);
        assert_eq!(updated_token_data.history.len(), 1);
        // ...
    }

    async fn create_account(
        banks_client: &mut BanksClient,
        payer: &Keypair,
        account: &Keypair,
        program_id: Pubkey,
        initial_data: TokenData,
    ) {
        let recent_blockhash = banks_client.get_recent_blockhash().await.unwrap();
        let ix = system_instruction::create_account(
            &payer.pubkey(),
            &account.pubkey(),
            1, // replace with your desired account space
            1024*1024, // maximum space allowed
            &program_id,
        );
        
        // airdrop
        let airdrop_tx = system_transaction::transfer(&payer, &account.pubkey(), 1000, recent_blockhash,);
        banks_client.process_transaction(airdrop_tx).await.unwrap();
        

        let message = Message::new(&[ix], Some(&payer.pubkey()));
        let mut transaction = Transaction::new_unsigned(message);
        transaction.sign(&[payer, account], recent_blockhash);
        banks_client.process_transaction(transaction).await.unwrap();

        let mut transaction = Transaction::new_with_payer(
            &[Instruction::new_with_bytes(
                program_id,
                &ProgramInstruction::SetTokenToAccount {
                    amount: 100,
                }
                .try_to_vec()
                .unwrap(),
                vec![
                    AccountMeta::new_readonly(account.pubkey(), false),
                ],
            )],
            Some(&payer.pubkey()),
        );
        transaction.sign(&[payer, account], recent_blockhash);

        banks_client.process_transaction(transaction).await.unwrap();
    }
}
