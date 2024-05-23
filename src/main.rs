// TODO: GLOBAL:
// Show state should receieve vector of questions.
// Fetch only 10 questions from redis with date sorting from old to new;
// Make tests? Test sql queries for possible sql injection
// And of course: BUTTONS

use funcs::Question;
use teloxide::{
    dispatching::{
        dialogue::{
            serializer::{Bincode, Json},
            ErasedStorage, InMemStorage, RedisStorage, Storage,
        },
        HandlerExt, UpdateFilterExt,
    },
    prelude::*,
    types::User,
    utils::command::{self, BotCommands},
};
mod funcs;

type SimpleDialouge = Dialogue<State, InMemStorage<State>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot..");
    let bot = Bot::from_env();
    let params = ConfigParameters {
        // TODO: get UserId from environment variable. (Don't forget about Docker). Use docker
        // secrets.
        bot_owner: UserId(471831737),
        owner_username: None,
    };
    funcs::connect_to_db("redis://127.0.0.1:6379/");
    let handler = Update::filter_message()
        .branch(dptree::case![State::Show { array, cur }].endpoint(fetch_questions))
        .branch(dptree::case![State::StartQuest].endpoint(receive_question))
        .branch(dptree::case![State::StartAns].endpoint(receive_answer))
        .branch(dptree::case![State::ReceiveQuest { tg_id, id }].endpoint(handle_question))
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
                    .map(|user| user.id != cfg.bot_owner)
                    .unwrap_or_default()
            })
            .filter_command::<UserCommands>()
            .endpoint(user_command_handler),
        );
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![params])
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

#[derive(Clone)]
struct ConfigParameters {
    bot_owner: UserId,
    owner_username: Option<String>,
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
    StartAns,
    Show {
        array: Vec<funcs::Question>,
        cur: usize,
    },
    ReceiveQuest {
        tg_id: String,
        id: u64,
    },
    ReceiveAns {
        to: u64,
    },
    BogusState,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum AdminCommands {
    Start,
    Load,
    Previous,
    Next,
    AnswerStart,
}

async fn user_command_handler(
    bot: Bot,
    msg: Message,
    dialogue: SimpleDialouge,
    cmd: UserCommands,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        UserCommands::Start => "Shpradfksdjfkljbj",
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
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        AdminCommands::Start => "Добро пожаловать",
        AdminCommands::Load => {
            // get_Comments()
            "..."
        }
        AdminCommands::Next => {
            // next()
            "... next"
        }
        AdminCommands::AnswerStart => "Напишите ответ:",
        AdminCommands::Previous => {
            // get_previous()
            "..."
        }
    };
    bot.send_message(msg.chat.id, text).await?;
    Ok(())
}

async fn handle_question(
    bot: Bot,
    dialogue: SimpleDialouge,
    (question, tg_id, id): (String, String, u8),
    msg: Message,
) -> Result<(), teloxide::RequestError> {
    // TODO: finish writing
    Ok(())
}

async fn fetch_questions(
    bot: Bot,
    dialogue: SimpleDialouge,
    (question, tg_id, id): (String, String, u8),
    msg: Message,
) -> Result<(), teloxide::RequestError> {
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
    let to = match to_maybe {
        Some(t) => t,
        None => {
            println!("There is an error because of the wrong state");
            0
        }
    };
    let id = teloxide::types::UserId(to);
    bot.send_message(id, "Вам пришёл ответ на ваше сообщение:")
        .await?;
    bot.send_message(id, msg.text().unwrap()).await?;
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

async fn receive_question(
    bot: Bot,
    dialogue: SimpleDialouge,
    msg: Message,
) -> Result<(), teloxide::RequestError> {
    Ok(())
}
