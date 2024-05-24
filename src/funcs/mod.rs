use redis::Commands;
use redis::{Connection, RedisResult};
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::dialogue::{
        serializer::{Bincode, Json},
        ErasedStorage, RedisStorage, Storage,
    },
    prelude::*,
    utils::command::BotCommands,
};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Question {
    pub question: String,
    pub id: i64,
    pub tg_id: String,
}
type QuestStorage = std::sync::Arc<ErasedStorage<Question>>;

pub async fn load_questions(quantity: u8) {
    // TODO: Write all useful functions for fethcing questions and handling them from Redis
    // WARN: Be careful with ownership and return results. Don't forget to check them. Avoid unsafe
    // code
}
pub fn upload_question(
    con: &mut Connection,
    question: Question,
) -> Result<(), Box<dyn std::error::Error>> {
    let serialized = serde_json::to_string(&question).unwrap();
    let _: () = con.set(question.tg_id, serialized)?;
    Ok(())
}

pub fn connect_to_db(url: &str) -> redis::Client {
    redis::Client::open(url).unwrap()
}
