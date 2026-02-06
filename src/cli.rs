use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "alfred-chrome-bookmarks")]
pub struct Opt {
    #[structopt(subcommand)]
    pub cmd: SubCommand,
}

#[derive(StructOpt, Debug)]
pub enum SubCommand {
    /// 搜索书签
    #[structopt(name = "search", alias = "s")]
    Search {
        /// 搜索关键词
        query: Vec<String>,

        /// 按tags过滤（逗号分隔，AND查询）
        #[structopt(short = "t", long = "tags")]
        tags: Option<String>,

        /// 按目录过滤（逗号分隔，支持多级目录，如 work/project）
        #[structopt(short = "p", long = "folders")]
        folders: Option<String>,

        /// 使用模糊搜索
        #[structopt(short = "f", long = "fuzzy")]
        fuzzy: bool,

        /// 限制结果数量
        #[structopt(short = "l", long = "limit", default_value = "50")]
        limit: usize,
    },

    /// 为书签添加tags
    #[structopt(name = "tag", alias = "t")]
    Tag {
        /// 书签URL或ID
        bookmark: String,

        /// 要添加的tags（空格分隔）
        tags: Vec<String>,
    },

    /// 从书签删除tag
    #[structopt(name = "untag", alias = "ut")]
    Untag {
        /// 书签URL或ID
        bookmark: String,

        /// 要删除的tag
        tag: String,
    },

    /// 列出所有tags
    #[structopt(name = "tags", alias = "lt")]
    ListTags {
        /// 搜索tag的前缀
        prefix: Option<String>,
    },

    /// 显示书签的tags
    #[structopt(name = "show", alias = "sh")]
    ShowTags {
        /// 书签URL或ID
        bookmark: String,
    },

    /// 重命名tag
    #[structopt(name = "rename", alias = "r")]
    RenameTag {
        /// 旧tag名称
        old_tag: String,

        /// 新tag名称
        new_tag: String,
    },

    /// 刷新浏览器书签缓存
    #[structopt(name = "refresh", alias = "rf")]
    Refresh,

    /// 显示统计信息
    #[structopt(name = "stats", alias = "st")]
    Stats,
}
