use candid::{CandidType, Decode, Encode, Nat};
use ic_agent::{
	agent::http_transport::ReqwestTransport, export::Principal, identity::Secp256k1Identity, Agent,
};
use icrc_ledger_types::icrc1::account::Account;
use icrc_ledger_types::icrc1::transfer::BlockIndex;
use icrc_ledger_types::icrc3::transactions::Transaction;
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
	let network = "https://ic0.app".to_string();

	let agent_identity = Secp256k1Identity::from_pem(
		"-----BEGIN EC PRIVATE KEY-----
		MHQCAQEEILLbYT5cEw65puvzNeCYvQUVej7Yp+0NyiIpAzhon+9moAcGBSuBBAAK
		oUQDQgAEz5laAZIQI7+44mYzRllX/b6ZzBXedT0VWYNd0cTxZXxLaB6lLXXeylfP
		HCrZI0tCmZfZZH9rsASN40otbb+/Kw==
		-----END EC PRIVATE KEY-----"
			.as_bytes(),
	)?;

	let agent = Agent::builder()
		.with_transport(ReqwestTransport::create(network).unwrap())
		.with_identity(agent_identity)
		.build()
		.map_err(|e| format!("{:?}", e))?;

	let canister_id = Principal::from_text("n5wcd-faaaa-aaaar-qaaea-cai".to_string())?;
	let owner = Principal::from_text(
		"hijd3-ferev-ybojm-nailk-pdk3t-l2h3o-h6cdy-mfynr-p3oen-d67mg-5ae".to_string(),
	)?;

	let reqst = GetAccountTransactionsArgs {
		account: Account {
			owner,
			subaccount: None,
		},
		start: None,
		max_results: Nat::from(50u64),
	};
	let arg = Encode!(&reqst)?;

	let ret = agent
		.update(&canister_id, "get_account_transactions")
		.with_arg(arg)
		.call_and_wait()
		.await?;

	let answer = Decode!(&ret, GetTransactionsResult)?;
	println!("{:?}", answer);

	Ok(())
}

#[derive(CandidType, Debug, candid::Deserialize, PartialEq, Eq)]
pub struct GetAccountTransactionsArgs {
	pub account: Account,
	// The txid of the last transaction seen by the client.
	// If None then the results will start from the most recent
	// txid.
	pub start: Option<BlockIndex>,
	// Maximum number of transactions to fetch.
	pub max_results: Nat,
}

pub type GetTransactionsResult = Result<GetTransactions, GetTransactionsErr>;

#[derive(CandidType, Debug, candid::Deserialize, PartialEq, Eq)]
pub struct GetTransactions {
	pub transactions: Vec<TransactionWithId>,
	// The txid of the oldest transaction the account has
	pub oldest_tx_id: Option<BlockIndex>,
}

#[derive(CandidType, Debug, candid::Deserialize, PartialEq, Eq)]
pub struct TransactionWithId {
	pub id: BlockIndex,
	pub transaction: Transaction,
}

#[derive(CandidType, Debug, candid::Deserialize, PartialEq, Eq)]
pub struct GetTransactionsErr {
	pub message: String,
}