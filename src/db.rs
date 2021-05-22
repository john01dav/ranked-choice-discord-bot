use sqlx::MySqlPool;
use crate::db::entities::{Candidate, Vote};
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

    pub async fn get_nth_vote(&self, user: u64, n: u32) -> sqlx::Result<Option<u32>>{
        Ok(
            sqlx::query!("SELECT option FROM votes WHERE user=? AND choice_number=?", user, n)
                .fetch_optional(&self.pool)
                .await?
                .map(|r| r.option)
        )
    }

    pub async fn get_1st_votes(&self) -> sqlx::Result<Vec<Vote>>{
        Ok(
            sqlx::query_as!(Vote, "SELECT user, option FROM votes WHERE choice_number=1")
                .fetch_all(&self.pool)
                .await?
        )
    }

}

pub mod entities{

    pub struct Candidate{
        pub id: u32,
        pub name: String
    }

    pub struct Vote{
        pub user: u64,
        pub option: u32
    }

}

impl TypeMapKey for Db{
    type Value = Arc<Db>;
}