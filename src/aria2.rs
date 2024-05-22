#![allow(non_snake_case)]
#![allow(dead_code)]
const ARIA2_URL: &'static str = "http://localhost:6800/jsonrpc";

use reqwest::{
    Error, {Client, Response},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct AddUriRsp {
    id: String,
    jsonrpc: String,
    result: String,
}

#[derive(Serialize, Deserialize)]
struct Uris {
    status: String,
    uri: String,
}

#[derive(Serialize, Deserialize)]
struct Files {
    completedLength: String,
    index: String,
    length: String,
    path: String,
    selected: String,
    uris: Vec<Uris>,
}
#[derive(Serialize, Deserialize)]
struct Status {
    #[serde(default)]
    bitfield: Option<String>,
    completedLength: String,
    connections: String,
    dir: String,
    downloadSpeed: String,
    files: Vec<Files>,
    gid: String,
    numPieces: String,
    pieceLength: String,
    status: String,
    totalLength: String,
    uploadLength: String,
    uploadSpeed: String,
}
#[derive(Serialize, Deserialize)]
struct TellStatusRsp {
    id: String,
    jsonrpc: String,
    result: Status,
}

pub async fn download(uri: &str, dir: &str, out: &str) -> Result<String, Error> {
    let req = json!({
        "id": "w3n9",
        "method": "aria2.addUri",
        "params": [
            [ uri ],
            {
                "dir": dir,
                "out": out,
                "referer": "*"
            }
        ]
    });

    let json: AddUriRsp = Client::new()
        .post(ARIA2_URL)
        .json(&req)
        .send()
        .await?
        .json()
        .await?;
    // let rsp = Client::new().post(ARIA2_URL).json(&req).send().await?;
    // println!("download rsp:{:?}", rsp);
    // let json: AddUriRsp = rsp.json().await?;
    Ok(json.result)
}

pub async fn jsonrpc(method: &str, uid: &str) -> Result<Response, Error> {
    let req = json!({
        "id": "w3n9",
        "method": method,
        "params": [uid]
    });

    Client::new().post(ARIA2_URL).json(&req).send().await
}

pub async fn pause(uid: &str) -> Result<Response, Error> {
    jsonrpc("aria2.pause", uid).await
}
pub async fn unpause(uid: &str) -> Result<Response, Error> {
    jsonrpc("aria2.unpause", uid).await
}
pub async fn remove(uid: &str) -> Result<Response, Error> {
    jsonrpc("aria2.remove", uid).await
}

pub async fn status(uid: &str) -> Result<(String, String, String), Error> {
    let rsp = jsonrpc("aria2.tellStatus", uid).await?;
    // let rsp = rsp.text().await?;
    // println!("{uid} status rsp: {rsp}");
    // let r: TellStatusRsp = serde_json::from_str(rsp.as_str()).unwrap();
    let r: TellStatusRsp = rsp.json().await?;
    Ok((
        r.result.status,
        r.result.completedLength,
        r.result.totalLength,
    ))
}
