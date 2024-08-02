use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, DateTime, Document},
    options::FindOptions,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Question {
    pub question: String,
    pub id: i64,
    pub tg_id: String,
    pub answered: bool,
    pub upload_time: DateTime,
}
pub async fn load_questions(
    collection: mongodb::Collection<Question>,
    res: &mut Vec<Question>,
) -> std::result::Result<(), mongodb::error::Error> {
    let filter = doc! {"answered": false};
    let filter_options = FindOptions::builder()
        .sort(doc! {"answered": 1, "upload_time": -1})
        .build();
    let mut doc_ptr = collection.find(filter, filter_options).await?;
    while let Some(quest) = doc_ptr.try_next().await? {
        res.push(quest);
    }
    Ok(())
}
pub async fn check_blacklist(
    collection: mongodb::Collection<Document>,
    tg_id: &String,
) -> std::result::Result<bool, mongodb::error::Error> {
    let filter = doc! {"tg_id": tg_id};
    let filter_options = FindOptions::builder().build();
    let mut doc_ptr = collection.find(filter, filter_options).await?;
    if let Some(_id) = doc_ptr.try_next().await? {
        return Ok(true);
    }
    Ok(false)
}
pub async fn upload_question(
    question: Question,
    collection: mongodb::Collection<Question>,
) -> Result<(), mongodb::error::Error> {
    let res = collection.insert_one(question, None).await?;
    println!("Inserted to collection with id: {:?}", res.inserted_id);
    Ok(())
}
