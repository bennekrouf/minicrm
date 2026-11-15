//! The screens rendered to HTML, as desktop-kit tests its header: what a
//! page shows for a given cockpit and place in the window.

use chrono::NaiveDate;
use dioxus::prelude::*;

use desktop_kit::library::{Item, Kind as MediaKind, Status};

use super::*;
use crate::model::Cockpit;
use crate::seed;

#[derive(Props, Clone, PartialEq)]
struct At {
    view: View,
    selected: Option<u64>,
    filter: Option<&'static str>,
    empty: bool,
    /// Sophie's pre-sale offer names a LinkedIn post in the media library.
    library: bool,
}

fn at(view: View) -> At {
    At {
        view,
        selected: None,
        filter: None,
        empty: false,
        library: false,
    }
}

/// The LinkedIn post the pre-sale offer went out in.
fn offer_post() -> Item {
    let mut post = Item::new(
        "publishkit:offer",
        "publishkit",
        MediaKind::Social,
        "Pre-sale: 5 places at half price",
        Status::Posted,
    );
    post.network = Some("linkedin".into());
    post.open = "/site/social/linkedin/offer.md".into();
    post.url = Some("https://www.linkedin.com/feed/update/1".into());
    post.date = "2026-11-08".into();
    post
}

fn presale(cockpit: &Cockpit) -> u64 {
    cockpit
        .actions
        .iter()
        .find(|r| r.get("action") == "Pre-sale offer sent to Sophie")
        .unwrap()
        .id
}

/// A Wednesday: the examples' follow-ups are one late, three due by
/// Sunday.
fn wednesday() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 11).unwrap()
}

fn examples() -> Cockpit {
    seed::examples(wednesday())
}

