use std::error::Error;

use itertools::Itertools;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

use crate::nika::response::NikaResponse;

pub fn make_classes_keyboard(
    nika: &NikaResponse,
) -> Result<InlineKeyboardMarkup, KeyboardMakerError> {
    let keyboard: Vec<Vec<InlineKeyboardButton>> = nika
        .classes
        .iter()
        .map(|(class_id, class_name)| -> Result<_, KeyboardMakerError> {
            let course = nika.class_courses.get(class_id).unwrap().clone();
            let button = InlineKeyboardButton::callback(class_name, format!("class {class_id}"));
            Ok((course, button))
        })
        .collect::<Result<Vec<(String, InlineKeyboardButton)>, KeyboardMakerError>>()?
        .into_iter()
        .chunk_by(|(course, _)| course.clone())
        .into_iter()
        .map(|(_, group)| group.map(|(_, button)| button).collect())
        .collect();

    Ok(InlineKeyboardMarkup::new(keyboard))
}

pub fn make_teachers_keyboard(nika: &NikaResponse) -> InlineKeyboardMarkup {
    let keyboard: Vec<Vec<InlineKeyboardButton>> = nika
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
