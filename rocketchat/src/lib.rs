// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Sergio Martins

use std::{collections::HashMap, sync::Mutex};

use chrono::TimeZone;

/// Represents the server
pub struct RocketChat {
    url: String,
    exclusive_data: Mutex<ExclusiveData>,
}

enum RoomType {
    /// A direct conversation with someone
    Direct,
    Unknown,
    Channel,
}

impl RoomType {
    pub fn from_str(s: &str) -> RoomType {
        match s {
            "d" => RoomType::Direct,
            "c" => RoomType::Channel,
            "p" => RoomType::Channel, // Locked channel
            _ => RoomType::Unknown,
        }
    }
}

/// Data that might be accessed by another thread
#[derive(Default)]
struct ExclusiveData {
    auth_token: String,
    user_id: String,
    joined_channels: Vec<Channel>,
    pub direct_rooms: Vec<DirectRoom>,
    pub channel_rooms: Vec<Channel>,
}

#[derive(Clone)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub num_msgs: u64,
    pub last_message_timestamp: Option<i64>,
}

#[derive(Clone)]
pub struct DirectRoom {
    pub id: String,
    pub num_msgs: u64,
    pub usernames: Vec<String>,
    pub last_message_timestamp: Option<i64>,
}

impl ExclusiveData {
    fn new(auth_token: String, user_id: String) -> Self {
        Self {
            auth_token,
            user_id,
            joined_channels: Vec::new(),
            direct_rooms: Vec::new(),
            channel_rooms: Vec::new(),
        }
    }
}

impl RocketChat {
    pub fn new(url: &str, auth_token: &str) -> Self {
        Self {
            url: url.to_string(),
            exclusive_data: Mutex::new(ExclusiveData::new(auth_token.to_string(), String::new())),
        }
    }

    pub fn is_logged_in(&self) -> bool {
        let data = self.exclusive_data.lock().unwrap();
        !data.user_id.is_empty()
    }

    pub fn get_user_id(&self) -> String {
        let data = self.exclusive_data.lock().unwrap();
        data.user_id.clone()
    }

    pub fn get_auth_token(&self) -> String {
        let data = self.exclusive_data.lock().unwrap();
        data.auth_token.clone()
    }

    pub fn set_auth_token(&self, token: String) {
        let mut data = self.exclusive_data.lock().unwrap();
        data.auth_token = token;
    }

    pub fn set_user_id(&self, id: String) {
        let mut data = self.exclusive_data.lock().unwrap();
        data.user_id = id;
    }

    fn clear_user_id(&self) {
        let mut data = self.exclusive_data.lock().unwrap();
        data.user_id.clear();
    }

    pub fn set_joined_channels(&self, channels: Vec<Channel>) {
        let mut data = self.exclusive_data.lock().unwrap();
        data.joined_channels = channels;
    }

    pub fn set_direct_rooms(&self, rooms: Vec<DirectRoom>) {
        let mut data = self.exclusive_data.lock().unwrap();
        data.direct_rooms = rooms;
    }

    pub fn set_channel_rooms(&self, rooms: Vec<Channel>) {
        let mut data = self.exclusive_data.lock().unwrap();
        data.channel_rooms = rooms;
    }

    pub fn get_joined_channels(&self) -> Vec<Channel> {
        let data = self.exclusive_data.lock().unwrap();
        data.joined_channels.clone()
    }

    pub fn get_direct_rooms(&self) -> Vec<DirectRoom> {
        let data = self.exclusive_data.lock().unwrap();
        data.direct_rooms.clone()
    }

    pub fn get_channel_rooms(&self) -> Vec<Channel> {
        let data = self.exclusive_data.lock().unwrap();
        data.channel_rooms.clone()
    }

    // TODO: Improve error handling
    fn config_path(ensure_created: bool) -> String {
        let dir = dirs_next::config_dir().expect("Couldn't find config dir");
        let dir = dir.to_str().expect("Couldn't find config dir");
        let rc_config_dir = if cfg!(test) {
            format!("{}/rc-slint/tests", dir)
        } else {
            format!("{}/rc-slint", dir)
        };

        if ensure_created {
            std::fs::create_dir_all(&rc_config_dir).expect("Couldn't create config dir");
        }

        rc_config_dir.to_string()
    }

