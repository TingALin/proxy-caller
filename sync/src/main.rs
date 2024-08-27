use anyhow::anyhow;
use candid::{Decode, Encode, Nat};
use dotenvy::dotenv;
use ic_agent::export::Principal;
use ic_agent::Agent;
use icrc_ledger_types::icrc3::transactions::{
	GetTransactionsRequest, GetTransactionsResponse, Transaction,
};
use log::{info, LevelFilter};
use log4rs::{
	append::console::ConsoleAppender,
	config::{Appender, Root},
};
use proxy_caller::dao::{Mutation, Query};
use proxy_caller::entity::caller;
use proxy_caller::utils::{with_canister, Database};
use sea_orm::DbConn;
use std::error::Error;
use std::ops::Add;

pub const LENGTHPERBLOCK: u16 = 1000u16;

pub async fn get_latest_first_index(
	agent: Agent,
	canister_id: Principal,
) -> Result<Nat, Box<dyn Error>> {
	let init_reqst = GetTransactionsRequest {
		start: Nat::from(0u8),
		length: Nat::from(0u8),
	};

	let init_arg = Encode!(&init_reqst)?;

	let init_ret = agent
		.update(&canister_id, "get_transactions")
		.with_arg(init_arg)
		.call_and_wait()
		.await?;

	let init_response = Decode!(&init_ret, GetTransactionsResponse)?;

	Ok(init_response.first_index)
}

pub async fn sync_tx(request_index: u64, acc: Principal) -> Result<(), Box<dyn Error>> {
	with_canister(
		"CKBTC_ARCHIVE_CANISTER_ID",
		|agent, canister_id| async move {
			info!(
				"{:?} syncing the archive transaction ... ",
				chrono::Utc::now()
			);

			let arg = Encode!(&request_index)?;

			let ret = agent
				.update(&canister_id, "get_transaction")
				.with_arg(arg)
				.call_and_wait()
				.await?;

			let answer = Decode!(&ret, Option<Transaction>)?;
			if let Some(tx) = answer {
				if let Some(_transfer) = tx.transfer {
					// 有需要就转换
					if _transfer.to == acc.into() {
						// TOCALL
						println!("{:?}", _transfer.to);
					}
				}
			}
			Ok(())
		},
	)
	.await
}

// 不知first_index什么时候更新，实时性会很差方案2找方法：如log_length>= first_index+1000,first_index就可以改成最新
pub async fn sync_txs(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_canister("CKBTC_CANISTER_ID", |agent, canister_id| async move {
		info!("{:?} syncing transactions ... ", chrono::Utc::now());

		let idx = Query::get_latest_block_index(db).await?;
		let current_index = get_latest_first_index(agent.clone(), canister_id).await?;
		let start_index = match idx.clone() {
			Some(idx) => {
				if Nat::from(idx.first_index.add(1) as u64) == current_index.clone() {
					let _ = current_index.clone();
				} else if Nat::from(idx.first_index as u64) == current_index.clone() {
					let _ = current_index.clone();
				}
				Nat::from(idx.first_index as u64)
			}
			None => current_index.clone(),
		};

		// 直接返回，什么都不用做
		if Nat::from(idx.clone().unwrap().first_index as u64) == current_index.clone() {
			return Ok(());
		}

		let reqst = GetTransactionsRequest {
			start: start_index.clone(),
			length: Nat::from(LENGTHPERBLOCK),
		};
		let arg = Encode!(&reqst)?;

		let ret = agent
			.update(&canister_id, "get_transactions")
			.with_arg(arg)
			.call_and_wait()
			.await?;

		if let Ok(tx_response) = Decode!(&ret, GetTransactionsResponse) {
			// 环境变量或全局
			let proxy_account = vec![
				Principal::from_text("akhru-myaaa-aaaag-qcvna-cai".to_string())?,
				Principal::from_text("akhru-myaaa-aaaag-qcvna-cai".to_string())?,
			];

			let mut block_index = 0;

			for acc in proxy_account {
				if tx_response.transactions.len() as u16 == LENGTHPERBLOCK {
					for tx in tx_response.clone().transactions {
						if let Some(t) = tx.transfer {
							if t.to.owner == acc {
								// TO CALL
								println!("{:?}", t.to.owner);
							}
						}
					}
					block_index = tx_response
						.first_index
						.to_string()
						.replace("_", "")
						.parse::<i64>()?;
				} else if tx_response.transactions.len() == 0 {
					let start = start_index.clone().to_string().parse::<u64>()?;
					let end = start.add(LENGTHPERBLOCK as u64);
					for idx in start..end {
						let _ = sync_tx(idx, acc).await?;
					}
					block_index = end as i64;
				}
			}

			let caller = caller::Model::new(block_index);
			let updated_block_index = Mutation::save_block_index(db, caller).await?;
			info!("Updated block index: {:?}", updated_block_index.first_index);
		}
		Ok(())
	})
	.await
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
	dotenv().ok();

	let stdout = ConsoleAppender::builder().build();
	let config = log4rs::config::Config::builder()
		.appender(Appender::builder().build("stdout", Box::new(stdout)))
		.build(Root::builder().appender("stdout").build(LevelFilter::Info))
		.unwrap();
	log4rs::init_config(config).unwrap();

	let db_url = std::env::var("DATABASE_URL").map_err(|_| anyhow!("DATABASE_URL is not found"))?;

	let db = Database::new(db_url.clone()).await;

	let _ = sync_txs(&db.connection).await;

	Ok(())
}
