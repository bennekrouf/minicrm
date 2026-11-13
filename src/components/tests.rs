//! The screens rendered to HTML, as desktop-kit tests its header: what a
//! page shows for a given cockpit and place in the window.

use chrono::NaiveDate;
use dioxus::prelude::*;

use super::*;
use crate::model::Cockpit;
use crate::seed;

#[derive(Props, Clone, PartialEq)]
struct At {
    view: View,
    selected: Option<u64>,
    filter: Option<&'static str>,
    empty: bool,
}

fn at(view: View) -> At {
    At {
        view,
        selected: None,
        filter: None,
        empty: false,
    }
}

fn examples() -> Cockpit {
    seed::examples(NaiveDate::from_ymd_opt(2026, 11, 11).unwrap())
}

#[allow(non_snake_case)]
fn Harness(at: At) -> Element {
    use_context_provider(|| {
        Signal::new(if at.empty {
            Cockpit::default()
        } else {
            examples()
        })
    });
    use_context_provider(|| Ui {
        view: Signal::new(at.view),
        selected: Signal::new(at.selected),
        search: Signal::new(String::new()),
        filter: Signal::new(at.filter),
        sort: Signal::new(None),
    });
    rsx! { Workspace {} }
}

fn render(at: At) -> String {
    let mut dom = VirtualDom::new_with_props(Harness, at);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn the_guide_explains_the_cockpit_and_counts_the_priorities() {
    let html = render(at(View::Guide));
    assert!(html.contains("My founder cockpit"));
    assert!(html.contains("It just needs to help me move forward"));
    for step in [
        "Set my 3 priorities for the week",
        "Log my field actions",
        "Capture the weak signals I notice",
        "Clear up my open questions",
    ] {
        assert!(html.contains(step), "{step}");
    }
    // One example priority is done, one under way, none blocked.
    assert!(
        html.contains(r#"<span class="recap-count">1</span><span class="recap-label">Done</span>"#)
    );
    assert!(html
        .contains(r#"<span class="recap-count">0</span><span class="recap-label">Blocked</span>"#));
}

#[test]
fn the_sidebar_lists_the_four_tables_with_their_line_counts() {
    let html = render(at(View::Guide));
    for sheet in Sheet::ALL {
        assert!(html.contains(sheet.title()), "{sheet:?}");
    }
    assert!(html.contains(r#"<span class="sidebar-count">3</span>"#));
    assert!(html.contains(r#"<span class="sidebar-count">4</span>"#));
}

#[test]
fn a_table_shows_its_lines_with_their_status_colours() {
    let html = render(at(View::Sheet(Sheet::Goals)));
    assert!(html.contains("Find 10 freelancers to interview"));
    assert!(html.contains("Test a first offer promise"));
    assert!(html.contains(r#"<span class="pill pill-neutral">To do</span>"#));
    assert!(html.contains(r#"<span class="pill pill-soft">In progress</span>"#));
    assert!(html.contains(r#"<span class="pill pill-good">Done</span>"#));
    // Nothing is selected, so there is no editor and every column shows.
    assert!(!html.contains("detail-col"));
    assert!(!html.contains("grid-compact"));
}

#[test]
fn the_filter_leaves_only_its_lines() {
    let mut view = at(View::Sheet(Sheet::Goals));
    view.filter = Some("Done");
    let html = render(view);
    assert!(html.contains("Prepare 5 interview questions"));
    assert!(!html.contains("Find 10 freelancers"));
    assert!(html.contains("1 of 3"));
}

#[test]
fn a_selected_line_opens_in_the_editor() {
    let id = examples().signals[0].id;
    let mut view = at(View::Sheet(Sheet::Signals));
    view.selected = Some(id);
    let html = render(view);
    assert!(html.contains("detail-col"));
    assert!(html.contains("grid-compact"));
    assert!(html.contains("row row-on"));
    // The importance it has is the one marked in the editor.
    assert!(html.contains(r#"class="choice choice-on choice-warn""#));
    assert!(html.contains("Duplicate"));
    assert!(html.contains("Delete"));
}

#[test]
fn a_line_of_another_table_is_not_opened() {
    let id = examples().signals[0].id;
    let mut view = at(View::Sheet(Sheet::Goals));
    view.selected = Some(id);
    assert!(!render(view).contains("detail-col"));
}

#[test]
fn an_empty_table_says_how_to_start() {
    let mut view = at(View::Sheet(Sheet::Questions));
    view.empty = true;
    let html = render(view);
    assert!(html.contains("This table is empty"));
    assert!(html.contains("+ Question"));
    assert!(!html.contains("<table"));
}
