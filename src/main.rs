use teloxide::{
    dispatching::dialogue::{
        serializer::{Bincode, Json},
        ErasedStorage, RedisStorage, Storage,
    },
    prelude::*,
    utils::command::BotCommands,
};

type MyStorage = std::sync::Arc<ErasedStorage<Question>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Test kurwa bober");
    // let storage: MyStorage = RedisStorage::open("redis://127.0.0.1:6379", Bincode)
    //    .await
    //    .unwrap()
    //    .erase();
    let bot = Bot::from_env();
    Command::repl(bot, answer).await;
}
#[derive(Clone, Default)]
pub enum State {
    #[default]
    Idle,
    Start,
    AskInit,
    RecieveQuestion {
        question: String,
    },
}
#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Question {
    question: String,
    chat: i64,
    username: String,
}

#[derive(Clone, BotCommands)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported!"
)]
enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Все доступные боту комманды")]
    Help,
    #[command(description = "handle question")]
    Ask(String),
    #[command(description = "Test command", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Start => bot.send_message(msg.chat.id, "Hello from MSK").await?,
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Ask(quest) => match std::env::var("ID") {
            Ok(val) => {
                let id = ChatId(val.trim().parse().expect("PASHOL NAH"));
                bot.send_message(id, format!("ТЕБЕ ПРИСЛАЛИ СТРАШНОЕ СООБЩЕНИЕ!!!! {quest}"))
                    .await?
            }
            Err(err) => {
                println!("Fuck: {err}");
                bot.send_message(msg.chat.id, "There is an internal error on the server")
                    .await?
            }
        },
        Command::UsernameAndAge { username, age } => {
            bot.send_message(
                msg.chat.id,
                format!("Your username is {username} and age is {age}."),
            )
            .await?
        }
    };
    Ok(())
}
