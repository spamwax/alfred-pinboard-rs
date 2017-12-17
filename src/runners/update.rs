use super::*;

pub fn run(mut pinboard: Pinboard) {
    pinboard.update_cache().unwrap_or_else(|err| {
        ::show_error_alfred(&err);
    });
    io::stdout().write(b"Successfully update cache files!").unwrap();
}
