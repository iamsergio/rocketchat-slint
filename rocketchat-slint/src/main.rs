// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Sergio Martins

use std::{cell::RefCell, rc::Rc};

use slint::SharedString;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let ui_weak = ui.as_weak();

    pub const RC_SLINT_TEST_URL: &str = env!("RC_SLINT_TEST_URL");

    let rc = Rc::new(RefCell::new(rocketchat::RocketChat::new(
        std::format!("https://{}", RC_SLINT_TEST_URL).as_str(),
        "",
    )));

    ui.on_request_login(move |username: SharedString, password: SharedString| {
        println!("login requested");

        let rc = Rc::clone(&rc);

        let ui_weak = ui_weak.clone();
        slint::spawn_local(async move {
            let result = rc.borrow_mut().login(&username, &password).await;
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
