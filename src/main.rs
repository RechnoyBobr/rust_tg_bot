// TODO: GLOBAL:
// Show state should receieve vector of questions.

use funcs::{load_questions, upload_question, Question};
use mongodb::{
    bson::{doc, to_document, DateTime},
    options::UpdateOptions,
    Client, Collection,
};
use teloxide::{
    dispatching::{
        dialogue::{self, InMemStorage, Storage},
        HandlerExt, UpdateFilterExt, UpdateHandler,
    },
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, UpdateKind},
    utils::command::BotCommands,
};
mod funcs;

type MyStorage = std::sync::Arc<InMemStorage<State>>;
type SimpleDialouge = Dialogue<State, InMemStorage<State>>;
#[derive(Clone)]
struct ConfigParameters {
    bot_owner: UserId,
    chat_id: ChatId,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum UserCommands {
    Start,
    Ask,
}
const BOT_NAME: &str = "julaila_ask_bot";
#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    StartQuest,
    Show {
        array: Vec<Question>,
        cur: usize,
    },
    ReceiveQuest {
        question: Question,
    },
    ReceiveAns {
        question: Question,
    },
    BogusState,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum AdminCommands {
    Start,
    Show,
    Previous,
    Next,
    Answer,
}
#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot..");
    dotenv::from_filename(".env.local").ok();
    let mongodb_uri = dotenv::var("MONGODB_URI").expect(
        "MONGODB_URI should be specified in the file .env.local in the root of the project",
    );
    let token: String =
        dotenv::var("TELOXIDE_TOKEN").expect("TELOXIDE_TOKEN should be specified in the .env file");
    let mongodb_client = Client::with_uri_str(mongodb_uri).await.unwrap();
    let db = mongodb_client.database("rust_bot");
    let collection: Collection<Question> = db.collection("questions");
    let params: ConfigParameters = ConfigParameters {
        // TODO: get UserId from environment variable. (Don't forget about Docker). Use docker
        // secrets.
        bot_owner: UserId(
            dotenv::var("ADMIN_ID")
                .expect("ADMIN_ID should be specified in the .env.local file")
                .parse::<u64>()
                .expect("ADMIN_ID should be u64 type"),
        ),
        chat_id: ChatId(
            dotenv::var("ADMIN_ID")
                .expect("ADMIN_ID should be specified in the .env.local file")
                .parse::<i64>()
                .expect("ADMIN_ID should be i64 type"),
        ),
    };
    let bot = Bot::new(token);
    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![
            params,
            InMemStorage::<State>::new(),
            collection
        ])
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
fn schema() -> UpdateHandler<teloxide::RequestError> {
    let callback_query_handler = Update::filter_callback_query()
        .branch(
            dptree::filter(|cfg: ConfigParameters, query: CallbackQuery| {
                query.from.id == cfg.bot_owner
            })
            .endpoint(admin_command_handler),
        )
        .branch(
            dptree::filter(|query: CallbackQuery| query.from.id != UserId(0))
                .endpoint(user_command_handler),
        );
    let handler = Update::filter_message()
        .branch(dptree::case![State::StartQuest].endpoint(receive_question))
        .branch(dptree::case![State::ReceiveAns { question }].endpoint(handle_answer))
        .branch(
            dptree::filter(|cfg: ConfigParameters, upd: Update| {
                if let UpdateKind::Message(msg) = upd.kind {
                    msg.from()
                        .map(|user| user.id == cfg.bot_owner)
                        .unwrap_or_default()
                } else {
                    false
                }
            })
            .filter_command::<AdminCommands>()
            .endpoint(admin_start),
        )
        .branch(
            dptree::filter(|upd: Update| {
                if let UpdateKind::Message(msg) = upd.kind {
                    msg.from()
                        .map(|user| user.id != UserId(0))
                        .unwrap_or_default()
                } else {
                    false
                }
            })
            .filter_command::<UserCommands>()
            .endpoint(user_start),
        );
    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(handler)
        .branch(callback_query_handler)
}
async fn user_command_handler(
    bot: Bot,
    dialogue: SimpleDialouge,
    // cmd: UserCommands,
    q: CallbackQuery,
) -> Result<(), teloxide::RequestError> {
    let cmd = UserCommands::parse(&q.data.unwrap(), BOT_NAME).unwrap();
    let text = match cmd {
        UserCommands::Start => "Приветствую вас в своём мега-крутом ботеее!".to_owned(),
        UserCommands::Ask => {
            dialogue.update(State::StartQuest).await.unwrap();
            "Задайте ваш вопрос".to_owned()
        }
    };
    bot.send_message(dialogue.chat_id(), text)
        .reply_markup(make_user_keyboard(dialogue.get().await.unwrap().unwrap()))
        .await?;
    Ok(())
}

