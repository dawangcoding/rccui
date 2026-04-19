use leptos::prelude::*;
use leptos_fluent::tr;

/// Renders a unified diff with color coding.
#[component]
pub fn DiffViewer(
    /// The raw diff text content
    #[prop(into)]
    diff: String,
    /// Whether the diff was truncated
    #[prop(default = false)]
    truncated: bool,
) -> impl IntoView {
    let lines = parse_diff_lines(&diff);

    view! {
        <div class="flex flex-col text-xs font-mono">
            {if truncated {
                view! {
                    <div class="px-3 py-1.5 text-warning bg-warning/10 border-b border-border text-xs">
                        {tr!("git-diff-truncated")}
                    </div>
                }.into_any()
            } else {
                view! {}.into_any()
            }}
            <pre class="overflow-x-auto p-0 m-0">
                {lines.into_iter().map(|line| {
                    let class = match line.kind {
                        DiffLineKind::Header => "text-muted-foreground bg-muted/30 font-semibold",
                        DiffLineKind::Hunk => "text-info bg-info/10",
                        DiffLineKind::Add => "text-success bg-success/10",
                        DiffLineKind::Remove => "text-destructive bg-destructive/10",
                        DiffLineKind::Context => "text-foreground/70",
                    };
                    view! {
                        <div class={format!("px-3 py-0 leading-5 whitespace-pre {class}")}>
                            {line.content}
                        </div>
                    }
                }).collect_view()}
            </pre>
        </div>
    }
}

// ─── Diff Parsing ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
enum DiffLineKind {
    Header,
    Hunk,
    Add,
    Remove,
    Context,
}

#[derive(Debug, Clone)]
struct DiffLine {
    kind: DiffLineKind,
    content: String,
}

fn parse_diff_lines(diff: &str) -> Vec<DiffLine> {
    diff.lines()
        .map(|line| {
            let kind = if line.starts_with("diff ") || line.starts_with("index ") || line.starts_with("---") || line.starts_with("+++") {
                DiffLineKind::Header
            } else if line.starts_with("@@") {
                DiffLineKind::Hunk
            } else if line.starts_with('+') {
                DiffLineKind::Add
            } else if line.starts_with('-') {
                DiffLineKind::Remove
            } else {
                DiffLineKind::Context
            };
            DiffLine {
                kind,
                content: line.to_string(),
            }
        })
        .collect()
}
