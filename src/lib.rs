


#[cfg(test)]
mod tests {
    use solana_client::rpc_client::RpcClient;
    use solana_program::{pubkey::Pubkey, system_instruction::transfer, hash::hash};
    use solana_sdk::{instruction::{AccountMeta, Instruction}, message::Message, signature::{read_keypair_file, Keypair, Signer}, system_program, transaction::Transaction};
    use std::str::FromStr;
    use bs58; use std::io::{self, BufRead};

    const RPC_URL: &str = "https://api.devnet.solana.com";

    
    #[test]
    fn keygen(){
        let kp = Keypair::new();
        println!("New solana wallet: {}", kp.pubkey().to_string());
        println!("");
        println!("{:?}", kp.to_bytes())
    }

    #[test]
    fn base58_to_wallet() {

        println!("Input your private key as base58 string");

        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();

        println!("your wallet file format is: ");
        let wallet = bs58::decode(base58).into_vec().unwrap();

        println!("{:?}", wallet);
    }


    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a json byte array");

        let stdin = io::stdin();

        // parse will convert each string in the iterator into a u8 but it still remains in the iterator. So we use collect to convert them all to a vector of u8.
        let wallet = stdin.lock().lines().next().unwrap().unwrap().trim_start_matches("[").trim_end_matches("]").split(",").map(|s| s.trim().parse::<u8>().unwrap()).collect::<Vec<u8>>();

        let base58 = bs58::encode(wallet).into_string();
        println!("your base58 encoded private key is: {:?}", base58)
    }

    #[test]
    fn airdrop(){
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");
        
        let client = RpcClient::new(RPC_URL);

        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000) {
            Ok(sig) => {
                println!("Successful transaction:");
                println!("https://explorer.solana.com/tx/{}?cluster=devnet", sig);
            }

            Err(err)=> {
                println!("Airdrop failed: {}", err);
            }            
        }


    }

    #[test]
    fn transfer_sol(){
        let keypair = read_keypair_file("dev-wallet.json").expect("Couldn't find wallet file");

        let pubkey = keypair.pubkey();

        let messge_bytes = b"I verify my solana keypair";
        let sig = keypair.sign_message(messge_bytes);
        let sig_hashed = hash(sig.as_ref());

        match sig.verify(&pubkey.to_bytes(), &sig_hashed.to_bytes()) {
            true => println!("signature verified"),
            false => println!("verification failed")
        }

        // destination
        let to_pubkey = Pubkey::from_str("2Q5nroNBxBugWxs481ijAopoPDi2c3nxWwHAYMKzV988").unwrap();

        let rpc_client = RpcClient::new(RPC_URL);

        let recent_blockhash = rpc_client.get_latest_blockhash().expect("Failed to get the latest blockhash");

        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), 
            &to_pubkey, 
            1_000_0000)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash
        );

        let signature = rpc_client.send_and_confirm_transaction(&transaction).expect("Failed to send and confirm transaction");

        println!("Successfully created transaction: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);

        
        // empty out local wallet
        let balance = rpc_client.get_balance(&keypair.pubkey()).expect("Unable to get balance");

        // mock tx to calc fee
        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(),
            &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash);

        // calc fee
        let fee = rpc_client.get_fee_for_message(&message).expect("Unable to calculate fee");

        // final tx
        let transaction = Transaction::new_signed_with_payer(&[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)], Some(&keypair.pubkey()), &vec![&keypair], recent_blockhash);

        let signature = rpc_client.send_and_confirm_transaction(&transaction).expect("final transaction failed");

        println!("Successfully transferred the entire balance: https://explorer.solana.com/tx/{}/?cluster=devnet", signature)

    }


    #[test]
    fn interact_with_turbine() {
        let rpc_client = RpcClient::new(RPC_URL);

        let signer = read_keypair_file("turbine-wallet.json").expect("Couldn't find wallet");

        let mint = Keypair::new();

        let turbin3_prereq_program = Pubkey::from_str("TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM").unwrap();

        let collection = Pubkey::from_str("5ebsp5RChCGK7ssRZMVMufgVZhd2kFbNaotcZ5UvytN2").unwrap();

        let mpl_core_program = Pubkey::from_str("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d").unwrap();

        let system_program = system_program::id();

        // deriving PDA using some inputs
        let signer_pubkey = signer.pubkey();
        let seeds = &[b"prereqs", signer_pubkey.as_ref()];

        let (prereq_pda, _bump) = Pubkey::find_program_address(seeds, &turbin3_prereq_program);

        // deriving authrotiy
        let authseeds = &[b"collection", collection.as_ref()];
        let (authority, _bump) = Pubkey::find_program_address(authseeds, &turbin3_prereq_program);
        
        let data = vec![77, 124, 82, 163, 21, 133, 181, 206];


        let accounts = vec![
            AccountMeta::new(signer.pubkey(), true),
            AccountMeta::new(prereq_pda, false),
            AccountMeta::new(mint.pubkey(), true),
            AccountMeta::new(collection, false),
            AccountMeta::new_readonly(authority, false),
            AccountMeta::new_readonly(mpl_core_program, false),
            AccountMeta::new_readonly(system_program, false)
        ];

        let blockhash = rpc_client.get_latest_blockhash().expect("failed to get the latest blockhash");

        let instruction = Instruction{
            program_id: turbin3_prereq_program,
            accounts,
            data
        };

        let transaction = Transaction::new_signed_with_payer(&[instruction], Some(&signer.pubkey()), &[&signer, &mint], blockhash);

        let signature = rpc_client.send_and_confirm_transaction(&transaction).expect("Failed to send transaction");

        println!("Successfully created transaction: https://explorer.solana.com/tx/{}/?cluster=devnet", signature);

    }
}





