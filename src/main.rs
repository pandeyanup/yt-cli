use clap::{command, Arg};
use colored::*;
use dashmap::DashMap;
use dialoguer::Select;
use futures::future::join_all;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::{
    header::{HeaderValue, ACCEPT_LANGUAGE, USER_AGENT},
    Client,
};
use scraper::{Html, Selector};
use std::{error::Error, process::Command, thread};

lazy_static! {
    static ref RE: Regex = Regex::new(r"/watch\?v\\x3d([^\\]+)").unwrap();
    static ref CACHE: DashMap<String, String> = DashMap::new();
}

fn get_video_ids(s: &str) -> Vec<String> {
    RE.captures_iter(s).map(|cap| cap[1].to_string()).collect()
}

fn play_selection(selection: &str, title: &str) {
    // Clear the terminal
    let _ = Command::new("clear").status();

    let selection = selection.to_owned();
    println!("{} {}", "Playing:".green().bold(), title.yellow());
    thread::spawn(move || {
        Command::new("mpv")
            .arg(&selection.trim())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("Failed to execute command. Ensure mpv is installed.");
    });
}

async fn get_video_title(video_id: &str, client: &Client) -> Result<String, Box<dyn Error>> {
    if let Some(title) = CACHE.get(video_id) {
        return Ok(title.value().clone());
    }

    let url = format!("https://www.youtube.com/watch?v={}", video_id);
    let resp = client.get(&url).send().await?;
    let body = resp.text().await?;

    let document = Html::parse_document(&body);
    let selector = Selector::parse("title").unwrap();
    let title = document.select(&selector).next().unwrap().inner_html();

    // Remove " - YouTube" from the end of the title
    let title = title.trim_end_matches(" - YouTube");
    CACHE.insert(video_id.to_string(), title.to_string().clone());
    Ok(title.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = command!()
        .about("A cli to search and play youtube videos")
        .version("1.0.0")
        .arg(
            Arg::new("search")
                .short('s')
                .long("search")
                .help_heading(Some("Search for a video"))
                .required(true),
        )
        .get_matches();

    let search = matches.get_one::<String>("search").unwrap();
    println!("{} {}", "Searching for:".green().bold(), search.yellow());
    let user_agent =
        "Mozilla/5.0 (X11; U; Linux armv7l; en-US; rv:1.9.2a1pre) Gecko/20090322 Fennec/1.0b2pre";

    let client = Client::new();
    let resp = client
        .get("https://www.youtube.com/results")
        .query(&[("search_query", search)])
        .header(USER_AGENT, HeaderValue::from_str(user_agent).unwrap())
        .header(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"))
        .send()
        .await?;

    let body = resp.text().await?;
    let video_ids = get_video_ids(&body);
    let mut video_urls = Vec::new();
    let mut titles = Vec::new();

    let futures = video_ids
        .iter()
        .map(|video_id| get_video_title(&video_id, &client));
    let results = join_all(futures).await;

    for (video_id, result) in video_ids.iter().zip(results) {
        let title = result?;
        let video_data = format!("https://www.youtube.com/watch?v={}", video_id);
        titles.push(title);
        video_urls.push(video_data);
    }

    loop {
        let selection = Select::new().items(&titles).default(0).interact().unwrap();
        play_selection(&video_urls[selection], &titles[selection]);
    }
}