    fn auto_token_path() -> String {
        format!("{}/.auth_token", Self::config_path(true))
    }

    pub fn saved_auth_token() -> String {
        let path = Self::auto_token_path();
        if let Ok(token) = std::fs::read_to_string(path) {
            token
        } else {
            "".to_string()
        }
    }

    fn save_auth_token(&self, token: &str) -> Result<(), std::io::Error> {
        {
            let mut data = self.exclusive_data.lock().unwrap();
            data.auth_token = token.to_string();
        }

        let path = Self::auto_token_path();
        println!("Saving token to {:?}", path);
        std::fs::write(path, token)
    }

    /// Sends a POST request
    pub async fn post(
        &self,
        endpoint: &str,
        data: HashMap<&str, &str>,
    ) -> Result<serde_json::Value, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/{}", self.url, endpoint);
        let res = client
            .post(&url)
            .json(&data)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .text()
            .await;

        if let Ok(body) = res {
            // println!("url = {:?} ; Post.body = {:?}", url, body);
            let json: serde_json::Value = serde_json::from_str(&body).unwrap();
            Result::Ok(json)
        } else {
            println!("Post failed: {:?}", res);
            Result::Err("Post failed".to_string())
        }
    }

    /// Sends a GET request
    /// TODO: handle errors
    pub async fn get(&self, endpoint: &str) -> Result<serde_json::Value, ()> {
        let client = reqwest::Client::new();
        let res = client
            .get(&format!("{}/{}", self.url, endpoint))
            .header("X-Auth-Token", self.get_auth_token())
            .header("X-User-Id", self.get_user_id().as_str())
            .send()
            .await
            .unwrap()
            .text()
            .await;

        if let Ok(body) = res {
            // println!("Get.body = {:?}", body);
            let json: serde_json::Value = serde_json::from_str(&body).unwrap();
            Result::Ok(json)
        } else {
            Result::Err(())
        }
    }

    /// Logs in via a pre-existing token
    pub async fn login_via_saved_token(&self) -> Result<bool, String> {
        self.clear_user_id();
        // println!("login_via_token: auth_token = {:?}", self.get_auth_token());
        if self.get_auth_token().is_empty() {
            // No error. But we can't login without a token.
            return Ok(false);
        }

        let mut map = HashMap::new();
        let auth_token = self.get_auth_token();
        map.insert("resume", auth_token.as_str());
        let json = self.post("api/v1/login", map).await?;

        // println!("login_via_token: {:?}", json);
        let success = json["status"].as_str().ok_or("status is missing")? == "success";
        if success {
            self.set_user_id(String::from(
                json["data"]["userId"]
                    .as_str()
                    .ok_or("data or userId is missing")?,
            ));
        }

        Ok(success)
    }

    pub async fn login(&self, user: &str, pwd: &str) -> Result<(), String> {
        let mut map = HashMap::new();

        self.login_via_saved_token().await?;
        if self.is_logged_in() {
            return Ok(());
        }

        self.save_auth_token("").unwrap();

        map.insert("user", user);
        map.insert("password", pwd);

        // println!("map = {:?}", map);
        let json = self.post("api/v1/login", map).await?;
        // println!("body = {:?}", json);

        let success = json["status"].as_str().ok_or("status is missing")? == "success";
        if !success {
            return Err(String::from("login failed, success=false"));
        }

        self.set_user_id(String::from(
            json["data"]["userId"]
                .as_str()
                .ok_or("data or userId is missing")?,
        ));

        if self.is_logged_in() {
            self.set_auth_token(String::from(
                json["data"]["authToken"]
                    .as_str()
                    .ok_or("data or authToken is missing")?,
            ));
            println!("login success. authToken: {:?}", self.get_auth_token());
            self.save_auth_token(self.get_auth_token().as_str())
                .unwrap();
            Ok(())
        } else {
            println!("login failed: {:?}", json);
            Err(String::from("login failed"))
        }
    }

    pub async fn list_joined_channels(&self) {
        if !self.is_logged_in() {
            panic!("not logged in");
        }
        let res = self.get("/api/v1/channels.list.joined").await;
        match res {
            Ok(body) => {
                // serde_json::to_writer_pretty(std::io::stdout(), &body).unwrap();
                let channels = body["channels"].as_array().unwrap();
                let mut joined_channels = Vec::new();
                for c in channels {
                    let channel = Channel {
                        id: String::from(c["_id"].as_str().unwrap()),
                        name: String::from(c["name"].as_str().unwrap()),
                        num_msgs: c["msgs"].as_u64().unwrap(),
                        last_message_timestamp: Some(str_to_timestamp(c["lm"].as_str())),
                    };

                    joined_channels.push(channel);
                }
                self.set_joined_channels(joined_channels);
            }
            Err(_) => todo!(),
        }
    }

    pub async fn list_rooms(&self) {
        if !self.is_logged_in() {
            panic!("not logged in");
        }
        let res = self.get("/api/v1/rooms.get").await;
        match res {
            Ok(body) => {
                // serde_json::to_writer_pretty(std::io::stdout(), &body).unwrap();
                let success = body["success"].as_bool().unwrap_or(false);
                if !success {
                    println!("list_rooms: Failed to list rooms!");
                    return;
                }

                let mut direct_rooms = Vec::new();
                let mut channel_rooms = Vec::new();

                let rooms = body["update"].as_array().unwrap();
                for r in rooms {
                    let room_type = RoomType::from_str(r["t"].as_str().unwrap_or(""));
                    match room_type {
                        RoomType::Direct => {
                            direct_rooms.push(DirectRoom {
                                id: String::from(r["_id"].as_str().unwrap()),
                                num_msgs: r["msgs"].as_u64().unwrap_or(0),
                                usernames: r["usernames"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .map(|v| String::from(v.as_str().unwrap()))
                                    .collect(),
                                last_message_timestamp: Some(str_to_timestamp(r["lm"].as_str())),
                            });
                        }
                        RoomType::Unknown => {
                            println!("unknown room type! {} ", r);
                        }
                        RoomType::Channel => {
                            channel_rooms.push(Channel {
                                id: String::from(r["_id"].as_str().unwrap()),
                                name: String::from(r["name"].as_str().unwrap()),
                                num_msgs: r["msgs"].as_u64().unwrap_or(0),
                                last_message_timestamp: Some(str_to_timestamp(r["lm"].as_str())),
                            });
                        }
                    }
                }

                self.set_direct_rooms(direct_rooms);
                self.set_channel_rooms(channel_rooms);
            }
            Err(_) => todo!(),
        }
    }
}

