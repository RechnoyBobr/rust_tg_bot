// TODO: GLOBAL:
// Show state should receieve vector of questions.
// Fetch only 10 questions from redis with date sorting from old to new;
// Make tests? Test sql queries for possible sql injection
// And of course: BUTTONS

use funcs::{load_questions, upload_question, Question};
use mongodb::{bson::DateTime, Client, Collection};
use std::cell::Cell;
use teloxide::{
    dispatching::{dialogue::InMemStorage, HandlerExt, UpdateFilterExt},
    prelude::*,
    utils::command::BotCommands,
};
mod funcs;

type SimpleDialouge = Dialogue<State, InMemStorage<State>>;

#[derive(Clone)]
struct ConfigParameters {
    bot_owner: UserId,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum UserCommands {
    Start,
    AskStart,
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    StartQuest,
    StartAns {
        q: Question,
    },
    Show {
        array: Vec<Question>,
        cur: usize,
    },
    ReceiveQuest {
        tg_id: String,
        id: i64,
    },
    ReceiveAns {
        to: i64,
    },
    BogusState,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum AdminCommands {
    Start,
    Load,
    Show,
    Previous,
    Next,
    AnswerStart,
}

const PARAMS: ConfigParameters = ConfigParameters {
    // TODO: get UserId from environment variable. (Don't forget about Docker). Use docker
    // secrets.
    bot_owner: UserId(471831737),
};
#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot..");
    dotenv::from_filename(".env.local").ok();
    let mongodb_uri = dotenv::var("MONGODB_URI").expect(
        "MONGODB_URI should be specified in the file .env.local in the root of the project",
    );
    let mongodb_client = Client::with_uri_str(mongodb_uri).await.unwrap();
    let db = mongodb_client.database("rust_bot");
    let collection: Collection<Question> = db.collection("questions");
    let test = collection.list_index_names().await.unwrap();
    for i in test {
        println!("{:?}", i);
    }

    let bot = Bot::from_env();
    let handler = Update::filter_message()
        .enter_dialogue::<Message, InMemStorage<State>, State>()
        .branch(dptree::case![State::StartQuest].endpoint(receive_question))
        .branch(dptree::case![State::ReceiveAns { to }].endpoint(handle_answer))
        .branch(
            dptree::filter(|cfg: ConfigParameters, msg: Message| {
                msg.from()
                    .map(|user| user.id == cfg.bot_owner)
                    .unwrap_or_default()
            })
            .filter_command::<AdminCommands>()
            .endpoint(admin_command_handler),
        )
        .branch(
            dptree::filter(|cfg: ConfigParameters, msg: Message| {
                msg.from()
                    .map(|user| user.id != UserId(0))
                    .unwrap_or_default()
            })
            .filter_command::<UserCommands>()
            .endpoint(user_command_handler),
        );
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            PARAMS,
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

async fn user_command_handler(
    bot: Bot,
    msg: Message,
    dialogue: SimpleDialouge,
    cmd: UserCommands,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        UserCommands::Start => "Приветствую вас в своём мега-крутом ботеее!",
        UserCommands::AskStart => {
            dialogue.update(State::StartQuest).await.unwrap();
            "Задайте ваш вопрос"
        }
    };
    bot.send_message(msg.chat.id, text).await?;
    // TODO: Дописать
    Ok(())
}

async fn admin_command_handler(
    bot: Bot,
    msg: Message,
    dialogue: SimpleDialouge,
    cmd: AdminCommands,
    client: mongodb::Collection<Question>,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        AdminCommands::Start => String::from("Добро пожаловать"),
        AdminCommands::Show => {
            // TODO: load array of messages
            let mut v: Vec<Question> = vec![];
            let load_result = load_questions(client, &mut v).await;
            match load_result {
                Ok(()) => {
                    dialogue.update(State::Show { array: v, cur: 0 }).await;
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
        AdminCommands::Load => {
            // get_Comments()
            String::from("...")
        }
        AdminCommands::Next => {
            if let State::Show { array, cur } = dialogue.get().await.unwrap().unwrap() {
                let ret_val = format_question(&array, (cur + 1) % array.len(), array.len());
                dialogue
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
        AdminCommands::AnswerStart => {
            let cur_state = dialogue.get().await.unwrap().unwrap();
            if let State::Show { array, cur } = cur_state {
                dialogue
                    .update(State::StartAns {
                        q: array[cur].clone(),
                    })
                    .await;
                String::from("Напишите ответ в сообщении ниже, он будет отправлен")
            } else {
                String::from("Нет вопроса, на который нужно ответить")
            }
        }
        AdminCommands::Previous => {
            if let State::Show { array, cur } = dialogue.get().await.unwrap().unwrap() {
                let ret_val = format_question(&array, (cur - 1) % array.len(), array.len());
                dialogue
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
    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}

async fn handle_answer(
    bot: Bot,
    dialogue: SimpleDialouge,
    msg: Message,
) -> Result<(), teloxide::RequestError> {
    // WARN: Maybe variables from state would not work
    bot.send_message(msg.chat.id, "Ваш ответ отправлен").await?;
    let state = match dialogue.get().await {
        Ok(s) => s.unwrap(),
        Err(e) => {
            println!("There is an error, probably because of wrong state");
            State::BogusState
        }
    };
    let to_maybe = match state {
        State::ReceiveAns { to } => Some(to),
        _ => None,
    };
    let id = match to_maybe {
        Some(t) => t,
        None => {
            println!("There is an error because of the wrong state");
            0
        }
    };
    bot.send_message(ChatId(id), "Вам пришёл ответ на ваше сообщение:")
        .await?;
    bot.send_message(ChatId(id), msg.text().unwrap()).await?;
    Ok(())
}
async fn receive_answer(
    bot: Bot,
    dialogue: SimpleDialouge,
    msg: Message,
) -> Result<(), teloxide::RequestError> {
    let cur_state = match dialogue.get().await {
        Ok(s) => s.unwrap_or(State::BogusState),
        Err(e) => State::BogusState,
    };
    let question = match cur_state {
        State::Show { array, cur } => Some(array[cur].clone()),
        _ => None,
    };
    let id = match question {
        Some(q) => q.id,
        None => {
            println!("Invalid state!!");
            0
        }
    };

    let res = dialogue.update(State::ReceiveAns { to: id }).await;
    match res {
        Ok(s) => {
            bot.send_message(msg.chat.id, "Ответьте в сообщении ниже:")
                .await?;
        }
        Err(e) => {
            bot.send_message(
                msg.chat.id,
                "Внутренняя ошибка, обратитесь к разрабу. Он еблан.",
            )
            .await?;
        }
    }
    Ok(())
}
fn format_question(array: &[Question], cur: usize, vec_size: usize) -> String {
    format!(
        "Вопрос {}/{} от @{} \n {}",
        cur, vec_size, array[cur].tg_id, array[cur].question
    )
}
async fn receive_question(
    bot: Bot,
    dialogue: SimpleDialouge,
    msg: Message,
    col: Collection<Question>,
) -> Result<(), teloxide::RequestError> {
    let text = msg.text().unwrap();
    // TODO: send message to redis;
    let res = dialogue
        .update(State::ReceiveQuest {
            tg_id: msg.from().unwrap().username.clone().unwrap(),
            id: msg.chat.id.0,
        })
        .await;
    match res {
        Ok(()) => {
            let tg_id = msg.from().unwrap().username.clone().unwrap();
            bot.send_message(msg.chat.id, text).await?;
            let res = Question {
                question: text.to_string(),
                tg_id,
                id: msg.chat.id.0,
                answered: false,
                upload_time: DateTime::now(),
            };
            println!("{:?}", res.tg_id);
            bot.send_message(
                PARAMS.bot_owner,
                format!("Вам пришёл новый вопрос от @{} \n {}", res.tg_id, text),
            )
            .await;
            let st = upload_question(res, col).await;
            let ans = match st {
                Ok(()) => "Ваш вопрос отправлен!",
                Err(e) => "Произошла ошибка при загрузке сообщения в БД, сообщите программисту",
            };
            bot.send_message(msg.chat.id, ans).await?;
        }
        Err(e) => {
            bot.send_message(
                msg.chat.id,
                "Произошла ошибка с teloxide api, сообщите программисту об этом",
            )
            .await?;
        }
    }
    Ok(())
}
