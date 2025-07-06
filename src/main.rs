use std::{env, error::Error, process::exit, sync::Arc, time::Duration};

use dotenv::dotenv;
use lyceumnstubot::{
    keyboards::{make_classes_keyboard, make_teachers_keyboard},
    nika::{client::NikaClient, formatter::NikaFormatter, response::NikaResponse},
};
use regex::Regex;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Me, ParseMode},
    utils::command::BotCommands,
};
use tokio::{sync::RwLock, time};

#[derive(Clone, Debug)]
struct GlobalData {
    nika_response: NikaResponse,
    classes_keyboard: InlineKeyboardMarkup,
    teachers_keyboard: InlineKeyboardMarkup,
}

impl GlobalData {
    async fn new() -> anyhow::Result<GlobalData> {
        let nika_response = NikaClient::get_data().await?;
        let classes_keyboard = make_classes_keyboard(&nika_response)?;
        let teachers_keyboard = make_teachers_keyboard(&nika_response);

        Ok(GlobalData {
            nika_response,
            classes_keyboard,
            teachers_keyboard,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting command bot...");

    dotenv().ok();

    let bot_token = env::var("BOT_TOKEN")?;
    let bot = Bot::new(bot_token);
    let global_data = match GlobalData::new().await {
        Ok(data) => Arc::new(RwLock::new(data)),
        Err(err) => {
            eprintln!("{err}");
            exit(1);
        }
    };

    let mut interval = time::interval(Duration::from_secs(5 * 60));
    tokio::spawn({
        let global_data = global_data.clone();

        async move {
            loop {
                interval.tick().await;
                let mut rw_data = global_data.write().await;

                match GlobalData::new().await {
                    Ok(data) => *rw_data = data,
                    Err(err) => eprintln!("{err}"),
                }
            }
        }
    });

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

    let classes_re = Regex::new(r"class (.+)")?;
    let teachers_re = Regex::new(r"teacher (.+)")?;

    let nika = {
        let data = global_data.read().await;
        data.nika_response.clone()
    };

    let query = match q.data.clone() {
        None => {
            bot.answer_callback_query(q.id)
                .text("Что-то пошло не так")
                .await?;
            return Ok(());
        }
        Some(query) => query,
    };

    if let Some(caps) = classes_re.captures(&query) {
        if let Some(class_id) = caps.get(1) {
            let class_schedule = NikaFormatter::format_class_schedule(&nika, class_id.as_str());
            let reply_markup =
                InlineKeyboardMarkup::new(vec![vec![InlineKeyboardButton::callback(
                    "Назад ⬅️",
                    "classSchedule",
                )]]);

            if let Some(message) = q.regular_message() {
                bot.edit_message_text(message.chat.id, message.id, class_schedule)
                    .reply_markup(reply_markup)
                    .parse_mode(ParseMode::Html)
                    .await?;
            } else if let Some(id) = q.inline_message_id {
                bot.edit_message_text_inline(id, class_schedule)
                    .reply_markup(reply_markup)
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
        }
    } else if let Some(caps) = teachers_re.captures(&query) {
        if let Some(teacher_id) = caps.get(1) {
            let teacher_schedule =
                NikaFormatter::format_teacher_schedule(&nika, teacher_id.as_str());

            if let Some(message) = q.regular_message() {
                bot.edit_message_text(message.chat.id, message.id, teacher_schedule)
                    .parse_mode(ParseMode::Html)
                    .await?;
            } else if let Some(id) = q.inline_message_id {
                bot.edit_message_text_inline(id, teacher_schedule)
                    .parse_mode(ParseMode::Html)
                    .await?;
            }
        }
    } else if query == "classSchedule" {
        let classes_keyboard = {
            let data = global_data.read().await;
            data.classes_keyboard.clone()
        };

        if let Some(message) = q.regular_message() {
            bot.edit_message_text(message.chat.id, message.id, "Выберите класс:")
                .reply_markup(classes_keyboard.clone())
                .await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, "Выберите класс:")
                .reply_markup(classes_keyboard.clone())
                .await?;
        }
    } else if query == "teacherSchedule" {
        let teachers_keyboard = {
            let data = global_data.read().await;
            data.teachers_keyboard.clone()
        };

        if let Some(message) = q.regular_message() {
            bot.edit_message_text(message.chat.id, message.id, "Выберите учителя:")
                .reply_markup(teachers_keyboard.clone())
                .await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, "Выберите учителя:")
                .reply_markup(teachers_keyboard.clone())
                .await?;
        }
    }

    Ok(())
}
