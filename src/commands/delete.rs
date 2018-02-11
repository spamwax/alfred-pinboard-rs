use super::*;
use std::io::Write;
use super::browser_info;

pub fn run(cmd: SubCommand, config: Config, pinboard: Pinboard) {
    let _ = config; // To silent compiler.
    info!("Starting in run");
    let tag = match cmd {
        SubCommand::Delete { tag } => tag,
        _ => unreachable!(),
    };

    if let Some(tag) = tag {
        unimplemented!(
            "deleting a tag {:?} is not supported by rusty-pin yet!",
            tag
        );
    } else {
        let browser_tab_info = browser_info::get().unwrap_or_else(|e| {
            io::stdout()
                .write(format!("Error: {}", e).as_ref())
                .expect("Couldn't write to stdout");
            process::exit(1);
        });
        if let Err(e) = pinboard.delete(&browser_tab_info.url) {
            io::stdout()
                .write(format!("Error: {}", e).as_ref())
                .expect("Couldn't write to stdout");
            process::exit(1);
        }
    }
}
