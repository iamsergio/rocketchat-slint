// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Sergio Martins

use std::rc::Rc;

use slint::SharedString;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let ui_weak = ui.as_weak();
    ui.set_usernameText(SharedString::from(env!("RC_SLINT_TEST_USER")));
    ui.set_passwordText(SharedString::from(env!("RC_SLINT_TEST_PWD")));

    pub const RC_SLINT_TEST_URL: &str = env!("RC_SLINT_TEST_URL");

    let rc = Rc::new(rocketchat::RocketChat::new(
        std::format!("https://{}", RC_SLINT_TEST_URL).as_str(),
        rocketchat::RocketChat::saved_auth_token().as_str(),
    ));

    let result = rc.login_via_token().await;
    if let Ok(success) = result {
        ui.set_logged_in(success);
    }

    ui.on_request_login(move |username: SharedString, password: SharedString| {
        println!("login requested");

        let rc = Rc::clone(&rc);

        let ui_weak = ui_weak.clone();
        slint::spawn_local(async move {
            let result = rc.login(&username, &password).await;
            ui_weak
                .upgrade_in_event_loop(move |ui| {
                    ui.set_logged_in(result.is_ok());
                    if let Err(e) = result {
                        println!("slint: login failed: {}", e);
                    }
                })
                .unwrap();
        })
        .unwrap();
    });

    ui.run()
}
