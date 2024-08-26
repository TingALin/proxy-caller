use anyhow::anyhow;
use candid::{Decode, Encode, Nat};
use dotenvy::dotenv;
use ic_agent::export::Principal;
use ic_agent::Agent;
use icrc_ledger_types::icrc3::transactions::{GetTransactionsRequest, GetTransactionsResponse};
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

// 方案1不知first_index什么时候更新，实时性会很差
// 方案2找方法：如log_length>= first_index+1000,first_index就可以改成最新
pub async fn sync_tx(db: &DbConn) -> Result<(), Box<dyn Error>> {
	with_canister("CKBTC_CANISTER_ID", |agent, canister_id| async move {
		info!("{:?} syncing transactions ... ", chrono::Utc::now());

		let idx = Query::get_block_index(db).await?;
		let current_index = get_latest_first_index(agent.clone(), canister_id).await?;

		let start_index = match idx {
			// 如果拿到即时的INDEX是现idx.block_id+1， 可用即时的INDEX, 不然就是现存的INDEX
			Some(idx) => {
				if idx.block_id.unwrap().parse::<u64>().unwrap().add(1) == current_index {
					current_index;
				}
				Nat::from(idx.block_id.unwrap().parse::<u64>().unwrap())
			}
			None => current_index,
		};

		// 如果拿到即时的INDEX是现idx.block_id，而数据库上现存LENGTH是LENGTHPERBLOCK就什么也不用做
		// 如果有问题就改成 &&, 确保这个后什么都不会执行
		if start_index == current_index {
			if let Some(_) = idx.unwrap().length {
				return Ok(());
			}
		}

		let reqst = GetTransactionsRequest {
			start: start_index,
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
				// Principal::from_text("
				// lrf2i-zba54-pygwt-tbi75-zvlz4-7gfhh-ylcrq-2zh73-6brgn-45jy5-cae".to_string())?,
				Principal::from_text("akhru-myaaa-aaaag-qcvna-cai".to_string())?,
			];

			//tx_response.transactions.len() == 1000u16 就执行以下，
			// 如果tx_response.transactions.len() < 1000u16 就什么都不要做
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
				} else if tx_response.transactions.len() == 0 {
					for i in start_index..current_index {
						// 只能在https://dashboard.internetcomputer.org/canister/nbsys-saaaa-aaaar-qaaga-cai 单个拿数据
						//需要看返回结构，可用if let Some(t) = tx.transfer
					}
				}
			}
			//不管怎样，first_index一定是会最新，所以不用改
			let block_index = tx_response.first_index.to_string().replace("_", "");
			// 可能要换成每个BLOCK都有一行
			let caller = caller::Model::new(block_index.clone(), Some(LENGTHPERBLOCK as i64));

			let updated_block_index = Mutation::save_block_index(db, caller).await?;
			info!("Updated block index: {:?}", updated_block_index.block_id);
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
