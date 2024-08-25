use crate::entity::caller;
use crate::entity::caller::Entity as Caller;
use log::info;
use sea_orm::sea_query::OnConflict;
use sea_orm::*;

pub struct Query;

impl Query {
	pub async fn get_block_index(db: &DbConn) -> Result<Option<caller::Model>, DbErr> {
		Caller::find_by_id(0_i16).one(db).await
	}
}

pub struct Mutation;

impl Mutation {
	pub async fn save_block_index(
		db: &DbConn,
		caller: caller::Model,
	) -> Result<caller::Model, DbErr> {
		let active_model: caller::ActiveModel = caller.clone().into();
		let on_conflict = OnConflict::column(caller::Column::Seq)
			.do_nothing()
			.to_owned();
		let insert_result = Caller::insert(active_model.clone())
			.on_conflict(on_conflict)
			.exec(db)
			.await;
		match insert_result {
			Ok(ret) => {
				info!("insert block index : {:?}", ret);
			}
			Err(_) => {
				info!("the block index already exited");

				let res = Caller::update(active_model)
					.filter(caller::Column::Seq.eq(caller.seq.to_owned()))
					.exec(db)
					.await
					.map(|caller| caller);
				info!("update block index : {:?}", res);
			}
		}
		Ok(caller::Model { ..caller })
	}
}
