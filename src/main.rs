use std::{env, error::Error, sync::Arc};

use lyceumnstubot::{
    keyboards::{make_classes_keyboard, make_teachers_keyboard},
    nika::{client::NikaClient, formatter::NikaFormatter, response::NikaResponse},
};
use teloxide::{
    prelude::*,
    types::{InlineKeyboardMarkup, Me, ParseMode},
    utils::command::BotCommands,
};
use tokio::sync::Mutex;

#[derive(Clone, Debug)]
struct GlobalData {
    nika_response: NikaResponse,
    classes_keyboard: InlineKeyboardMarkup,
    teachers_keyboard: InlineKeyboardMarkup,
}

impl GlobalData {
    async fn new() -> GlobalData {
        let nika_response = NikaClient::get_data().await.unwrap();
        let classes_keyboard = make_classes_keyboard(&nika_response).unwrap();
        let teachers_keyboard = make_teachers_keyboard(&nika_response);

        GlobalData {
            nika_response,
            classes_keyboard,
            teachers_keyboard,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    let bot_token = env::var("BOT_TOKEN")?;
    let bot = Bot::new(bot_token);

    let global_data = Arc::new(Mutex::new(GlobalData::new().await));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![global_data])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Поддерживаются эти команды:")]
enum Command {
    #[command(description = "отображает этот текст.")]
    Help,
    #[command(description = "отображает меню с выбором класса.")]
    Classes,
    #[command(description = "отображает меню с выбором учителя.")]
    Teachers,
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
    global_data: Arc<Mutex<GlobalData>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
            }
            Ok(Command::Classes) => {
                let data = global_data.lock().await;
                bot.send_message(msg.chat.id, "Выберите класс:")
                    .reply_markup(data.classes_keyboard.clone())
                    .await?;
            }
            Ok(Command::Teachers) => {
                let data = global_data.lock().await;
                bot.send_message(msg.chat.id, "Выберите учителя:")
                    .reply_markup(data.teachers_keyboard.clone())
                    .await?;
            }
            Err(_) => {
                bot.send_message(msg.chat.id, "Команда не найдена!").await?;
            }
        }
    }

    Ok(())
}

async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    global_data: Arc<Mutex<GlobalData>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let data = global_data.lock().await;

    if let Some(ref class_id) = q.data {
        bot.answer_callback_query(&q.id).await?;

        let class_schedule = NikaFormatter::format_class_schedule(&data.nika_response, class_id);

        if let Some(message) = q.regular_message() {
            bot.edit_message_text(message.chat.id, message.id, class_schedule)
                .parse_mode(ParseMode::Html)
                .await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, class_schedule)
                .parse_mode(ParseMode::Html)
                .await?;
        }

        log::info!("You chose: {}", class_id);
    }

    Ok(())
}
