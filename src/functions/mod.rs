use crate::misc;
use teloxide::{
    dispatching::{
        dialogue::{
            serializer::{Bincode, Json},
            ErasedStorage, InMemStorage, RedisStorage, Storage,
        },
        UpdateFilterExt,
    },
    prelude::*,
};
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub type CurrentDialogue = Dialogue<misc::State, InMemStorage<misc::State>>;

pub async fn answer(bot: Bot, msg: Message) -> HandlerResult {
    //
    Ok(())
}
