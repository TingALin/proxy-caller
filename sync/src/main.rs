use anyhow::anyhow;
use candid::{Decode, Encode, Nat};
use dotenvy::dotenv;
use ic_agent::export::Principal;
use icrc_ledger_types::icrc3::transactions::{GetTransactionsRequest, GetTransactionsResponse};
use log::{info, LevelFilter};
use proxy_caller::dao::{Mutation, Query};
use proxy_caller::entity::caller;
use proxy_caller::utils::{with_canister, Database};
use sea_orm::DbConn;
use std::error::Error;
use log4rs::{
	append::console::ConsoleAppender,
	config::{Appender, Root},
};

pub async fn sync_tx(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_canister("CKBTC_CANISTER_ID", |agent, canister_id| async move {
		info!("{:?} syncing transactions ... ", chrono::Utc::now());

		let idx = Query::get_block_index(db).await?;

		let start_index = match idx {
			Some(idx) => Nat::from(idx.block_id.unwrap().parse::<u64>().unwrap()),
			None => {
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
				init_response.first_index
			}
		};

		let reqst = GetTransactionsRequest {
			start: start_index,
			length: Nat::from(2u8),
		};
		let arg = Encode!(&reqst)?;

		let ret = agent
			.update(&canister_id, "get_transactions")
			.with_arg(arg)
			.call_and_wait()
			.await?;

		if let Ok(tx_response) = Decode!(&ret, GetTransactionsResponse) {
			let proxy_account = vec![
				Principal::from_text("xmiu5-jqaaa-aaaag-qbz7q-cai".to_string())?,
				// Principal::from_text("xmiu5-jqaaa-aaaag-qbz7q-cai".to_string())?,
			];

			for acc in proxy_account {
				for tx in tx_response.clone().transactions {
					if let Some(t) = tx.transfer {
						if t.to.owner == acc {
							// TO CALL
							println!("{:?}", t.to.owner);
						}
					}
				}
			}

			let block_index = tx_response.first_index.to_string();
			let caller = caller::Model::new(block_index.clone());

			let _ = Mutation::save_block_index(db, caller);
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

	let _ = sync_tx(&db.connection).await;

	Ok(())
}
