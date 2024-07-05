use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, DateTime, Document},
    options::FindOptions,
};
use serde::{Deserialize, Serialize};
use teloxide::{
    dispatching::dialogue::{
        serializer::{Bincode, Json},
        ErasedStorage, RedisStorage, Storage,
    },
    prelude::*,
    utils::command::BotCommands,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct Question {
    pub question: String,
    pub id: i64,
    pub tg_id: String,
    pub answered: bool,
    pub upload_time: DateTime,
}
type QuestStorage = std::sync::Arc<ErasedStorage<Question>>;

pub async fn load_questions(
    quantity: u8,
    collection: mongodb::Collection<Question>,
    res: &mut Vec<Question>,
) -> std::result::Result<(), mongodb::error::Error> {
    let filter = doc! {"answered": false};
    let filter_options = FindOptions::builder().sort(doc! {"answered": 1}).build();
    let mut doc_ptr = collection.find(filter, filter_options).await?;
    let mut cnt = 0;
    while let Some(quest) = doc_ptr.try_next().await? {
        if cnt == quantity {
            break;
        }
        res.push(quest);
        cnt += 1;
    }
    Ok(())
}
pub async fn upload_question(
    question: Question,
    collection: mongodb::Collection<Question>,
) -> Result<(), mongodb::error::Error> {
    collection.insert_one(question, None).await?;
    Ok(())
}
