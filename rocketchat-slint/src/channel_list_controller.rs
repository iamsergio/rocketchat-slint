// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Sergio Martins

use crate::slint_generatedAppWindow::AppWindow;
use crate::slint_generatedAppWindow::Channel;
use slint::SharedString;
use std::rc::Rc;

pub struct Controller {
    ui: AppWindow,
    model: Rc<rocketchat::RocketChat>,
}

pub fn on_login_changed(controller: Rc<Controller>) {
    if controller.model.is_logged_in() {
        let controller = controller.clone();
        slint::spawn_local(async move {
            controller.load_channel_list().await;
        })
        .unwrap();
    }
}

impl Controller {
    pub fn new(ui: AppWindow, model: Rc<rocketchat::RocketChat>) -> Rc<Self> {
        Rc::new(Self { ui, model })
    }

    pub async fn load_channel_list(&self) {
        self.model.list_joined_channels().await;
        let channels = self.model.get_joined_channels();
        let mut ui_channels: Vec<Channel> = Vec::new();
        for c in channels {
            ui_channels.push(Channel {
                id: SharedString::from(c.id.clone()),
                name: SharedString::from(c.name.clone()),
            });
        }
        let channel_model = Rc::new(slint::VecModel::from(ui_channels));
        self.ui.set_channelModel(channel_model.into());
    }
}
