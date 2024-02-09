use clap::{command, Arg};
use colored::*;
use dialoguer::Select;
use reqwest::{
    header::{HeaderValue, ACCEPT_LANGUAGE, USER_AGENT},
    Client,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, ops::Not, process::Command};

const SEARCH_URL: &str = "https://pipedapi.kavin.rocks/search";
const YT_URL: &str = "https://www.youtube.com";

#[derive(Debug, Serialize, Deserialize)]
pub struct Video {
    url: String,
    #[serde(rename = "type")]
    video_type: String,
    title: Option<String>,
    duration: Option<i32>,
    video_duration: Option<String>,
    #[serde(rename = "isShort")]
    is_short: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    items: Vec<Video>,
}

use yt_cli::search;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = command!()
        .about("A cli to search and play videos from piped API")
        .version("1.0.0")
        .arg(
            Arg::new("search")
                .short('s')
                .long("search")
                .help_heading(Some("Search for a video")),
        )
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .help_heading("Play a video by url"),
        )
        .arg(
            Arg::new("audio-only")
                .short('a')
                .long("audio")
                .help_heading("Play audio only")
                .required(false)
                .num_args(0),
        )
        .get_matches();

    let audio_only = matches.get_one::<bool>("audio-only").unwrap();
    let url_is_not_empty = matches.get_one::<String>("url").is_some();
    let search_is_empty = matches.get_one::<String>("search").is_some().not();

    if url_is_not_empty && search_is_empty.not() {
        println!(
            "{}",
            "Please provide either a search query or a video url, not both.".red()
        );
        return Ok(());
    }

    if url_is_not_empty {
        let url = matches
            .get_one::<String>("url")
            .map(|s| s.to_string())
            .unwrap();
        search::play_selection(&url, "From URL", *audio_only);
        return Ok(());
    }

    let search = matches
        .get_one::<String>("search")
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            let search: String = dialoguer::Input::new()
                .with_prompt("Search for a video".yellow().to_string())
                .interact()
                .unwrap();
            search
        });

    if search.is_empty() {
        println!("{}", "No search query provided".red());
        return Ok(());
    }
    let _ = Command::new("clear").status();
    println!("{} {}", "Searching for:".green().bold(), search.yellow());
    let user_agent =
        "Mozilla/5.0 (X11; U; Linux armv7l; en-US; rv:1.9.2a1pre) Gecko/20090322 Fennec/1.0b2pre";

    let client = Client::new();
    let search_value: &str = &search;
    let search_url = format!("{}?q={}&filter=all", SEARCH_URL, search_value);

    let resp = client
        .get(search_url)
        .header(USER_AGENT, HeaderValue::from_str(user_agent).unwrap())
        .header(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"))
        .send()
        .await?;

    let body = resp.text().await?;

    // searliaze the response
    let response: Response = serde_json::from_str(&body)?;

    if response.items.is_empty() {
        println!("{}", "No results found".red());
        return Ok(());
    }

    let mut videos: Vec<Video> = vec![];

    // push the videos to the videos vector if the title is "stream"
    for video in response.items {
        if video.video_type.to_lowercase() == "stream" && video.is_short.is_none().not() {
            videos.push(video);
        }
    }

    if videos.is_empty() {
        println!("{}", "No results found".red());
        return Ok(());
    }

    let mut video_titles_display: Vec<String> = vec![];
    let mut video_urls: Vec<String> = vec![];
    let mut video_titles: Vec<String> = vec![];

    let duration = |d: i32| -> String {
        let minutes = d / 60;
        let seconds = d % 60;
        // if seconds and minutes is less than 10, add a leading zero
        if seconds < 10 && minutes > 10 {
            return format!("{}:0{}", minutes, seconds);
        }
        if seconds > 10 && minutes < 10 {
            return format!("0{}:{}", minutes, seconds);
        }
        format!("{}:{}", minutes, seconds)
    };

    for (i, video) in videos.iter().enumerate() {
        let title = video.title.as_ref().unwrap().to_string().replace("//", "");
        let watch_id = video.url.to_string();
        let video_url = format!("{}{}", YT_URL, watch_id);
        let vid_duration = duration(video.duration.unwrap());
        video_urls.push(video_url);
        video_titles_display.push(format!(
            "{}. {} [󰔛 {}]",
            (i + 1).to_string().red(),
            title.clone(),
            vid_duration.clone().yellow()
        ));
        video_titles.push(title);
    }

    loop {
        let selection = Select::new()
            .items(&video_titles_display)
            .default(0)
            .interact()
            .unwrap();
        search::play_selection(
            &video_urls[selection],
            &video_titles[selection],
            *audio_only,
        );
    }
}
