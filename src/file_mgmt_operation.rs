use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Operation {
    path: String,
    #[serde(default)]
    dest: Option<String>,
    #[serde(default)]
    newname: Option<String>,
    #[serde(default = "default_ondup")]
    ondup: String
}

fn default_ondup() -> String {
    "fail".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Request {
    CopyMove(Vec<Operation>),
    Rename(Vec<Operation>),
    Delete(Vec<String>),
}

#[derive(Deserialize)]
pub struct ErrNo {
    pub errno: i32,
}

pub async fn post_request(client: &reqwest::Client,url: &str, payload: reqwest::multipart::Form) -> Result<ErrNo, Box<dyn Error>> {

    let response = client
                    .post(url)
                    .multipart(payload)
                    .send().await?
                    .text().await?;
    let body: ErrNo = serde_json::from_str(&response)?;

    Ok(body)

}