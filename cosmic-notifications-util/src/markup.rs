use cosmic::{
    cosmic_theme,
    iced::core::text::Span,
    iced::{
        Font,
        font::{Style, Weight},
    },
};
use std::borrow::Cow;

/// Payload attached to a clickable `<a href>` span: the sanitized URL.
pub type Link = String;

/// URL schemes a body hyperlink is allowed to open. Notification bodies come from any client on
/// the session bus, so anything that could hand an arbitrary local handler a URL (`file:`,
/// `ms-*:`, custom app schemes) stays inert text.
const ALLOWED_SCHEMES: [&str; 3] = ["http", "https", "mailto"];

/// Longest entity body (between `&` and `;`) we bother to look at, e.g. `#x1F600`.
const MAX_ENTITY_LEN: usize = 10;

// Handle break lines, etc. in the future
// Used only in `parse_html` function
fn _prepare_html(text: &str) -> String {
    let text = text
        // handle break lines
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");

    text.to_owned()
}

/// Resolve a single entity body (the text between `&` and `;`) to its character.
///
/// Covers the five XML named entities plus decimal/hex numeric character references. Anything
/// else — including other HTML named entities — returns `None` and is left as literal text.
fn decode_entity(body: &str) -> Option<char> {
    match body {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => {
            let digits = body.strip_prefix('#')?;
            let code = if let Some(hex) = digits.strip_prefix(['x', 'X']) {
                u32::from_str_radix(hex, 16).ok()?
            } else {
                digits.parse::<u32>().ok()?
            };
            let c = char::from_u32(code)?;
            // Control characters would render as garbage or mangle the card layout.
            (!c.is_control() || c == '\n' || c == '\t').then_some(c)
        }
    }
}

/// Decode HTML entities in a text run.
///
/// Runs *after* tag parsing, so a decoded `<` can never open a new tag. Malformed or unknown
/// entities are passed through verbatim rather than dropped.
fn decode_entities(text: &str) -> Cow<'_, str> {
    if !text.contains('&') {
        return Cow::Borrowed(text);
    }

    let mut out = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let after = &rest[amp + 1..];

        let decoded = after
            .as_bytes()
            .iter()
            .take(MAX_ENTITY_LEN + 1)
            .position(|&b| b == b';')
            .and_then(|end| decode_entity(&after[..end]).map(|c| (c, end)));

        if let Some((c, end)) = decoded {
            out.push(c);
            rest = &after[end + 1..];
        } else {
            out.push('&');
            rest = after;
        }
    }

    out.push_str(rest);
    Cow::Owned(out)
}

/// Accept an `href` only if it is an absolute URL in [`ALLOWED_SCHEMES`].
fn sanitize_href(href: &str) -> Option<Link> {
    let url = url::Url::parse(href.trim()).ok()?;
    ALLOWED_SCHEMES
        .contains(&url.scheme())
        .then(|| url.to_string())
}

// Sanitize only tags allowed by Freedesktop Notification Specifications
// https://specifications.freedesktop.org/notification/1.2/markup.html
// TODO: impl <img> tag handling
fn sanitize_html(tags: &[String], link: Option<&Link>, content: &str) -> Span<'static, Link> {
    let mut font = Font::default();
    let mut span = Span::new(content.to_owned());

    for tag in tags {
        match tag.as_str() {
            "b" => font.weight = Weight::Bold,
            "i" => font.style = Style::Italic,
            "u" => span = span.underline(true),
            // Style `<a>` only when it actually opens something, so link styling always means
            // "this is clickable".
            "a" => {
                if let Some(link) = link {
                    let theme = cosmic_theme::Theme::preferred_theme();
                    span = span
                        .underline(true)
                        .color(theme.accent_text_color())
                        .link(link.clone());
                }
            }
            _ => {}
        }
    }

    span.font(font)
}

