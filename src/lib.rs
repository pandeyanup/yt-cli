pub mod backend {
    use reqwest::{
        header::{HeaderValue, ACCEPT_LANGUAGE, USER_AGENT},
        Client,
    };
    use serde::{Deserialize, Serialize};
    use std::{error::Error, ops::Not, process::Command, thread};

    const TRENDING: &str = "https://pipedapi.kavin.rocks/trending?region=US";
    const USR_AGENT: &str =
        "Mozilla/5.0 (X11; U; Linux armv7l; en-US; rv:1.9.2a1pre) Gecko/20090322 Fennec/1.0b2pre";
    const YT_URL: &str = "https://www.youtube.com";
    const SEARCH_URL: &str = "https://pipedapi.kavin.rocks/search";

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Video {
        url: String,
        #[serde(rename = "type")]
        video_type: String,
        title: Option<String>,
        duration: Option<i32>,
        #[serde(rename = "uploaderName")]
        uploader_name: Option<String>,
        video_duration: Option<String>,
        #[serde(rename = "isShort")]
        is_short: Option<bool>,
        #[serde(rename = "uploaderVerified")]
        uploader_verified: Option<bool>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Response {
        items: Vec<Video>,
        nextpage: String,
    }

    pub struct OrangeResult {
        pub title: String,
        pub url: String,
        pub duration: String,
        pub uploader: String,
        pub is_verified: bool,
    }

    pub fn play_selection(selection: &str) {
        let selection = selection.to_owned();
        thread::spawn(move || {
            Command::new("mpv")
                .arg(&selection.trim())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("Failed to execute command. Ensure mpv is installed.");
        });
    }

    pub fn play_url(url: &str) {
        let url = url.to_owned();
        Command::new("mpv")
            .arg(&url.trim())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("Failed to execute command. Ensure mpv is installed.");
    }

    #[tokio::main]
    pub async fn get_search(search: &str) -> Result<Vec<OrangeResult>, Box<dyn Error>> {
        if search.is_empty() {
            return Ok(Vec::new());
        }
        let client = Client::new();

        let resp = client
            .get(SEARCH_URL)
            .query(&[("q", search)])
            .query(&[("filter", "all")])
            .header(USER_AGENT, HeaderValue::from_str(USR_AGENT).unwrap())
            .header(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"))
            .send()
            .await?;

        let body = resp.text().await?;
        let mut level_search: Response = serde_json::from_str(&body)?;

        // run loop x times to get more results
        for _ in 0..2 {
            let resp = client
                .get(SEARCH_URL)
                .query(&[("q", search)])
                .query(&[("filter", "all")])
                .query(&[("nextpage", &level_search.nextpage)])
                .header(USER_AGENT, HeaderValue::from_str(USR_AGENT).unwrap())
                .header(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"))
                .send()
                .await?;

            let body = resp.text().await?;
            let response: Response = serde_json::from_str(&body)?;
            level_search.items.extend(response.items);
        }

        let mut results: Vec<OrangeResult> = vec![];

        if level_search.items.is_empty() {
            return Ok(Vec::new());
        }

        let mut videos: Vec<Video> = vec![];

        // push the videos to the videos vector if the title is "stream"
        for video in level_search.items {
            if video.video_type.to_lowercase() == "stream" && video.is_short.is_none().not() {
                videos.push(video);
            }
        }

        if videos.is_empty() {
            return Ok(Vec::new());
        }

        let duration = |d: i32| -> String {
            let minutes = d / 60;
            let seconds = d % 60;
            if seconds < 10 && minutes > 10 {
                return format!("{}:0{}", minutes, seconds);
            }
            if seconds > 10 && minutes < 10 {
                return format!("0{}:{}", minutes, seconds);
            }
            format!("{}:{}", minutes, seconds)
        };

        for video in videos {
            let title = video.title.as_ref().unwrap().to_string().replace("//", "");
            let watch_id = video.url.to_string();
            let video_url = format!("{}{}", YT_URL, watch_id);
            let vid_duration = duration(video.duration.unwrap());
            let uploader = video.uploader_name.unwrap().to_string();
            let is_verified = video.uploader_verified.unwrap();
            results.push(OrangeResult {
                title,
                url: video_url,
                duration: vid_duration,
                uploader,
                is_verified,
            });
        }

        Ok(results)
    }

    #[tokio::main]
    pub async fn get_trending() -> Result<Vec<OrangeResult>, Box<dyn Error>> {
        let client = Client::new();
        let resp = client
            .get(TRENDING)
            .header(USER_AGENT, HeaderValue::from_str(USR_AGENT).unwrap())
            .header(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"))
            .send()
            .await?;

        let body = resp.text().await?;

        let response: Vec<Video> = serde_json::from_str(&body)?;
        let mut results: Vec<OrangeResult> = vec![];

        if response.is_empty() {
            return Ok(Vec::new());
        }

        let mut videos: Vec<Video> = vec![];

        for video in response {
            if video.video_type.to_lowercase() == "stream" && video.is_short.is_none().not() {
                videos.push(video);
            }
        }

        if videos.is_empty() {
            return Ok(Vec::new());
        }

        let duration = |d: i32| -> String {
            let minutes = d / 60;
            let seconds = d % 60;
            if seconds < 10 && minutes > 10 {
                return format!("{}:0{}", minutes, seconds);
            }
            if seconds > 10 && minutes < 10 {
                return format!("0{}:{}", minutes, seconds);
            }
            format!("{}:{}", minutes, seconds)
        };

        for video in videos {
            let title = video.title.as_ref().unwrap().to_string().replace("//", "");
            let watch_id = video.url.to_string();
            let video_url = format!("{}{}", YT_URL, watch_id);
            let vid_duration = duration(video.duration.unwrap());
            let is_verified = video.uploader_verified.unwrap();
            let uploader = video.uploader_name.unwrap().to_string();
            results.push(OrangeResult {
                title,
                url: video_url,
                duration: vid_duration,
                uploader,
                is_verified,
            });
        }

        Ok(results)
    }
}
