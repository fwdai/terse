pub fn banner() -> String {
    const INNER_WIDTH: usize = 46;

    const RESET: &str = "\x1b[0m";
    const FRAME: &str = "\x1b[38;5;45m";
    const TITLE: &str = "\x1b[1;38;5;231m";
    const SUBTLE: &str = "\x1b[2;38;5;250m";

    fn line(content_colored: &str, content_visible_len: usize) -> String {
        const INNER_WIDTH: usize = 46;
        const RESET: &str = "\x1b[0m";
        const FRAME: &str = "\x1b[38;5;45m";

        let pad = INNER_WIDTH.saturating_sub(content_visible_len);
        format!(
            "{FRAME}┃{RESET}{content_colored}{}{FRAME}┃{RESET}\n",
            " ".repeat(pad)
        )
    }

    let mut out = String::new();
    out.push_str(&format!(
        "{FRAME}┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓{RESET}\n"
    ));
    out.push_str(&line(" ".repeat(INNER_WIDTH).as_str(), INNER_WIDTH));

    let title_plain = "            T  ·  E  ·  R  ·  S  ·  E";
    let title_visible_len = title_plain.chars().count();
    let title_colored = format!(
        "            {TITLE}T  ·  E  ·  R  ·  S  ·  E{RESET}"
    );
    out.push_str(&line(&title_colored, title_visible_len));
    out.push_str(&line(" ".repeat(INNER_WIDTH).as_str(), INNER_WIDTH));

    let version = env!("CARGO_PKG_VERSION");
    let version_plain = format!("                      v{version}               ");
    let version_visible_len = version_plain.chars().count();
    let version_colored = format!("                    {SUBTLE}v{version}{RESET}                 ");
    out.push_str(&line(&version_colored, version_visible_len));
    out.push_str(&line(" ".repeat(INNER_WIDTH).as_str(), INNER_WIDTH));

    out.push_str(&format!(
        "{FRAME}┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛{RESET}\n"
    ));
    out
}
