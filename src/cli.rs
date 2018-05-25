#[derive(StructOpt, Debug)]
#[structopt(name = "alfred-pinboard")]
/// Command line component of Alfred Workflow for Pinboard (Written in Rust!)
pub struct Opt {
    #[structopt(name = "debug", default_value = "0", long = "debug")]
    pub debug_level: i8,
    #[structopt(subcommand)]
    pub cmd: SubCommand,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    #[structopt(name = "config")]
    /// Configures options and settings of interacting with API and searching items.
    Config {
        /// Show all the configuration settings, after setting any given config options.
        #[structopt(name = "display", short = "d", long = "display")]
        display: bool,

        /// Set API authorization token.
        /// (Obtain it from your Pinboard account's setting page).
        #[structopt(name = "auth", long = "authorization", short = "a")]
        auth_token: Option<String>,

        /// Number of bookmarks to show in Alfred's window. [default: 10]
        #[structopt(long = "bookmark-numbers", short = "p")]
        number_pins: Option<u8>,

        /// Number of tags to show in Alfred's window. [default: 10]
        #[structopt(long = "tag-numbers", short = "l")]
        number_tags: Option<u8>,

        /// By default, make all new bookmarks public/shared. [default: false]
        #[structopt(name = "shared", short = "s", long = "shared")]
        shared: Option<bool>,

        /// By default, set all new bookmarks' toread flag. [default: false]
        #[structopt(name = "toread", short = "r", long = "toread")]
        toread: Option<bool>,

        /// When searching tags/bookmarks, enable 'fuzzy' searching. (similar to `selecta`) [default: false]
        #[structopt(name = "fuzzy", short = "f", long = "fuzzy")]
        fuzzy: Option<bool>,

        /// When searching, only look up query in 'tag' field of bookmarks. [default: false]
        #[structopt(name = "tags_only", short = "t", long = "tags-only")]
        tags_only: Option<bool>,

        /// After posting a bookmark to Pinboard, update the local cache files. [default: true]
        #[structopt(name = "auto_update", short = "u", long = "auto-update")]
        auto_update: Option<bool>,

        /// When posting a new bookmark, show 3 popular tags for the URL (if available). [default: true]
        #[structopt(name = "suggest_tags", short = "o", long = "suggest-tags")]
        suggest_tags: Option<bool>,
    },
    #[structopt(name = "list")]
    /// Lists all bookmarks (default) or tags.
    List {
        /// Only list tags
        #[structopt(name = "tags", long = "tags", short = "t")]
        tags: bool,
        /// Retrieve suggestion for tags from Pinboard API. Will be ignored if user is not listing
        /// tags.
        #[structopt(name = "suggest", long = "suggest", short = "s")]
        suggest: Option<bool>,
        /// Optional query word used to narrow the output list.
        /// Only works with --tags option! To narrow down bookmarks, use `search` sub-command
        query: Option<String>,
    },
    #[structopt(name = "post")]
    /// Creates a bookmark for the current page of the active browser.
    Post {
        /// Space-delimited list of tags for the url
        #[structopt(name = "tags", long = "tags", short = "t")]
        tags: Vec<String>,
        /// Extra description note for the url
        #[structopt(name = "description", long = "description", short = "d")]
        description: Option<String>,
        /// Mark this bookmark shared (overrides user's settings)
        #[structopt(name = "shared", long = "shared", short = "s")]
        shared: Option<bool>,
        /// Mark this bookmark as toread (overrides user's settings)
        #[structopt(name = "toread", long = "toread", short = "b")]
        toread: Option<bool>,
    },
    #[structopt(name = "delete")]
    /// Deletes a bookmark for the current page of the active browser, or a given tag.
    Delete {
        /// Url/bookmark to be deleted.
        /// If not given, the bookmark for active browser's tab will be returned.
        #[structopt(name = "url")]
        url: Option<String>,
    },
    #[structopt(name = "search")]
    /// Searches bookmarks.
    Search {
        /// Only search within tags, can be combined with other flags.
        #[structopt(name = "tags", long = "tags", short = "t")]
        tags: bool,

        /// Only search within title field, can be combined with other flags.
        #[structopt(name = "title", long = "title", short = "T")]
        title: bool,

        /// Only search within description field, can be combined with other flags.
        #[structopt(name = "description", long = "description", short = "d")]
        description: bool,

        /// Only search within url field, can be combined with other flags.
        #[structopt(name = "url", long = "url", short = "u")]
        url: bool,

        /// Query string to look for in all fields of bookmarks, unless modified by -t, -T or -u
        /// flags (space delimited). Bookmarks that have all of query strings will be
        /// returned.
        #[structopt(name = "query")]
        query: Vec<String>,
    },

    /// Update Workflow's cache by doing a full download from Pinboard.
    #[structopt(name = "update")]
    Update,

    /// Check for or download the latest version of this workflow
    #[structopt(name = "self")]
    SelfUpdate {
        /// Check if a new version is available
        #[structopt(name = "check", short = "c")]
        check: bool,

        /// Download the latest version of thir workflow and save it to its cache folder
        #[structopt(name = "download", short = "d")]
        download: bool,
    },
}
