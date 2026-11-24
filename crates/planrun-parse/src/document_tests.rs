use super::*;
use crate::source;

fn doc(text: &str) -> Document {
    let source = source::load("t.md", text.as_bytes()).unwrap_or_else(|e| panic!("{e:?}"));
    Document::build(source).unwrap_or_else(|e| panic!("{e:?}"))
}

fn errors(text: &str) -> Vec<Diagnostic> {
    let source = source::load("t.md", text.as_bytes()).unwrap_or_else(|e| panic!("{e:?}"));
    match Document::build(source) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics.0,
    }
}

/// Each block as `depth kind text`, the text resolved.
fn shape(doc: &Document) -> Vec<String> {
    doc.blocks
        .iter()
        .map(|b| {
            let what = match &b.block {
                Block::Heading { level, text } => format!("h{level} {}", doc.text(text)),
                Block::Paragraph { text } => format!("p {}", doc.text(text)),
                Block::ListItem { text, task_box } => {
                    format!("li{task_box:?} {}", doc.text(text))
                }
                Block::Fence {
                    info,
                    body,
                    fence_len,
                } => format!("fence{fence_len} {} {:?}", doc.text(info), doc.text(body)),
                Block::Table { text } => format!("table {}", doc.text(text)),
                Block::Quote { text } => format!("quote {}", doc.text(text)),
                Block::Rule => "rule".to_owned(),
                Block::Html { text } => format!("html {}", doc.text(text)),
            };
            format!("{} {what}", b.depth)
        })
        .collect()
}

#[test]
fn blocks_keep_their_raw_text() {
    let d = doc("# T\n\n**Step 1 — write `x`**\n\n- Create `a.rs`\n- Modify `b.rs`\n\n---\n");
    assert_eq!(
        shape(&d),
        [
            "0 h1 T",
            "0 p **Step 1 — write `x`**",
            "0 liNone Create `a.rs`",
            "0 liNone Modify `b.rs`",
            "0 rule",
        ]
    );
    assert_eq!(d.source.slice(d.blocks[1].span), "**Step 1 — write `x`**");
}

#[test]
fn a_longer_fence_contains_shorter_fences() {
    let d = doc("````markdown\n```rust\nfn a() {}\n```\n````\n");
    assert_eq!(
        shape(&d),
        ["0 fence4 markdown \"```rust\\nfn a() {}\\n```\\n\""]
    );
}

#[test]
fn a_fence_in_a_list_item_is_rebuilt() {
    let d = doc("- Run it:\n\n  ```powershell\n  cargo test\n    indented\n  ```\n\nAfter.\n");
    assert_eq!(
        shape(&d),
        [
            "0 liNone Run it:",
            "1 fence3 powershell \"cargo test\\n  indented\\n\"",
            "0 p After.",
        ]
    );
    let Block::Fence { body, .. } = &d.blocks[1].block else {
        panic!("not a fence");
    };
    assert!(matches!(body, TextRef::Owned { .. }));
}

#[test]
fn a_top_level_fence_body_is_a_span() {
    let d = doc("```rust\nfn a() {}\n\nfn b() {}\n```\n");
    let Block::Fence { body, .. } = &d.blocks[0].block else {
        panic!("not a fence");
    };
    assert!(matches!(body, TextRef::Span(_)));
    assert_eq!(d.text(body), "fn a() {}\n\nfn b() {}\n");
}

#[test]
fn crlf_input_gives_lf_bodies() {
    let d = doc("```text\r\na\r\nb\r\n```\r\n");
    assert_eq!(shape(&d), ["0 fence3 text \"a\\nb\\n\""]);
}

#[test]
fn an_empty_fence_has_an_empty_body() {
    let d = doc("```diff\n```\n\nNext.\n");
    assert_eq!(shape(&d), ["0 fence3 diff \"\"", "0 p Next."]);
}

#[test]
fn task_items_quotes_tables_and_html() {
    let d = doc(
        "<!-- expect: x -->\n\n> **Note:** a\n> b\n\n| a | b |\n|---|---|\n| 1 | 2 |\n\n\
         - [ ] **Step 1: go**\n- [x] done\n",
    );
    assert_eq!(
        shape(&d),
        [
            "0 html <!-- expect: x -->",
            "0 quote > **Note:** a\n> b",
            "0 table | a | b |\n|---|---|\n| 1 | 2 |",
            "0 liSome(false) **Step 1: go**",
            "0 liSome(true) done",
        ]
    );
}

#[test]
fn nested_items_are_one_deeper() {
    let d = doc("1. outer\n   - inner\n\n     inner text\n2. next\n");
    assert_eq!(
        shape(&d),
        [
            "0 liNone outer",
            "1 liNone inner",
            "2 p inner text",
            "0 liNone next",
        ]
    );
}

#[test]
fn an_indented_code_block_is_a_fence_of_length_zero() {
    let d = doc("Para.\n\n    code\n");
    assert_eq!(shape(&d), ["0 p Para.", "0 fence0  \"code\\n\""]);
}

#[test]
fn a_fence_left_open_is_an_error() {
    let found = errors("# T\n\n```rust\nfn a() {}\n");
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].code, "unclosed-fence");
    assert_eq!(found[0].span, Some(Span::new(5, 12)));
}

#[test]
fn a_fence_closed_by_the_next_blocks_fence_is_an_error() {
    let text = "```rust\n// never closed\n\n**Step 2**\n\n```powershell\ncargo test\n```\n";
    let found = errors(text);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].code, "unclosed-fence");
    assert_eq!(found[0].span, Some(Span::new(0, 7)));
    let related = found[0].related.first().map(|(span, _)| *span);
    assert_eq!(
        related.map(|s| &text[s.start as usize..s.end as usize]),
        Some("```powershell")
    );
}

#[test]
fn shorter_or_bare_inner_fences_are_legal() {
    assert!(errors("````markdown\n```rust\nx\n```\n````\n").is_empty());
    assert!(errors("```text\n~~~rust\nx\n```\n").is_empty());
    assert!(errors("```text\n    ```rust\n```\n").is_empty());
}