fn _handle_recursive(
    handle: &tl::NodeHandle,
    parser: &tl::Parser,
    tags: &mut Vec<String>,
    links: &mut Vec<Link>,
    buffer: &mut Vec<Span<'static, Link>>,
) {
    if let Some(node) = handle.get(parser) {
        match node {
            tl::Node::Tag(tag) => {
                let tag_name = tag.name().as_utf8_str();

                let mut pushed_link = false;
                if tag_name == "a"
                    && let Some(href) = tag.attributes().get("href").flatten()
                    && let Some(url) = sanitize_href(&decode_entities(&href.as_utf8_str()))
                {
                    links.push(url);
                    pushed_link = true;
                }
                tags.push(tag_name.into_owned());

                tag.children().top().iter().for_each(|t| {
                    _handle_recursive(t, parser, tags, links, buffer);
                });

                tags.pop();
                if pushed_link {
                    links.pop();
                }
            }
            tl::Node::Raw(bytes) => {
                let raw = bytes.as_utf8_str();
                buffer.push(sanitize_html(tags, links.last(), &decode_entities(&raw)));
            }
            _ => {}
        }
    }
}

pub fn html_to_spans(text: &str) -> Vec<Span<'static, Link>> {
    let mut buffer = Vec::new();
    let html = _prepare_html(text);
    let dom = tl::parse(&html, tl::ParserOptions::default());

    if let Ok(vdom) = dom {
        let parser = vdom.parser();
        let elements = vdom.children();
        let mut tags = Vec::new();
        let mut links = Vec::new();

        for node_handle in elements {
            _handle_recursive(node_handle, parser, &mut tags, &mut links, &mut buffer);
        }
    }

    buffer
}

#[cfg(test)]
mod tests {
    use super::{decode_entities, html_to_spans};

    fn text_of(body: &str) -> String {
        html_to_spans(body)
            .iter()
            .map(|s| s.text.as_ref())
            .collect()
    }

    #[test]
    fn decodes_named_and_numeric_entities() {
        assert_eq!(decode_entities("a &amp; b"), "a & b");
        assert_eq!(decode_entities("&lt;b&gt;"), "<b>");
        assert_eq!(decode_entities("&quot;x&apos;y&quot;"), "\"x'y\"");
        assert_eq!(decode_entities("&#60;&#x3E;"), "<>");
        assert_eq!(decode_entities("no entities here"), "no entities here");
    }

    #[test]
    fn leaves_malformed_entities_alone() {
        assert_eq!(decode_entities("Q&A"), "Q&A");
        assert_eq!(decode_entities("&nbsp;"), "&nbsp;");
        assert_eq!(decode_entities("&amp"), "&amp");
        assert_eq!(decode_entities("&#;"), "&#;");
        assert_eq!(decode_entities("&#x110000;"), "&#x110000;");
        assert_eq!(decode_entities("&#0;"), "&#0;");
        // A stray `&` must not swallow a distant `;`.
        assert_eq!(
            decode_entities("R&D means research; development"),
            "R&D means research; development"
        );
    }

    #[test]
    fn decoded_markup_does_not_become_tags() {
        assert_eq!(text_of("&lt;b&gt;not bold&lt;/b&gt;"), "<b>not bold</b>");
        assert_eq!(html_to_spans("&lt;b&gt;not bold&lt;/b&gt;").len(), 1);
    }

    #[test]
    fn links_carry_their_href() {
        let spans = html_to_spans(r#"see <a href="https://example.com/?a=1&amp;b=2">this</a>"#);
        let link = spans.iter().find(|s| s.text == "this").unwrap();
        assert_eq!(link.link.as_deref(), Some("https://example.com/?a=1&b=2"));
        assert!(link.underline);
        // Text outside the anchor stays unlinked.
        assert!(
            spans
                .iter()
                .find(|s| s.text == "see ")
                .unwrap()
                .link
                .is_none()
        );
    }

    #[test]
    fn rejects_non_web_and_missing_hrefs() {
        for body in [
            r#"<a href="file:///etc/passwd">x</a>"#,
            r#"<a href="javascript:alert(1)">x</a>"#,
            r#"<a href="/relative">x</a>"#,
            "<a>x</a>",
        ] {
            let spans = html_to_spans(body);
            let span = spans.iter().find(|s| s.text == "x").unwrap();
            assert!(span.link.is_none(), "{body} should not be clickable");
            assert!(!span.underline, "{body} should not be styled as a link");
        }
    }

    #[test]
    fn nested_markup_keeps_the_link() {
        let spans = html_to_spans(r#"<a href="http://a.test"><b>bold link</b></a>"#);
        let span = spans.iter().find(|s| s.text == "bold link").unwrap();
        assert_eq!(span.link.as_deref(), Some("http://a.test/"));
    }
}
