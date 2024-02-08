pub mod ytsearch {
    use colored::*;
    use dashmap::DashMap;
    use lazy_static::lazy_static;
    use regex::Regex;
    use reqwest::Client;
    use scraper::{Html, Selector};
    use std::{error::Error, process::Command, thread};

    lazy_static! {
        static ref RE: Regex = Regex::new(r"/watch\?v\\x3d([^\\]+)").unwrap();
        static ref CACHE: DashMap<String, String> = DashMap::new();
    }

    pub fn get_video_ids(s: &str) -> Vec<String> {
        RE.captures_iter(s).map(|cap| cap[1].to_string()).collect()
    }

    pub fn play_selection(selection: &str, title: &str, audio_only: bool) {
        // Clear the terminal
        let _ = Command::new("clear").status();

        let selection = selection.to_owned();
        println!("{} {}", "Playing:".green().bold(), title.yellow());
        if audio_only {
            let mut child = Command::new("mpv")
                .arg("--no-video")
                // .arg("--force-window")
                .arg(&selection.trim())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("Failed to execute command. Ensure mpv is installed.");

            let child_id = child.id();

            ctrlc::set_handler(move || {
                let _ = Command::new("kill")
                    .arg("-2")
                    .arg(child_id.to_string())
                    .spawn();
                // then exit the program
                std::process::exit(0);
            })
            .expect("Error setting Ctrl-C handler");

            // Wait for the child process to finish
            let _ = child.wait().expect("Failed to wait on child");
        } else {
            thread::spawn(move || {
                Command::new("mpv")
                    .arg(&selection.trim())
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .spawn()
                    .expect("Failed to execute command. Ensure mpv is installed.")
            });
        }
    }

    pub async fn get_video_title(
        video_id: &str,
        client: &Client,
    ) -> Result<String, Box<dyn Error>> {
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
}
