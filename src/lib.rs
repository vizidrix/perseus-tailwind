//! This is a simple plugin for Perseus that runs the Tailwind CLI at build time.
//! 
//! Starting point derived from https://github.com/wingertge/perseus-tailwind
//! * Removed automatic installation of TailwindCli so this must exsist on the build env
//! * Updated to work with the latest Perseus 0.4.0-beta.14
//! 
//! It will look for class names in Rust files in `src` and HTML files in `static`.
//! Further configuration can be done as usual in `tailwind.config.js`.
//!
//! # Usage
//!
//! Add the plugin to you Perseus App in your Perseus main function.
//!
//! ```
//! # use perseus::PerseusApp;
//! # use perseus::plugins::Plugins;
//! PerseusApp::new()
//!     .plugins(Plugins::new().plugin(
//!         perseus_tailwind::get_tailwind_plugin,
//!         perseus_tailwind::TailwindOptions {
//!             in_file: "src/tailwind.css".into(),
//!             // Don't put this in /static, it will trigger build loops.
//!             // Put this in /dist and use a static alias instead.
//!             out_file: "dist/static/tailwind.css".into(),
//!         },
//!     ))
//!     .static_alias("/static/tailwind.css", "dist/static/tailwind.css")
//! # ;
//! ```
//!
//! If you're already using plugins just add the plugin to your `Plugins` as usual.
//!
//! # Stability
//!
//! The plugin is fairly simple and shouldn't break anything since it just executes the Tailwind CLI.

#[cfg(engine)]
use perseus::plugins::PluginAction;
use perseus::plugins::{empty_control_actions_registrar, Plugin, PluginEnv};
#[cfg(engine)]
use std::{fs::File, io::Write, path::PathBuf, process::Command};

static PLUGIN_NAME: &str = "tailwind-plugin";

/// Options for the Tailwind CLI
#[derive(Debug)]
pub struct TailwindOptions {
    /// The path to the input CSS file
    pub in_file: String,
    /// The path to the CSS file output by the CLI.\
    /// **DO NOT PUT THIS IN `/static` UNLESS YOU LIKE BUILD LOOPS!**\
    /// Always put it somewhere in `/dist` use static aliases instead.\
    pub out_file: String,
}

/// The plugin constructor
pub fn get_tailwind_plugin() -> Plugin<TailwindOptions> {
    #[allow(unused_mut)]
    Plugin::new(
        PLUGIN_NAME,
        |mut actions| {
            #[cfg(engine)]
            {
                actions
                    .build_actions
                    .before_build
                    .register_plugin(PLUGIN_NAME, |_, data| {
                        if let Some(options) = data.downcast_ref::<TailwindOptions>() {
                            try_run_tailwind(options);
                            return Ok(())
                        } else {
                            unreachable!()
                        }
                    });
                actions
                    .export_actions
                    .before_export
                    .register_plugin(PLUGIN_NAME, |_, data| {
                        if let Some(options) = data.downcast_ref::<TailwindOptions>() {
                            try_run_tailwind(options);
                            return Ok(())
                        } else {
                            unreachable!()
                        }
                    });
            }
            actions
        },
        empty_control_actions_registrar,
        PluginEnv::Server,
    )
}

#[cfg(engine)]
fn try_run_tailwind(options: &TailwindOptions) {
    if !PathBuf::from("tailwind.config.js").exists() {
        init_tailwind();
    }

    let mut args = vec!["build", &options.in_file, "-o", &options.out_file];
    if cfg!(not(debug_assertions)) {
        args.push("-m");
        args.push("-p");
    }

    let output = Command::new(format!("tailwindcli"))
        .args(args)
        .output()
        .expect("Failed to run Tailwind CLI");
    let output = String::from_utf8_lossy(&output.stderr);
    // Errors always contain a JSON object. Please start using result codes Tailwind
    // Also, don't write info messages to stderr instead of stdout
    // Also if you're going to print JSON make the whole thing JSON and not some exception stack
    // trace syntax followed by JSON
    if output.contains('}') {
        panic!("{}", output);
    }
}

#[cfg(engine)]
fn init_tailwind() {
    log::info!(
        "Initializing Tailwind to search all Rust files in 'src' and all HTML files in 'static'."
    );
    let default_config = include_bytes!("default-config.js");
    let mut config = File::create("tailwind.config.js").expect("Failed to create config file");
    config
        .write_all(default_config)
        .expect("Failed to write default config");
}
