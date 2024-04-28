// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Sergio Martins

use std::collections::HashMap;

/// Represents the server
pub struct RocketChat {
    url: String,
    user_id: String,
    auth_token: String,
}

impl RocketChat {
    pub fn new(url: &str, auth_token: &str) -> Self {
        Self {
            url: url.to_string(),
            user_id: String::new(),
            auth_token: auth_token.to_string(),
        }
    }

    pub fn is_logged_in(&self) -> bool {
        !self.user_id.is_empty()
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
            .unwrap()
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
            .header("X-Auth-Token", self.auth_token.as_str())
            .header("X-User-Id", self.user_id.as_str())
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
    pub async fn login_via_token(&mut self) -> Result<bool, String> {
        self.user_id.clear();
        if self.auth_token.is_empty() {
            // No error. But we can't login without a token.
            return Ok(false);
        }

        let mut map = HashMap::new();
        map.insert("resume", self.auth_token.as_str());
        let json = self.post("api/v1/login", map).await?;

        println!("login_via_token: {:?}", json);
        let success = json["status"].as_str().ok_or("status is missing")? == "success";
        if success {
            self.user_id = String::from(
                json["data"]["userId"]
                    .as_str()
                    .ok_or("data or userId is missing")?,
            );
        }

        Ok(success)
    }

    pub async fn login(&mut self, user: &str, pwd: &str) -> Result<(), String> {
        let mut map = HashMap::new();

        self.login_via_token().await?;
        if self.is_logged_in() {
            return Ok(());
        }

        map.insert("user", user);
        map.insert("password", pwd);

        let json = self.post("api/v1/login", map).await?;
        // println!("body = {:?}", res);

        let success = json["status"].as_str().ok_or("status is missing")? == "success";
        if !success {
            return Err(String::from("login failed, success=false"));
        }

        self.user_id = String::from(
            json["data"]["userId"]
                .as_str()
                .ok_or("data or userId is missing")?,
        );

        if self.is_logged_in() {
            self.auth_token = String::from(
                json["data"]["authToken"]
                    .as_str()
                    .ok_or("data or authToken is missing")?,
            );
            println!("login success. authToken: {:?}", self.auth_token);
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
        let mut rc = RocketChat::new(std::format!("https://{}", RC_SLINT_TEST_URL).as_str(), "");
        rc.login(RC_SLINT_TEST_USER, RC_SLINT_TEST_PWD)
            .await
            .expect("failed");
        assert_eq!(rc.is_logged_in(), true);
    }
}
