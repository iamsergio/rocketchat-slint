// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Sergio Martins

use std::rc::Rc;

mod login_controller;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    pub const RC_SLINT_TEST_URL: &str = env!("RC_SLINT_TEST_URL");

    let rc = Rc::new(rocketchat::RocketChat::new(
        std::format!("https://{}", RC_SLINT_TEST_URL).as_str(),
        rocketchat::RocketChat::saved_auth_token().as_str(),
    ));

    let login_controller = login_controller::Controller::new(ui.clone_strong(), Rc::clone(&rc));
    login_controller.login_via_saved_token().await;

    ui.run()
}
