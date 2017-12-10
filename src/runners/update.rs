use super::*;

pub fn run() {
    let (config, pinboard) = Config::setup().unwrap_or_else(|err| {
        show_error_alfred(&err);
        process::exit(1);
    });

    pinboard.update_cache().unwrap_or_else(|err| {
        show_error_alfred(&err);
    });
    io::stdout().write(b"Successfully update cache files!").unwrap();
}
