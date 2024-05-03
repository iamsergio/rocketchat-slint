// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Sergio Martins

use crate::slint_generatedAppWindow::AppWindow;
use slint::{ComponentHandle, SharedString};
use std::rc::Rc;

pub struct Controller {
    ui: AppWindow,
    model: Rc<rocketchat::RocketChat>,
}

impl Controller {
    pub fn new(ui: AppWindow, model: Rc<rocketchat::RocketChat>) -> Rc<Self> {
        ui.set_usernameText(SharedString::from(env!("RC_SLINT_TEST_USER")));
        ui.set_passwordText(SharedString::from(env!("RC_SLINT_TEST_PWD")));

        let controller = Rc::new(Self { ui, model });

        let controller_copy = controller.clone();
        controller
            .ui
            .on_request_login(move |username: SharedString, password: SharedString| {
                controller_copy.login(username, password);
            });

        controller
    }

    fn login(&self, username: SharedString, password: SharedString) {
        println!("login requested");

        let model = self.model.clone();
        let ui = self.ui.clone_strong();
        slint::spawn_local(async move {
            let result = model.login(&username, &password).await;
            Self::set_logged_in(ui, model, result.is_ok()).await;
            if let Err(e) = result {
                println!("slint: login failed: {}", e);
            }
        })
        .unwrap();
    }

    pub async fn login_via_saved_token(&self) {
        let result = self.model.login_via_saved_token().await;
        Controller::set_logged_in(
            self.ui.clone_strong(),
            self.model.clone(),
            result.is_ok() && result.unwrap(),
        )
        .await;
    }

    async fn set_logged_in(ui: AppWindow, model: Rc<rocketchat::RocketChat>, logged_in: bool) {
        ui.set_logged_in(logged_in);
        if logged_in {
            model.list_joined_channels().await;
        }
    }
}
