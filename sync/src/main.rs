use candid::{Decode, Encode, Nat};
use ic_agent::{
	agent::{self, http_transport::ReqwestTransport}, export::Principal, identity::Secp256k1Identity, Agent,
};
use icrc_ledger_types::icrc3::transactions::{GetTransactionsRequest, GetTransactionsResponse};
use anyhow::anyhow;
use dotenvy::dotenv;
use utils::{Database, with_canister};
mod utils;
use sea_orm::DbConn;
use std::error::Error;
use log::info;
mod dao;

pub async fn sync_tx(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_canister("CKBTC_CANISTER_ID", |agent, canister_id| async move {
		info!("{:?} syncing transactions ... ", chrono::Utc::now());

		let start_index = match{
		// start_index: read from the db
	// if none, read from the current.

	// let init_reqst = GetTransactionsRequest{start: Nat::from(0u64), length: Nat::from(0u64)};
	// let init_arg = Encode!(&init_reqst)?;
	// let init_ret = agent
	// .update(&canister_id, "get_transactions")
	// .with_arg(init_arg)
	// .call_and_wait()
	// .await?;
	// let init_answer = Decode!(&init_ret, GetTransactionsResponse)?;
	// let start_index  = init_answer.first_index;
	};

	let reqst = GetTransactionsRequest{start: start_index, length: Nat::from(2u64)};
	let arg = Encode!(&reqst)?;

	let ret = agent
	.update(&canister_id, "get_transactions")
	.with_arg(arg)
	.call_and_wait()
	.await?;

	let answer = Decode!(&ret, GetTransactionsResponse)?;
	// 只要get_transactions成功了就可以更新DB的block index了

	let proxy_account = vec![Principal::from_text("il25e-7ncru-p5jb5-zu6tn-wjetc-nmh5d-4aplx-qre2t-ww6gy-ahtzz-yae".to_string())?, Principal::from_text("ix5qj-xyaaa-aaaar-qahfa-cai".to_string())?];

	for acc in proxy_account {
		for tx in answer.clone().transactions {
			if let Some(t) = tx.transfer {
				if t.to.owner == acc{
					// TO CALL
					println!("{:?}", t.to.owner);
				}
			}
		}
	}
	Ok(())
	}).await
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
	dotenv().ok();

	let db_url = std::env::var("DATABASE_URL").map_err(|_| anyhow!("DATABASE_URL is not found"))?;
	let db = Database::new(db_url.clone()).await;

	let _ = sync_tx(&db.connection).await;
	Ok(())
}