use std::{env, error::Error, sync::Arc};

use lyceumnstubot::{
    keyboards::{make_classes_keyboard, make_teachers_keyboard},
    nika::{client::NikaClient, formatter::NikaFormatter, response::NikaResponse},
};
use regex::Regex;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardMarkup, Me, ParseMode},
    utils::command::BotCommands,
};
use tokio::sync::RwLock;

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

    let global_data = Arc::new(RwLock::new(GlobalData::new().await));

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
#[command(
    rename_rule = "lowercase",
    description = "Поддерживаются следующие команды:"
)]
enum Command {
    #[command(description = "отображает этот текст.")]
    Help,
    #[command(
        description = "отображает меню с выбором класса.",
        alias = "class",
        hide_aliases
    )]
    Classes,
    #[command(
        description = "отображает меню с выбором учителя.",
        alias = "teacher",
        hide_aliases
    )]
    Teachers,
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
    global_data: Arc<RwLock<GlobalData>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            Ok(Command::Help) => {
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
            }
            Ok(Command::Classes) => {
                let classes_keyboard = {
                    let data = global_data.read().await;
                    data.classes_keyboard.clone()
                };

                bot.send_message(msg.chat.id, "Выберите класс:")
                    .reply_markup(classes_keyboard.clone())
                    .await?;
            }
            Ok(Command::Teachers) => {
                let teachers_keyboard = {
                    let data = global_data.read().await;
                    data.teachers_keyboard.clone()
                };

                bot.send_message(msg.chat.id, "Выберите учителя:")
                    .reply_markup(teachers_keyboard.clone())
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
    global_data: Arc<RwLock<GlobalData>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.answer_callback_query(&q.id).await?;

    let classes_re = Regex::new(r"class (.+)").unwrap();
    let teachers_re = Regex::new(r"teacher (.+)").unwrap();

    let nika = {
        let data = global_data.read().await;
        data.nika_response.clone()
    };

    let query = q.data.clone().unwrap();

    if let Some(caps) = classes_re.captures(&query) {
        if let Some(class_id) = caps.get(1) {
            let class_schedule = NikaFormatter::format_class_schedule(&nika, class_id.as_str());

            if let Some(message) = q.regular_message() {
                bot.edit_message_text(message.chat.id, message.id, class_schedule)
                    .parse_mode(ParseMode::Html)
                    .await?;
            } else if let Some(id) = q.inline_message_id {
                bot.edit_message_text_inline(id, class_schedule)
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
        }
    } else if let Some(caps) = teachers_re.captures(&query) {
        if let Some(_teacher_id) = caps.get(1) {
            todo!();

            // let teacher_schedule =
            //     NikaFormatter::format_teachers_schedule(&nika, teacher_id.as_str());

            // if let Some(message) = q.regular_message() {
            //     bot.edit_message_text(message.chat.id, message.id, teacher_schedule)
            //         .parse_mode(ParseMode::Html)
            //         .await?;
            // } else if let Some(id) = q.inline_message_id {
            //     bot.edit_message_text_inline(id, teacher_schedule)
            //         .parse_mode(ParseMode::Html)
            //         .await?;
            // }
        }
    }

    Ok(())
}