// example input: 2022-05-17T14:55:23.276Z
fn str_to_timestamp(s: Option<&str>) -> i64 {
    if s.is_none() {
        return -1;
    }
    let s = s.unwrap();
    let mut parts = s.split('T');
    let date = parts.next().unwrap();
    let time = parts.next().unwrap();
    let mut parts = time.split('.');
    let time = parts.next().unwrap();
    let mut parts = time.split(':');
    let hour = parts.next().unwrap().parse::<i64>().unwrap();
    let min = parts.next().unwrap().parse::<i64>().unwrap();
    let sec = parts.next().unwrap().parse::<i64>().unwrap();
    let mut parts = date.split('-');
    let year = parts.next().unwrap().parse::<i32>().unwrap();
    let month = parts.next().unwrap().parse::<u32>().unwrap();
    let day = parts.next().unwrap().parse::<u32>().unwrap();
    let dt = chrono::Utc.with_ymd_and_hms(year, month, day, hour as u32, min as u32, sec as u32);
    dt.unwrap().timestamp()
}

#[cfg(test)]
mod tests {
    pub const RC_SLINT_TEST_USER: &str = env!("RC_SLINT_TEST_USER");
    pub const RC_SLINT_TEST_PWD: &str = env!("RC_SLINT_TEST_PWD");
    pub const RC_SLINT_TEST_URL: &str = env!("RC_SLINT_TEST_URL");

    use super::*;

    #[tokio::test]
    async fn test_login() {
        let rc = RocketChat::new(std::format!("https://{}", RC_SLINT_TEST_URL).as_str(), "");
        rc.login(RC_SLINT_TEST_USER, RC_SLINT_TEST_PWD)
            .await
            .expect("failed");
        assert_eq!(rc.is_logged_in(), true);
    }
}
