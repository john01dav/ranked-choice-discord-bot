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

    pub async fn set_vote(&self, user: u64, votes: Vec<u32>) -> sqlx::Result<()>{
        let mut transaction = self.pool.begin().await?;

        //first, remove existing vote if any
        sqlx::query!("DELETE FROM votes WHERE user=?", user).execute(&mut transaction).await?;

        //now, add new votes
        for i in 0..votes.len(){
            sqlx::query!("INSERT INTO votes(user, option, choice_number) VALUES(?, ?, ?)", user, votes[i], (i+1) as u32)
                .execute(&mut transaction)
                .await?;
        }

        transaction.commit().await?;

        Ok(())
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