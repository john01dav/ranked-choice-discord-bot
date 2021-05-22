use sqlx::MySqlPool;
use crate::db::entities::Candidate;
use serenity::prelude::TypeMapKey;
use std::sync::Arc;

pub struct Db{
    pool: MySqlPool
}

impl Db{

    pub async fn new(url: &str) -> sqlx::Result<Self>{
        Ok(
            Self{
                pool: MySqlPool::connect(url).await?
            }
        )
    }

    pub async fn list_candidates(&self) -> sqlx::Result<Vec<Candidate>>{
        Ok(
            //sqlx::query_as!(Candidate, "SELECT id, name FROM candidates").fetch_all(&self.pool).await?
            sqlx::query_as!(Candidate, "SELECT id, name FROM candidates").fetch_all(&self.pool).await?
        )
    }

}

pub mod entities{

    pub struct Candidate{
        pub id: u32,
        pub name: String
    }

}

impl TypeMapKey for Db{
    type Value = Arc<Db>;
}