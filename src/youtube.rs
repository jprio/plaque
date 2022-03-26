// in `src/youtube.rs`

use color_eyre::{eyre::eyre, Report};
use reqwest::{Client, Response};
use std::error::Error;
use tap::Pipe;
use url::Url;
const YT_CHANNEL_ID: &str = "UCs4fQRyl1TJvoeOdekW6lYA";

#[tracing::instrument]
pub(crate) async fn fetch_video_id() -> Result<String, Report> {
    Ok(Client::new()
        .get({
            let mut url = Url::parse("https://www.youtube.com/feeds/videos.xml")?;
            url.query_pairs_mut()
                .append_pair("channel_id", YT_CHANNEL_ID);
            url
        })
        // make our mark on the world
        .header("user-agent", "cool/bear")
        .send()
        .await? // this will cover connect errors
        .error_for_status()? // this will cover HTTP errors
        .bytes()
        .await? // errors while streaming the response body?
        .pipe(|bytes| feed_rs::parser::parse(&bytes[..]))? // parse the feed
        .pipe_ref(|feed| feed.entries.get(0))
        .ok_or(eyre!("no entries in video feed"))?
        .pipe_ref(|entry| entry.id.strip_prefix("yt:video:"))
        .ok_or(eyre!("first video feed item wasn't a video"))?
        .to_string())
}
