use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "alfred-pinboard")]
/// the stupid content tracker
pub struct Commands {
    #[structopt(name = "debug", default_value = "1", required = false, long = "debug")]
    debug_level: i8,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "config")]
    /// Configure options and settings of interacting with API and searching items.
    Config {
        /// Show all the configuration settings, after setting any given config options.
        #[structopt(name = "display", short = "d", long = "display")]
        display: bool,
        /// Set API authorization token.
        /// (Obtain it from your Pinboard account's setting page).
        #[structopt(name = "auth", long = "authorization", short = "a")]
        auth_token: Option<String>,
        /// Number of bookmarks to show in Alfred's window.
        #[structopt(long = "bookmark-numbers", short = "p")]
        number_pins: Option<u8>,
        /// Number of tags to show in Alfred's window.
        #[structopt(long = "tag-numbers", short = "l")]
        number_tags: Option<u8>,
        /// By default, make all new bookmarks public/shared.
        #[structopt(name = "shared", short = "s", long = "shared")]
        shared: bool,
        /// When searching tags/bookmarks, enable 'fuzzy' searching. (similar to `selecta`)
        #[structopt(name = "fuzzy", short = "f", long = "fuzzy")]
        fuzzy: bool,
        /// When searching, only look up query in 'tag' field of bookmarks.
        #[structopt(name = "tags-only", short = "t", long = "tags-only")]
        tags_only: bool,
        /// After posting a bookmark to Pinboard, update the local cache files.
        #[structopt(name = "auto-update", short = "u", long = "auto-update")]
        auto_update: bool,
        /// When posting a new bookmark, show 3 popular tags for the URL (if available).
        #[structopt(name = "suggest_tags", short = "o", long = "suggest-tags")]
        suggest_tags: bool,
    },
//    #[structopt(name = "sparkle")]
//    /// Add magical sparkles -- the secret ingredient!
//    Sparkle {
//        #[structopt(name = "gooz", short = "g", required = false)]
//        gooz: String,
//        #[structopt(short = "m")]
//        magicality: bool,
//        #[structopt(name = "color")]
//        color: Option<String>,
//        #[structopt(name = "zard")]
//        zard: Option<String>,
//    },
//    /// Some help
//    #[structopt(name = "finish")]
//    Finish {
//        #[structopt(short = "t")]
//        time: u32,
//        #[structopt(subcommand)] // Note that we mark a field as a subcommand
//        mtype: FinishType,
//    },
}

//#[derive(StructOpt, Debug)]
//enum FinishType {
//    #[structopt(name = "glaze")]
//    Glaze { applications: u32 },
//    #[structopt(name = "powder")]
//    Powder { flavor: String, dips: Option<u32> },
//}
