use chrono::{Duration, NaiveDate};
use itertools::Itertools;

use super::response::NikaResponse;

pub struct NikaFormatter {}

impl NikaFormatter {
    pub fn format_class_schedule(nika: &NikaResponse, class_id: &str) -> String {
        let beginning_date = {
            let beginning = nika.periods.clone().first_entry().unwrap().get().b.clone();
            NaiveDate::parse_from_str(&beginning, "%d.%m.%Y").unwrap()
        };

        let mut schedule: Vec<String> = nika
            .class_schedule
            .clone()
            .first_entry()
            .unwrap()
            .get()
            .get(class_id)
            .unwrap()
            .iter()
            .map(|(lesson_id, class_schedule_entry)| {
                let groups = class_schedule_entry.s.len();
                let lesson_number = lesson_id.parse::<i32>().unwrap() % 100;

                let entry = (0..groups)
                    .map(|group_id| {
                        let important_data = {
                            if class_schedule_entry.r[group_id].is_empty()
                                || class_schedule_entry.s[group_id].is_empty()
                                || class_schedule_entry.t[group_id].is_empty()
                            {
                                "<b>нет занятий</b>".to_string()
                            } else {
                                let room = &nika.rooms[&class_schedule_entry.r[group_id]];
                                let subject = &nika.subjects[&class_schedule_entry.s[group_id]];
                                let teacher = &nika.teachers[&class_schedule_entry.t[group_id]];
                                let lesson_times =
                                    nika.lesson_times.get(&lesson_number.to_string()).unwrap();

                                format!(
                                    "{room}: {subject} | {teacher} <code>{}-{}</code>",
                                    lesson_times[0], lesson_times[1]
                                )
                            }
                        };

                        let tag = if groups == 1 {
                            format!("{lesson_number}.")
                        } else {
                            format!("{lesson_number}. <i>(Груп.{})</i>", group_id + 1)
                        };

                        format!("{tag} {important_data}")
                    })
                    .join("\n");

                (lesson_id, entry)
            })
            .chunk_by(|(lesson_id, _)| {
                lesson_id.chars().nth(0).unwrap().to_digit(10).unwrap() as usize
            })
            .into_iter()
            .map(|(day_id, group)| {
                let date = beginning_date + Duration::days(day_id.try_into().unwrap());

                format!(
                    "<b>{} / {}:</b>\n{}",
                    nika.day_names[day_id - 1],
                    date.format("%d.%m.%Y"),
                    group.map(|(_, text_entry)| text_entry).join("\n")
                )
            })
            .collect();

        let class_name = nika.classes.get(class_id).unwrap();
        schedule.insert(0, format!("<i>Расписание для {class_name}:</i>"));
        schedule.join("\n\n")
    }

    pub fn format_teacher_schedule(nika: &NikaResponse, teacher_id: &str) -> String {
        let beginning_date = {
            let beginning = nika.periods.clone().first_entry().unwrap().get().b.clone();
            NaiveDate::parse_from_str(&beginning, "%d.%m.%Y").unwrap()
        };

        let mut schedule: Vec<String> = nika
            .teach_schedule
            .clone()
            .first_entry()
            .unwrap()
            .get()
            .get(teacher_id)
            .unwrap()
            .iter()
            .map(|(lesson_id, teacher_schedule_entry)| {
                let subject = &nika.subjects[&teacher_schedule_entry.s];
                let lesson_number = lesson_id.parse::<i32>().unwrap() % 100;
                let lesson_times = nika.lesson_times.get(&lesson_number.to_string()).unwrap();

                let room = teacher_schedule_entry
                    .r
                    .clone()
                    .map_or("-".to_string(), |room_id| nika.rooms[&room_id].clone());

                let classes =
                    teacher_schedule_entry
                        .c
                        .clone()
                        .map_or("-".to_string(), |class_ids| {
                            class_ids
                                .into_iter()
                                .map(|class_id| nika.classes[&class_id].clone())
                                .join(", ")
                        });

                let important_data = format!(
                    "{room}: {subject} | {classes} <code>{}-{}</code>",
                    lesson_times[0], lesson_times[1]
                );

                let entry = format!("{lesson_number}. {important_data}");

                (lesson_id, entry)
            })
            .chunk_by(|(lesson_id, _)| {
                lesson_id.chars().nth(0).unwrap().to_digit(10).unwrap() as usize
            })
            .into_iter()
            .map(|(day_id, group)| {
                let date = beginning_date + Duration::days(day_id.try_into().unwrap());

                format!(
                    "<b>{} / {}:</b>\n{}",
                    nika.day_names[day_id - 1],
                    date.format("%d.%m.%Y"),
                    group.map(|(_, text_entry)| text_entry).join("\n")
                )
            })
            .collect();

        let class_name = nika.teachers.get(teacher_id).unwrap();
        schedule.insert(0, format!("<i>Расписание для {class_name}:</i>"));
        schedule.join("\n\n")
    }
}
