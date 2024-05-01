// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Sergio Martins

use std::{collections::HashMap, sync::Mutex};

/// Represents the server
pub struct RocketChat {
    url: String,
    exclusive_data: Mutex<ExclusiveData>,
}

/// Data that might be accessed by another thread
#[derive(Default)]
struct ExclusiveData {
    auth_token: String,
    user_id: String,
}

impl ExclusiveData {
    fn new(auth_token: String, user_id: String) -> Self {
        Self {
            auth_token,
            user_id,
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
        data.user_id.is_empty()
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
    pub async fn login_via_token(&self) -> Result<bool, String> {
        self.clear_user_id();
        if self.get_auth_token().is_empty() {
            // No error. But we can't login without a token.
            return Ok(false);
        }

        let mut map = HashMap::new();
        let auth_token = self.get_auth_token();
        map.insert("resume", auth_token.as_str());
        let json = self.post("api/v1/login", map).await?;

        println!("login_via_token: {:?}", json);
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

        self.login_via_token().await?;
        if self.is_logged_in() {
            return Ok(());
        }

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
            Ok(())
        } else {
            println!("login failed: {:?}", json);
            Err(String::from("login failed"))
        }
    }
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
