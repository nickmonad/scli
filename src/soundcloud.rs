use reqwest::header;
use std::env;
use std::fs::File;
use std::{thread, time};

pub struct Client {
    client: reqwest::Client,
    oauth: String,
    url: String,
}

#[derive(Deserialize)]
pub struct Resource {
    pub location: String,
}

#[derive(Deserialize)]
pub struct Track {
    pub title: String,
    pub genre: String,
    pub stream_url: String,
    pub user: User,
}

#[derive(Deserialize)]
pub struct User {
    pub username: String,
}

impl Client {
    pub fn new() -> Client {
        let oauth = env::var("SC_TOKEN").expect("no oauth token set");
        let rc = reqwest::Client::builder()
            .redirect(reqwest::RedirectPolicy::none())
            .build()
            .unwrap();

        Client {
            client: rc,
            oauth: oauth,
            url: "https://api.soundcloud.com".to_string(),
        }
    }

    pub fn stream(&self, url: String) -> Result<File, reqwest::Error> {
        self.resolve(url)
            .and_then(|location: String| {
                // fetch track metadata
                let mut resp = self
                    .client
                    .get(&location)
                    .header(header::USER_AGENT, "scli")
                    .query(&[("oauth_token", &self.oauth)])
                    .send()?;

                let track: self::Track = resp.json()?;
                Ok(track.stream_url)
            })
            .and_then(|stream_url: String| {
                // fetch stream url
                let mut resp = self
                    .client
                    .get(&stream_url)
                    .header(header::USER_AGENT, "scli")
                    .query(&[("oauth_token", &self.oauth)])
                    .send()?;

                let resource: self::Resource = resp.json()?;
                Ok(resource.location)
            })
            .and_then(|location: String| {
                // fetch raw audio from resolved stream CDN location
                let mut resp = self.client.get(&location).send().unwrap();

                // create a temporary file on disk and spawn a thread to write to it
                let mut writer = File::create("stream.mp3").unwrap();
                let reader = File::open("stream.mp3").unwrap();

                // TODO(ngmiller)
                // This is terribly hacky. It seems the returned reader file handle
                // doesn't handle the writing very well when stream.mp3 doesn't exist
                // and causes the player thread to error out with an unrecognized format,
                // so we need to sleep a bit after starting the writer thread.
                // Ideally, this is all buffered in memory and we don't have to use a file
                // to coordinate.
                thread::spawn(move || {
                    resp.copy_to(&mut writer).unwrap();
                });

                thread::sleep(time::Duration::from_millis(100));
                Ok(reader)
            })
    }

    fn resolve(&self, url: String) -> Result<String, reqwest::Error> {
        let endpoint = format!("{}{}", self.url, "/resolve");
        let mut resp = self
            .client
            .get(&endpoint)
            .header(header::USER_AGENT, "scli")
            .query(&[("oauth_token", &self.oauth)])
            .query(&[("url", url)])
            .send()?;

        let resolved: self::Resource = resp.json()?;
        Ok(resolved.location)
    }
}
