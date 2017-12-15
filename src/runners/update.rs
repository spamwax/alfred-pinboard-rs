use super::*;

pub fn run() {
    let config = Config::setup().unwrap_or_else(|err| {
        show_error_alfred(&err);
        process::exit(1);
    });

    let mut pinboard = Pinboard::new(config.auth_token.as_ref()).unwrap_or_else(|err| {
        show_error_alfred(&err);
        process::exit(1);
    });
    pinboard.update_cache().unwrap_or_else(|err| {
        show_error_alfred(&err);
    });
    io::stdout().write(b"Successfully update cache files!").unwrap();
}