#[allow(non_snake_case)]
fn Harness(at: At) -> Element {
    use_context_provider(|| {
        let mut cockpit = if at.empty {
            Cockpit::default()
        } else {
            examples()
        };
        if at.library {
            let id = presale(&cockpit);
            cockpit.set(Sheet::Actions, id, "media", "publishkit:offer".into());
        }
        Signal::new(cockpit)
    });
    use_context_provider(|| {
        Signal::new(if at.library {
            vec![offer_post()]
        } else {
            Vec::new()
        })
    });
    use_context_provider(|| Ui {
        view: Signal::new(at.view),
        selected: Signal::new(at.selected),
        search: Signal::new(String::new()),
        filter: Signal::new(at.filter),
        sort: Signal::new(None),
        notice: Signal::new(None),
        today: Signal::new(wednesday()),
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
fn the_sidebar_lists_the_tables_with_their_line_counts() {
    let html = render(at(View::Guide));
    for sheet in Sheet::ALL {
        assert!(html.contains(sheet.title()), "{sheet:?}");
    }
    assert!(html.contains(r#"<span class="sidebar-count">3</span>"#));
    assert!(html.contains(r#"<span class="sidebar-count">4</span>"#));
    assert!(html.contains(r#"<span class="sidebar-count">5</span>"#));
    // Four follow-ups due, one of them late, so the count is red.
    assert!(html.contains("Follow-ups"));
    assert!(html
        .contains(r#"class="sidebar-count sidebar-count-late" title="Some are overdue">4</span>"#));
}

#[test]
fn the_guide_shows_the_pipeline_and_the_follow_ups_due() {
    let html = render(at(View::Guide));
    for stage in [
        "Lead",
        "Contacted",
        "In conversation",
        "Proposal sent",
        "Client",
        "Not now",
    ] {
        assert!(html.contains(stage), "{stage}");
    }
    assert!(html.contains(
        r#"<span class="recap-count">1</span><span class="recap-label">Proposal sent</span>"#
    ));
    assert!(html.contains("4 follow-ups due before Sunday, 1 of them late"));
}

#[test]
fn the_follow_ups_page_lists_the_late_first() {
    let html = render(at(View::FollowUps));
    let late = html.find("Overdue").unwrap();
    let today = html.find("Later this week").unwrap();
    let sophie = html.find("Sophie Durand").unwrap();
    let lea = html.find("Léa Martin").unwrap();
    assert!(late < sophie && sophie < today && today < lea);
    assert!(!html.contains("Nina Rossi"), "due next week");
    assert!(html.contains("Ask whether the pre-sale offer suits her"));
    assert!(html.contains(r#"<span class="pill pill-bad cell-date">10 Nov 2026</span>"#));
    assert!(html.contains("In a week"));
    assert!(html.contains("Done"));
}

#[test]
fn nothing_due_says_so() {
    let mut view = at(View::FollowUps);
    view.empty = true;
    assert!(render(view).contains("Nothing to follow up this week"));
}

#[test]
fn a_contact_editor_lists_what_was_done_with_them() {
    let cockpit = examples();
    let id = cockpit.contact_named("Léa Martin").unwrap().id;
    let mut view = at(View::Sheet(Sheet::Contacts));
    view.selected = Some(id);
    let html = render(view);
    assert!(html.contains("What I did with them"));
    assert!(html.contains("Interview with a freelance graphic designer"));
    assert!(html.contains("+ Log an action with them"));
    // The stage is marked, and the follow-up can be cleared.
    assert!(html.contains(r#"class="choice choice-on choice-soft">In conversation"#));
    assert!(html.contains("Clear"));
}

#[test]
fn an_action_names_its_contact_in_the_grid_and_the_editor() {
    let cockpit = examples();
    let action = cockpit
        .actions
        .iter()
        .find(|r| r.get("action") == "Pre-sale offer sent to Sophie")
        .unwrap()
        .id;
    let sophie = cockpit.contact_named("Sophie Durand").unwrap().id;
    let html = render(at(View::Sheet(Sheet::Actions)));
    assert!(html.contains(r#"<div class="clamp">Sophie Durand</div>"#));
    let mut view = at(View::Sheet(Sheet::Actions));
    view.selected = Some(action);
    let html = render(view);
    assert!(
        html.contains(&format!(r#"value="{sophie}" selected=true"#)),
        "{html}"
    );
    assert!(html.contains("No one in particular"));
    assert!(html.contains("Open \u{203a}"));
}

#[test]
fn a_contact_grid_colours_follow_ups_as_they_come_round() {
    let html = render(at(View::Sheet(Sheet::Contacts)));
    assert!(html.contains("Import CSV"));
    assert!(html.contains("Export CSV"));
    // Sophie is late, Léa due Friday, Nina next week.
    assert!(html.contains(r#"<span class="pill pill-bad cell-date">10 Nov 2026</span>"#));
    assert!(html.contains(r#"<span class="pill pill-warn cell-date">13 Nov 2026</span>"#));
    assert!(html.contains(r#"<span class="pill pill-neutral cell-date">18 Nov 2026</span>"#));
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

#[test]
fn an_action_shows_the_post_it_went_out_in_and_where_it_stands() {
    let cockpit = examples();
    let mut view = at(View::Sheet(Sheet::Actions));
    view.library = true;
    view.selected = Some(presale(&cockpit));
    let html = render(view);
    assert!(html.contains("Video or post"));
    assert!(
        html.contains("Pre-sale: 5 places at half price \u{b7} LinkedIn post, posted"),
        "{html}"
    );
    assert!(html.contains(r#"<span class="pill pill-good">Posted</span>"#));
    assert!(html.contains("Open in PublishKit \u{203a}"));
    assert!(html.contains("See it online \u{203a}"));
}

#[test]
fn a_contact_s_history_names_the_post_an_action_was_about() {
    let cockpit = examples();
    let mut view = at(View::Sheet(Sheet::Contacts));
    view.library = true;
    view.selected = Some(cockpit.contact_named("Sophie Durand").unwrap().id);
    let html = render(view);
    assert!(
        html.contains("LinkedIn post: Pre-sale: 5 places at half price"),
        "{html}"
    );
}

#[test]
fn with_an_empty_library_the_field_says_what_it_is_for() {
    let cockpit = examples();
    let mut view = at(View::Sheet(Sheet::Actions));
    view.selected = Some(presale(&cockpit));
    let html = render(view);
    assert!(html.contains("What PolishCast, TrackCut and PublishKit make shows here"));
}

#[test]
fn a_goal_names_the_video_or_post_it_is_for() {
    let cockpit = examples();
    let mut view = at(View::Sheet(Sheet::Goals));
    view.selected = Some(cockpit.goals[0].id);
    let html = render(view);
    assert!(html.contains("Video or post"), "{html}");
}

#[test]
fn a_contact_reads_name_then_next_step_in_the_grid_and_on_its_card() {
    let html = render(at(View::Sheet(Sheet::Contacts)));
    let heading = |label: &str| html.find(&format!(">{label}<")).unwrap();
    assert!(heading("Name") < heading("Next step"));
    assert!(heading("Next step") < heading("Who they are"));
    // The same lines as cards, for a narrow window, the name first.
    let cards = html.find(r#"class="cards""#).unwrap();
    let card = &html[cards..];
    let sophie = card.find("Sophie Durand").unwrap();
    let next = card
        .find("Ask whether the pre-sale offer suits her")
        .unwrap();
    assert!(sophie < next, "{card}");
    assert!(card.contains(r#"<span class="pill pill-bad cell-date">10 Nov 2026</span>"#));
}
