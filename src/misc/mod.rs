use teloxide::utils::command::BotCommands;
#[derive(Clone, Default)]
pub enum State {
    #[default]
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
pub enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Все доступные боту комманды")]
    Help,
    #[command(description = "")]
    Ask(String),
    #[command(description = "Test command", parse_with = "split")]
    UsernameAndAge { username: String, age: u8 },
}
