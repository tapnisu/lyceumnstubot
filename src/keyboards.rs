use std::error::Error;

use itertools::Itertools;
use regex::Regex;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::nika::response::NikaResponse;

pub fn make_classes_keyboard(
    nika_response: &NikaResponse,
) -> Result<InlineKeyboardMarkup, KeyboardMakerError> {
    let grade_regex = Regex::new(r#"^(\d{1,2})([\u0430-\u0433]|(?:-[1-4]))$"#)?;

    let keyboard: Vec<Vec<InlineKeyboardButton>> = nika_response
        .classes
        .iter()
        .map(|(class_id, class_name)| -> Result<_, KeyboardMakerError> {
            let grade = grade_regex
                .captures(class_name)
                .ok_or(KeyboardMakerError::GradeParsing)?
                .get(1)
                .ok_or(KeyboardMakerError::GradeParsing)?
                .as_str()
                .parse::<i32>()
                .unwrap();

            let button = InlineKeyboardButton::callback(class_name, format!("class {class_id}"));

            Ok((grade, button))
        })
        .collect::<Result<Vec<(i32, InlineKeyboardButton)>, KeyboardMakerError>>()?
        .into_iter()
        .chunk_by(|(grade, _)| *grade)
        .into_iter()
        .map(|(_, group)| group.map(|(_, button)| button).collect())
        .collect();

    Ok(InlineKeyboardMarkup::new(keyboard))
}

pub fn make_teachers_keyboard(nika_response: &NikaResponse) -> InlineKeyboardMarkup {
    let keyboard: Vec<Vec<InlineKeyboardButton>> = nika_response
        .teachers
        .iter()
        .sorted_by(|(_, a), (_, b)| a.cmp(b))
        .map(|(teacher_id, teacher_name)| {
            InlineKeyboardButton::callback(teacher_name, format!("teacher {teacher_id}"))
        })
        .chunks(3)
        .into_iter()
        .map(|chunk| chunk.collect())
        .collect();

    InlineKeyboardMarkup::new(keyboard)
}

#[derive(Debug)]
pub enum KeyboardMakerError {
    Regex(regex::Error),
    GradeParsing,
}

impl Error for KeyboardMakerError {}

impl std::fmt::Display for KeyboardMakerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyboardMakerError::Regex(err) => err.fmt(f),
            KeyboardMakerError::GradeParsing => write!(f, "couldn't parse grade"),
        }
    }
}

impl From<regex::Error> for KeyboardMakerError {
    fn from(err: regex::Error) -> KeyboardMakerError {
        KeyboardMakerError::Regex(err)
    }
}
