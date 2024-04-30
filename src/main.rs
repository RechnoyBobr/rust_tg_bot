use teloxide::{
    dispatching::{
        dialogue::{
            serializer::{Bincode, Json},
            ErasedStorage, RedisStorage, Storage,
        },
        UpdateFilterExt,
    },
    prelude::*,
    utils::command::{self, BotCommands},
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting bot..");
    let bot = Bot::from_env();
    let params = ConfigParameters {
        bot_owner: UserId(0),
        owner_username: None,
    };
    // TODO: dispatching
    let handler = Update::filter_message().branch(dptree);
}

#[derive(Clone)]
struct ConfigParameters {
    bot_owner: UserId,
    owner_username: Option<String>,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum SimpleCommands {
    Start,
    Ask,
    Show,
    Answer,
    Next,
    Previous,
    Load,
}

async fn command_handler(
    cfg: ConfigParameters,
    bot: Bot,
    me: teloxide::types::Me,
    msg: Message,
    cmd: SimpleCommands,
) -> Result<(), teloxide::RequestError> {
    if msg.from().unwrap().id == cfg.bot_owner {
        let text = match cmd {
            SimpleCommands::Start => "Shpradfksdjfkljbj",
            SimpleCommands::Ask => "Ты не можешь этого XD",
            SimpleCommands::Show => "to be continued",
            SimpleCommands::Next => "load next",
            SimpleCommands::Load => "Loading more",
            SimpleCommands::Answer => "Answer ze question",
            SimpleCommands::Previous => "Loading Previous",
        };

        bot.send_message(msg.chat.id, text).await?;
    } else {
        let text = match cmd {
            SimpleCommands::Start => "Shpradfksdjfkljbj",
            SimpleCommands::Ask => "Спрашивай",
            SimpleCommands::Show => "не-а",
            SimpleCommands::Next => "не-а",
            SimpleCommands::Load => "не-а",
            SimpleCommands::Answer => "не-а",
            SimpleCommands::Previous => "не-а",
        };
        bot.send_message(msg.chat.id, text).await?;
    }
    // TODO: Дописать
    Ok(())
}
