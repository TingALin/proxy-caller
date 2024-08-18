use candid::{Decode, Encode, Nat};
use ic_agent::{
	agent::http_transport::ReqwestTransport, export::Principal, identity::Secp256k1Identity, Agent,
};
use std::error::Error;
use icrc_ledger_types::icrc3::transactions::{GetTransactionsRequest, GetTransactionsResponse};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
	let network = "https://ic0.app".to_string();

	let agent_identity = Secp256k1Identity::from_pem(
		"-----BEGIN EC PRIVATE KEY-----
		MHQCAQEEILLbYT5cEw65puvzNeCYvQUVej7Yp+0NyiIpAzhon+9moAcGBSuBBAAK
		oUQDQgAEz5laAZIQI7+44mYzRllX/b6ZzBXedT0VWYNd0cTxZXxLaB6lLXXeylfP
		HCrZI0tCmZfZZH9rsASN40otbb+/Kw==
		-----END EC PRIVATE KEY-----".as_bytes(),
	)?;

	let agent = Agent::builder()
		.with_transport(ReqwestTransport::create(network).unwrap())
		.with_identity(agent_identity)
		.build()
		.map_err(|e| format!("{:?}", e))?;

	let canister_id = Principal::from_text("mxzaz-hqaaa-aaaar-qaada-cai".to_string())?;

	let reqst = GetTransactionsRequest{start: Nat::from(1_732_000u64), length: Nat::from(2u64)};
	let arg = Encode!(&reqst)?;

	let ret = agent
	.update(&canister_id, "get_transactions")
	.with_arg(arg)
	.call_and_wait()
	.await?;

	let answer = Decode!(&ret, GetTransactionsResponse)?;
	// hijd3-ferev-ybojm-nailk-pdk3t-l2h3o-h6cdy-mfynr-p3oen-d67mg-5ae
	let proxy_account = Principal::from_text("lrf2i-zba54-pygwt-tbi75-zvlz4-7gfhh-ylcrq-2zh73-6brgn-45jy5-cae".to_string())?;

	// use for loop if there are multiple accounts
	for tx in answer.transactions {
		if let Some(t) = tx.transfer {
			if t.to.owner == proxy_account{
				// TO CALL
				println!("{:?}", t.to.owner);
			}
		}
	}

	Ok(())
}