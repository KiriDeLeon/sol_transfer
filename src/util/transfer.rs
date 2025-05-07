use anyhow::Result;
use std::sync::Arc;
use std::time::Instant;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, signature::Signer, system_instruction,
    transaction::Transaction,
};

use crate::util::get_key::parse_keypair_base58;
use crate::util::read_config::Config;
use futures::future::join_all;

pub async fn token_transfers(config: Config) -> Result<()> {
    let client = Arc::new(RpcClient::new_with_commitment(
        "https://api.mainnet-beta.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    ));

    let amount_sol = config.value;
    let amount_lamports = solana_sdk::native_token::sol_to_lamports(amount_sol);
    let to_list = config.to;
    let sender_list = config.sender;

    let tasks = to_list.into_iter().enumerate().map(|(i, to_str)| {
        let client = Arc::clone(&client);
        let from_str = sender_list[i % sender_list.len()].clone();

        async move {
            let start = Instant::now();
            let from_kp = parse_keypair_base58(&from_str)?;
            let to_kp = parse_keypair_base58(&to_str)?;
            let to_pubkey = to_kp.pubkey();
            let from_pubkey = from_kp.pubkey();

            let balance = client.get_balance(&from_pubkey).await?;
            if balance < amount_lamports {
                println!(
                    "Wallet sender: {} | Addressee: {} | Unsuccess: Not money ({} lamports, need {}) | Time: {:?}",
                    from_pubkey,
                    to_pubkey,
                    balance,
                    amount_lamports,
                    start.elapsed()
                );
                return Ok::<_, anyhow::Error>(());
            }

            let instr = system_instruction::transfer(
                &from_pubkey,
                &to_pubkey,
                amount_lamports,
            );

            let blockhash = client.get_latest_blockhash().await?;
            let tx = Transaction::new_signed_with_payer(
                &[instr],
                Some(&from_pubkey),
                &[&from_kp],
                blockhash,
            );

            match client.send_and_confirm_transaction(&tx).await {
                Ok(sig) => {
                    println!(
                        "Wallet sender: {} | Addressee: {} | Success: TX: {} | Time: {:?}",
                        from_pubkey,
                        to_pubkey,
                        sig,
                        start.elapsed()
                    );
                }
                Err(e) => {
                    println!(
                        "Wallet sender: {} | Addressee: {} | Unsuccess: {} | Time: {:?}",
                        from_pubkey,
                        to_pubkey,
                        e,
                        start.elapsed()
                    );
                }
            }

            Ok(())
        }
    });

    join_all(tasks).await;

    Ok(())
}
