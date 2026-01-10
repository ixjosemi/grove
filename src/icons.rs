use std::collections::HashMap;
use std::sync::LazyLock;

pub static ICONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    // Directories
    m.insert("dir_open", "󰉋 ");
    m.insert("dir_closed", "󰉖 ");

    // Programming languages
    m.insert("rs", " ");
    m.insert("py", " ");
    m.insert("js", " ");
    m.insert("ts", " ");
    m.insert("jsx", " ");
    m.insert("tsx", " ");
    m.insert("go", " ");
    m.insert("rb", " ");
    m.insert("php", " ");
    m.insert("java", " ");
    m.insert("c", " ");
    m.insert("cpp", " ");
    m.insert("h", " ");
    m.insert("hpp", " ");
    m.insert("cs", "󰌛 ");
    m.insert("swift", " ");
    m.insert("kt", " ");
    m.insert("scala", " ");
    m.insert("hs", " ");
    m.insert("lua", " ");
    m.insert("vim", " ");
    m.insert("sh", " ");
    m.insert("bash", " ");
    m.insert("zsh", " ");
    m.insert("fish", " ");

    // Web
    m.insert("html", " ");
    m.insert("css", " ");
    m.insert("scss", " ");
    m.insert("sass", " ");
    m.insert("less", " ");
    m.insert("vue", " ");
    m.insert("svelte", " ");

    // Data/Config
    m.insert("json", " ");
    m.insert("yaml", " ");
    m.insert("yml", " ");
    m.insert("toml", " ");
    m.insert("xml", "󰗀 ");
    m.insert("csv", " ");
    m.insert("sql", " ");

    // Documentation
    m.insert("md", " ");
    m.insert("txt", " ");
    m.insert("pdf", " ");
    m.insert("doc", "󰈬 ");
    m.insert("docx", "󰈬 ");

    // Images
    m.insert("png", " ");
    m.insert("jpg", " ");
    m.insert("jpeg", " ");
    m.insert("gif", " ");
    m.insert("svg", "󰜡 ");
    m.insert("ico", " ");
    m.insert("webp", " ");

    // Archives
    m.insert("zip", " ");
    m.insert("tar", " ");
    m.insert("gz", " ");
    m.insert("rar", " ");
    m.insert("7z", " ");

    // Git
    m.insert("git", " ");
    m.insert("gitignore", " ");

    // Docker
    m.insert("dockerfile", " ");
    m.insert("docker", " ");

    // Misc
    m.insert("lock", " ");
    m.insert("env", " ");
    m.insert("log", " ");

    // Default
    m.insert("default", " ");
    m
});

pub fn get_icon(filename: &str, is_dir: bool, is_expanded: bool) -> &'static str {
    if is_dir {
        return if is_expanded {
            ICONS.get("dir_open").unwrap_or(&"󰉋 ")
        } else {
            ICONS.get("dir_closed").unwrap_or(&"󰉖 ")
        };
    }

    // Check special filenames first
    let lower_name = filename.to_lowercase();
    if lower_name == "dockerfile" {
        return ICONS.get("dockerfile").unwrap_or(&" ");
    }
    if lower_name.contains(".git") {
        return ICONS.get("git").unwrap_or(&" ");
    }
    if lower_name.ends_with(".lock") {
        return ICONS.get("lock").unwrap_or(&" ");
    }

    // Get extension
    let ext = filename
        .rsplit('.')
        .next()
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    ICONS.get(ext.as_str()).unwrap_or(ICONS.get("default").unwrap_or(&" "))
}
