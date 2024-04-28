// SPDX-License-Identifier: MIT
// SPDX-FileCopyrightText: Sergio Martins

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    ui.run()
}
