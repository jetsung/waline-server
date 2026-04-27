use comrak::{Options, markdown_to_html};

pub fn render(content: &str) -> String {
    let mut opts = Options::default();
    opts.extension.strikethrough = true;
    opts.extension.table = true;
    opts.extension.autolink = true;
    opts.extension.tasklist = true;
    opts.extension.superscript = true;
    opts.render.escape = true; // Replace unsafe_ with escape for newer Comrak

    let html = markdown_to_html(content, &opts);

    // XSS sanitization via ammonia
    ammonia::clean(&html)
}