async fn user_start(
    bot: Bot,
    msg: Message,
    dialogue: SimpleDialouge,
    cmd: UserCommands,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        UserCommands::Start => String::from("Добро пожаловать"),
        _ => String::from("Ты как это сделал😳"),
    };
    bot.send_message(msg.chat.id, text)
        .reply_markup(make_user_keyboard(dialogue.get().await.unwrap().unwrap()))
        .await?;
    Ok(())
}
async fn admin_start(
    bot: Bot,
    msg: Message,
    dialogue: SimpleDialouge,
    cmd: AdminCommands,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        AdminCommands::Start => String::from("Добро пожаловать"),
        _ => String::from("В данном боте поддерживается только команда /start, всё остальное осуществляется с помощью кнопок"),
    };
    bot.send_message(msg.chat.id, text)
        .reply_markup(make_keyboard(dialogue.get().await.unwrap().unwrap()))
        .await?;
    Ok(())
}
async fn admin_command_handler(
    bot: Bot,
    dialogue: SimpleDialouge,
    client: mongodb::Collection<Question>,
    q: CallbackQuery,
) -> Result<(), teloxide::RequestError> {
    let cmd = AdminCommands::parse(&q.data.unwrap(), BOT_NAME).unwrap();
    let text = match cmd {
        AdminCommands::Start => String::from("Добро пожаловать"),
        AdminCommands::Show => {
            // TODO: load array of messages
            let mut v: Vec<Question> = vec![];
            let load_result = load_questions(client, &mut v).await;
            if v.is_empty() {
                String::from("На данный момент нет вопросов, на которые можно ответить")
            } else {
                match load_result {
                    Ok(()) => {
                        let _ = dialogue.update(State::Show { array: v, cur: 0 }).await;
                        if let State::Show { array, cur } = dialogue.get().await.unwrap().unwrap() {
                            format_question(&array, cur, array.len())
                        } else {
                            String::from("Невозможно")
                        }
                    }
                    Err(e) => {
                        println!("{:?}", e.to_string());
                        String::from(
                            "Произошла ошибка при загрузке вопросов. Обратитесь к разработчику",
                        )
                    }
                }
            }
        }
        AdminCommands::Next => {
            if let State::Show { array, cur } = dialogue.get().await.unwrap().unwrap() {
                let ret_val = format_question(&array, (cur + 1) % array.len(), array.len());
                let _ = dialogue
                    .update(State::Show {
                        cur: (cur + 1) % array.len(),
                        array,
                    })
                    .await;
                ret_val
            } else {
                String::from("Сначала надо кнопочку show нажать")
            }
        }
        AdminCommands::Answer => {
            let cur_state = dialogue.get().await.unwrap().unwrap();
            if let State::Show { array, cur } = cur_state {
                match dialogue
                    .update(State::ReceiveAns {
                        question: array[cur].clone(),
                    })
                    .await
                {
                    Ok(()) => String::from("Напишите ответ в сообщении ниже, он будет отправлен"),
                    Err(_e) => String::from("Стейт не обновился"),
                }
            } else if let State::ReceiveQuest { question } = dialogue.get().await.unwrap().unwrap()
            {
                dialogue.update(State::ReceiveAns { question }).await;
                String::from("Напишите Ответ в сообщении ниже, он будет отправлен")
            } else {
                String::from("Нет вопроса, на который нужно ответить")
            }
        }
        AdminCommands::Previous => {
            if let State::Show { array, cur } = dialogue.get().await.unwrap().unwrap() {
                let ret_val = format_question(&array, (cur - 1) % array.len(), array.len());
                let _ = dialogue
                    .update(State::Show {
                        cur: (cur - 1) % array.len(),
                        array,
                    })
                    .await;
                ret_val
            } else {
                String::from("Сначала надо кнопочку show нажать")
            }
        }
    };
    bot.send_message(dialogue.chat_id(), text)
        .reply_markup(make_keyboard(dialogue.get().await.unwrap().unwrap()))
        .send()
        .await?;
    Ok(())
}
fn make_keyboard(state: State) -> InlineKeyboardMarkup {
    let kb = match state {
        State::Start => {
            vec![vec![InlineKeyboardButton::callback(
                "Показать новые вопросы",
                "/show",
            )]]
        }
        State::Show { array: _, cur: _ } => {
            vec![vec![
                InlineKeyboardButton::callback("⬅️", "/previous"),
                InlineKeyboardButton::callback("Ответить", "/answer"),
                InlineKeyboardButton::callback("➡️", "/next"),
            ]]
        }
        State::ReceiveQuest { question: _ } => {
            vec![vec![
                InlineKeyboardButton::callback("Ответить", "/answer"),
                InlineKeyboardButton::callback("Просмотреть все вопросы", "/show"),
            ]]
        }
        _ => {
            vec![vec![]]
        }
    };
    InlineKeyboardMarkup::new(kb)
}

