#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use crate::client::ExampleClientPlugin;
use crate::server::ExampleServerPlugin;
use crate::shared::SharedPlugin;
use bevy::prelude::*;
use lightyear_examples_common::app::Apps;
use lightyear_examples_common::settings::{Settings};

mod client;
mod protocol;
mod server;
mod shared;
mod settings;
mod renderer;

fn main() {
    let cli = lightyear_examples_common::app::cli();
    let settings = settings::get_settings();
    // build the bevy app (this adds common plugin such as the DefaultPlugins)
    let mut apps = Apps::new(settings, cli, "example".to_string());
    // add `ClientPlugins` and `ServerPlugins` plugin groups
    apps.add_lightyear_plugins();

    #[cfg(feature = "gui")]
    apps.add_user_renderer_plugin(renderer::ExampleRendererPlugin);

    apps.add_user_shared_plugin(shared::SharedPlugin);
    #[cfg(feature = "client")]
    apps.add_user_client_plugin(client::ExampleClientPlugin);
    #[cfg(feature = "server")]
    apps.add_user_server_plugin(server::ExampleServerPlugin);

    // run the app
    apps.run();
}
