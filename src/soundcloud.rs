use reqwest::{header, Response};

// scli soundcloud app client id
const CLIENT_ID: &str = "nWYlHdW5jX1OyNQ9pipPhlUK9xDX8XFF";

pub struct Client {
    client: reqwest::Client,
    client_id: String,
    url: String,
}

#[derive(Deserialize)]
pub struct Resource {
    pub location: String,
}

#[derive(Deserialize)]
pub struct Track {
    pub duration: u32,
    pub genre: String,
    pub waveform_url: String,
    pub stream_url: String,
    pub title: String,
    pub user: User,
}

#[derive(Deserialize)]
pub struct Wave {
    pub width: u16,
    pub height: u16,
    pub samples: Vec<u16>,
}

#[derive(Deserialize)]
pub struct User {
    pub username: String,
}

impl Client {
    pub fn new() -> Client {
        let rc = reqwest::Client::builder()
            .redirect(reqwest::RedirectPolicy::none())
            .build()
            .unwrap();

        Client {
            client: rc,
            client_id: CLIENT_ID.to_string(),
            url: "https://api.soundcloud.com".to_string(),
        }
    }

    pub fn track(&self, url: String) -> Result<Track, reqwest::Error> {
        self.resolve(url).and_then(|location: String| {
            let mut resp = self
                .client
                .get(&location)
                .header(header::USER_AGENT, "scli")
                .query(&[("client_id", &self.client_id)])
                .send()?;

            Ok(resp.json()?)
        })
    }

    pub fn stream(&self, stream_url: &String) -> Result<Response, reqwest::Error> {
        // resolve stream url
        let mut resolve_resp = self
            .client
            .get(stream_url)
            .header(header::USER_AGENT, "scli")
            .query(&[("client_id", &self.client_id)])
            .send()?;

        // get raw audio from resolved resource
        let resource: Resource = resolve_resp.json()?;
        let resp = self.client.get(&resource.location).send().unwrap();

        Ok(resp)
    }

    pub fn wave(&self, track: &Track) -> Result<Wave, reqwest::Error> {
        // build a json waveform url from the png url
        let url = track.waveform_url.replace(".png", ".json");
        let mut resp = self
            .client
            .get(&url)
            .header(header::USER_AGENT, "scli")
            .send()?;

        Ok(resp.json()?)
    }

    fn resolve(&self, url: String) -> Result<String, reqwest::Error> {
        let endpoint = format!("{}{}", self.url, "/resolve");
        let mut resp = self
            .client
            .get(&endpoint)
            .header(header::USER_AGENT, "scli")
            .query(&[("client_id", &self.client_id)])
            .query(&[("url", url)])
            .send()?;

        let resource: Resource = resp.json()?;
        Ok(resource.location)
    }
}