fn make_user_keyboard(state: State) -> InlineKeyboardMarkup {
    let kb = match state {
        State::Start => {
            vec![vec![InlineKeyboardButton::callback(
                "Задать вопрос",
                "/ask",
            )]]
        }
        _ => {
            vec![vec![]]
        }
    };
    InlineKeyboardMarkup::new(kb)
}
async fn handle_answer(
    bot: Bot,
    dialogue: SimpleDialouge,
    msg: Message,
    col: Collection<Question>,
) -> Result<(), teloxide::RequestError> {
    let state = match dialogue.get().await {
        Ok(s) => s.unwrap(),
        Err(_e) => {
            println!("There is an error, probably because of wrong state");
            State::BogusState
        }
    };
    let to_maybe = match state {
        State::ReceiveAns { question } => Some(question),
        _ => None,
    };
    let quest = match to_maybe {
        Some(question) => question,
        None => {
            println!("There is an error because of the wrong state");
            return Ok(());
        }
    };
    dialogue.update(State::Start).await;
    bot.send_message(msg.chat.id, "Ваш ответ отправлен")
        .reply_markup(make_keyboard(dialogue.get().await.unwrap().unwrap()))
        .await?;
    bot.send_message(ChatId(quest.id), "Вам пришёл ответ на ваше сообщение:")
        .await?;
    bot.send_message(ChatId(quest.id), msg.text().unwrap())
        .await?;
    let options = UpdateOptions::builder().build();
    match col
        .update_one(
            to_document(&quest).unwrap(),
            doc! {"$set": doc!{"answered": true}},
            options,
        )
        .await
    {
        Ok(_update_result) => match dialogue.update(State::Start).await {
            Ok(()) => println!("Document updated successfully!"),
            Err(_e) => println!("Ошибка в обновлении состояния диалога"),
        },
        Err(e) => {
            println!("There is an error {}", e)
        }
    }

    Ok(())
}
fn format_question(array: &[Question], cur: usize, vec_size: usize) -> String {
    format!(
        "Вопрос {}/{} от @{} \n {}",
        cur + 1,
        vec_size,
        array[cur].tg_id,
        array[cur].question
    )
}
async fn receive_question(
    bot: Bot,
    dialogue: SimpleDialouge,
    msg: Message,
    col: Collection<Question>,
    params: ConfigParameters,
    storage: MyStorage,
) -> Result<(), teloxide::RequestError> {
    let text = msg.text().unwrap();
    // TODO: send message to redis;
    let tg_id = msg.from().unwrap().username.clone().unwrap();
    let res = Question {
        question: text.to_string(),
        tg_id: tg_id.clone(),
        id: msg.chat.id.0,
        answered: false,
        upload_time: DateTime::now(),
    };
    let st = upload_question(res.clone(), col).await;
    let ans = match st {
        Ok(()) => {
            InMemStorage::update_dialogue(
                storage.clone(),
                params.chat_id,
                State::ReceiveQuest { question: res },
            )
            .await;
            bot.send_message(
                params.bot_owner,
                format!("Вам пришёл новый вопрос от @{} \n {}", tg_id, text),
            )
            .reply_markup(make_keyboard(
                InMemStorage::get_dialogue(storage, params.chat_id)
                    .await
                    .unwrap()
                    .unwrap(),
            ))
            .await?;
            "Ваш вопрос отправлен!"
        }
        Err(_e) => "Произошла ошибка при загрузке сообщения в БД, сообщите программисту",
    };
    dialogue.update(State::Start).await;
    bot.send_message(msg.chat.id, ans)
        .reply_markup(make_user_keyboard(dialogue.get().await.unwrap().unwrap()))
        .await?;
    Ok(())
}
