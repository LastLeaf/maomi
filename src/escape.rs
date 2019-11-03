use std::borrow::Cow;

pub fn escape_html<'a>(s: &'a str) -> Cow<'a, str> {
    let need_escape_from = s.chars().position(|c| {
        match c {
            '<' => true,
            '>' => true,
            '"' => true,
            '&' => true,
            _ => false,
        }
    });
    match need_escape_from {
        None => Cow::Borrowed(s),
        Some(p) => {
            let mut ret = String::new();
            for c in s.chars().skip(p) {
                match c {
                    '<' => ret += "&lt;",
                    '>' => ret += "&gt;",
                    '"' => ret += "&quot;",
                    '&' => ret += "&amp;",
                    _ => ret.push(c),
                }
            }
            Cow::Owned(ret)
        }
    }
}
