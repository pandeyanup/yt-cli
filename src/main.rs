use clap::{command, Arg};
use dialoguer::Select;
use regex::Regex;
use reqwest::{
    header::{HeaderValue, ACCEPT_LANGUAGE, USER_AGENT},
    Client,
};
use scraper::{Html, Selector};
use std::{error::Error, process::Command};

fn get_video_ids(s: &str) -> Vec<String> {
    let re = Regex::new(r"/watch\?v\\x3d([^\\]+)").unwrap();
    re.captures_iter(s).map(|cap| cap[1].to_string()).collect()
}

fn play_selection(selection: &str) {
    println!("Playing: {}", selection);
    //let output =
    Command::new("mpv")
        .arg(&selection.trim())
        .output()
        .expect("Failed to execute command. Ensure mpv is installed.");

    // println!("{}", String::from_utf8_lossy(&output.stdout));
}

async fn get_video_title(video_id: &str) -> Result<String, Box<dyn Error>> {
    let url = format!("https://www.youtube.com/watch?v={}", video_id);
    let resp = Client::new().get(&url).send().await?;
    let body = resp.text().await?;

    let document = Html::parse_document(&body);
    let selector = Selector::parse("title").unwrap();
    let title = document.select(&selector).next().unwrap().inner_html();

    // Remove " - YouTube" from the end of the title
    let title = title.trim_end_matches(" - YouTube");
    Ok(title.to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = command!()
        .about("A cli to search and play youtube videos")
        .version("0.1.0")
        .arg(
            Arg::new("search")
                .short('s')
                .long("search")
                .help_heading(Some("Search for a video"))
                .required(true),
        )
        .get_matches();

    let search = matches.get_one::<String>("search").unwrap();
    let user_agent =
        "Mozilla/5.0 (X11; U; Linux armv7l; en-US; rv:1.9.2a1pre) Gecko/20090322 Fennec/1.0b2pre";

    let client = reqwest::Client::new();
    let resp: reqwest::Response = client
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

    for video_id in video_ids {
        let title = get_video_title(&video_id).await?;
        let video_data = format!("https://www.youtube.com/watch?v={}", video_id);
        titles.push(title);
        video_urls.push(video_data);
    }

    loop {
        let selection = Select::new().items(&titles).default(0).interact().unwrap();
        let _ = play_selection(&video_urls[selection]);
    }
}
