use std::error::Error;

use regex::Regex;

use super::response::NikaResponse;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NikaClient {}

impl NikaClient {
    pub async fn get_data() -> Result<NikaResponse, NikaClientError> {
        let filename_regex = Regex::new(
            r#"<script type="text/javascript" src="(nika_data_\d{8}_\d{6}\.js)"></script>"#,
        )?;
        let nika_json_regex = Regex::new(r#"(\{.+})"#)?;

        let html = reqwest::get("https://lyceum.nstu.ru/rasp/schedule.html")
            .await?
            .text()
            .await?;

        let filename = filename_regex
            .captures(&html)
            .ok_or(NikaClientError::FilenameParsing)?
            .get(1)
            .ok_or(NikaClientError::FilenameParsing)?
            .as_str();

        let nika_script_url = format!("https://lyceum.nstu.ru/rasp/{}", filename);
        let nika_script = reqwest::get(nika_script_url).await?.text().await?;

        let nika_json = nika_json_regex
            .captures(&nika_script)
            .ok_or(NikaClientError::FilenameParsing)?
            .get(1)
            .ok_or(NikaClientError::FilenameParsing)?
            .as_str();

        let nika = serde_json::from_str(nika_json)?;

        Ok(nika)
    }
}

#[derive(Debug)]
pub enum NikaClientError {
    Request(reqwest::Error),
    Json(serde_json::Error),
    Regex(regex::Error),
    FilenameParsing,
}

impl Error for NikaClientError {}

impl std::fmt::Display for NikaClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NikaClientError::Request(err) => err.fmt(f),
            NikaClientError::Regex(err) => err.fmt(f),
            NikaClientError::FilenameParsing => write!(f, "couldn't parse filename"),
            NikaClientError::Json(err) => err.fmt(f),
        }
    }
}

impl From<reqwest::Error> for NikaClientError {
    fn from(err: reqwest::Error) -> NikaClientError {
        NikaClientError::Request(err)
    }
}

impl From<regex::Error> for NikaClientError {
    fn from(err: regex::Error) -> NikaClientError {
        NikaClientError::Regex(err)
    }
}

impl From<serde_json::Error> for NikaClientError {
    fn from(err: serde_json::Error) -> NikaClientError {
        NikaClientError::Json(err)
    }
}
