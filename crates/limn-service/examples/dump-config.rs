//! Debug surface for the config load/save path.
//!
//! Prints the resolved config path and the loaded config (default if the
//! file is missing). Set `LIMN_CONFIG_WRITE_DEFAULT=1` to write the
//! default config to the resolved path — a way to eyeball the on-disk
//! TOML round-trip without launching the app.
//!
//! ```sh
//! cargo run -p limn-service --example dump-config
//! LIMN_CONFIG_WRITE_DEFAULT=1 cargo run -p limn-service --example dump-config
//! ```

use limn_service::Config;

fn main() {
    let path = Config::config_path();
    println!("config path: {}", path.display());

    match Config::load() {
        Ok(config) => println!("loaded config: {config:#?}"),
        Err(e) => {
            eprintln!("failed to load config: {e}");
            std::process::exit(1);
        }
    }

    if std::env::var("LIMN_CONFIG_WRITE_DEFAULT").as_deref() == Ok("1") {
        match Config::default().save_to(&path) {
            Ok(()) => println!("wrote default config to {}", path.display()),
            Err(e) => {
                eprintln!("failed to write default config: {e}");
                std::process::exit(1);
            }
        }
    }
}
