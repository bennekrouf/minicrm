//! The example lines a new cockpit opens with, so no table starts blank.
//!
//! They follow one made-up project: an entrepreneur launching a coaching
//! offer that helps freelancers organise their sales prospecting. Dated in
//! the week the app is first opened, so they read as this week's work, with
//! one follow-up already late and others due before Sunday.

use chrono::{Datelike, Duration, NaiveDate};

use crate::model::{week_label, Cockpit, Sheet};

/// A cockpit holding the examples, as of `today`.
pub fn examples(today: NaiveDate) -> Cockpit {
    let monday = today - Duration::days(i64::from(today.weekday().num_days_from_monday()));
    let day = |offset: i64| {
        (monday + Duration::days(offset))
            .format("%Y-%m-%d")
            .to_string()
    };
    let week = week_label(today);
    let mut cockpit = Cockpit::default();

    // The people first: the actions below name them.
    let mut contact = std::collections::BTreeMap::new();
    for (name, about, stage, source, link, follow_up, next, notes) in [
        (
            "Léa Martin",
            "Freelance graphic designer, 3 years in business",
            "In conversation",
            "LinkedIn network",
            "linkedin.com/in/lea-martin-design",
            Some(4),
            "Ask her for 2 contacts to interview",
            "Prospects in bursts, only when she runs short of clients",
        ),
        (
            "Karim Benali",
            "Freelance web developer",
            "Contacted",
            "Commented on my LinkedIn post",
            "linkedin.com/in/karim-benali",
            Some(3),
            "Thank him for his comment and offer a 20-minute call",
            "",
        ),
        (
            "Sophie Durand",
            "Freelance copywriter",
            "Proposal sent",
            "Former colleague",
            "sophie-durand.fr",
            Some(1),
            "Ask whether the pre-sale offer suits her",
            "Interested in follow-up templates above all",
        ),
        (
            "Tom Leroy",
            "Freelance photographer",
            "Lead",
            "Recommended by Léa",
            "",
            None,
            "Send a first, short and personal message",
            "",
        ),
        (
            "Nina Rossi",
            "Freelance UX designer",
            "Client",
            "My newsletter",
            "nina-rossi.design",
            Some(9),
            "Check in after her first week with the method",
            "First paying client of the pilot",
        ),
    ] {
        let follow_up = follow_up.map(day).unwrap_or_default();
        let id = cockpit.add_with(
            Sheet::Contacts,
            &[
                ("name", name),
                ("about", about),
                ("stage", stage),
                ("source", source),
                ("link", link),
                ("follow_up", &follow_up),
                ("next", next),
                ("notes", notes),
            ],
        );
        contact.insert(name, id.to_string());
    }
    let with = |name: &str| contact.get(name).cloned().unwrap_or_default();

    for (priority, goal, action, status, next, comment) in [
        (
            "1",
            "Find 10 freelancers to interview",
            "List profiles on LinkedIn and in my network",
            "In progress",
            "Send the first 5 messages",
            "6 profiles found so far",
        ),
        (
            "2",
            "Prepare 5 interview questions",
            "Write a short interview guide",
            "Done",
            "Try the questions in the first interview",
            "Questions centred on following up and choosing targets",
        ),
        (
            "3",
            "Test a first offer promise",
            "Write 3 wordings and show them to 3 freelancers",
            "To do",
            "Keep the wording people understand best",
            "",
        ),
    ] {
        cockpit.add_with(
            Sheet::Goals,
            &[
                ("week", &week),
                ("priority", priority),
                ("goal", goal),
                ("action", action),
                ("status", status),
                ("next", next),
                ("comment", comment),
            ],
        );
    }

    for (offset, action, kind, who, target, result, learned, follow_up, follow_up_on) in [
        (
            0,
            "Message sent to 5 freelancers",
            "Message sent",
            "",
            "Freelancers in my LinkedIn network",
            "2 replies, 1 interview booked",
            "A short, personal message gets more replies",
            "Follow up on Friday with the 3 who did not reply",
            Some(4),
        ),
        (
            1,
            "Interview with a freelance graphic designer",
            "Interview",
            "Léa Martin",
            "Freelance graphic designer, 3 years in business",
            "45 minutes of conversation, plenty of quotes",
            "She prospects in bursts, only when she runs short of clients",
            "Ask her for 2 contacts to interview",
            None,
        ),
        (
            2,
            "LinkedIn post about prospecting",
            "Post",
            "",
            "Freelancers in my network",
            "12 reactions, 3 comments",
            "The word \u{201c}follow-up\u{201d} gets a reaction",
            "Contact the 3 people who commented",
            None,
        ),
        (
            3,
            "Test of an offer promise sentence",
            "Test",
            "",
            "3 freelancers asked",
            "2 out of 3 understand the offer straight away",
            "\u{201c}Find clients without giving up your evenings\u{201d} lands better than \u{201c}prospecting method\u{201d}",
            "Reword and test again next week",
            Some(7),
        ),
    ] {
        let follow_up_on = follow_up_on.map(day).unwrap_or_default();
        cockpit.add_with(
            Sheet::Actions,
            &[
                ("date", &day(offset)),
                ("action", action),
                ("type", kind),
                ("contact", &with(who)),
                ("target", target),
                ("result", result),
                ("learned", learned),
                ("follow_up", follow_up),
                ("follow_up_on", &follow_up_on),
            ],
        );
    }
    // The offer sent to Sophie, the week before.
    cockpit.add_with(
        Sheet::Actions,
        &[
            ("date", &day(-3)),
            ("action", "Pre-sale offer sent to Sophie"),
            ("type", "Message sent"),
            ("contact", &with("Sophie Durand")),
            ("result", "She asked for a few days to think"),
            ("follow_up", "Ask for her answer"),
        ],
    );

    for (offset, signal, source, meaning, importance, action) in [
        (
            1,
            "Several freelancers say they dislike following up",
            "Interviews and messages",
            "Following up feels awkward to them, rather than a matter of time",
            "High",
            "Offer ready-to-use follow-up templates",
        ),
        (
            1,
            "Many struggle to know who to contact",
            "Interviews",
            "Targeting may be the first problem to solve",
            "High",
            "Add a question about targeting to the interviews",
        ),
        (
            2,
            "Some confuse prospecting with pushy selling",
            "LinkedIn comments",
            "The word \u{201c}prospecting\u{201d} may put people off",
            "Medium",
            "Try other words in the promise",
        ),
        (
            3,
            "Freelancers want simple, quick methods",
            "Interviews, LinkedIn post",
            "A short, concrete offer would appeal more than a long programme",
            "Medium",
            "Sketch a format that takes 30 minutes a week",
        ),
    ] {
        cockpit.add_with(
            Sheet::Signals,
            &[
                ("date", &day(offset)),
                ("signal", signal),
                ("source", source),
                ("meaning", meaning),
                ("importance", importance),
                ("action", action),
            ],
        );
    }

    for (question, why, hypothesis, check, status, answer) in [
        (
            "Do freelancers really want to pay to prospect better?",
            "Without willingness to pay, there is no viable offer",
            "Yes, if they see new clients quickly",
            "Offer a pre-sale to 5 freelancers",
            "To clarify",
            "",
        ),
        (
            "Which target comes first?",
            "To focus my effort on a single profile",
            "Creative freelancers in business for 1 to 3 years",
            "Compare answers by profile after 10 interviews",
            "To clarify",
            "",
        ),
        (
            "Which problem comes up most often?",
            "The offer has to answer the main problem",
            "The difficulty of following up",
            "Count the problems mentioned in the interviews",
            "Being tested",
            "So far, following up and targeting are tied",
        ),
        (
            "Which offer promise is the clearest?",
            "A vague promise does not sell",
            "\u{201c}Find clients without giving up your evenings\u{201d}",
            "Show 3 wordings to 5 freelancers",
            "Being tested",
            "2 freelancers out of 3 understand it straight away",
        ),
    ] {
        cockpit.add_with(
            Sheet::Questions,
            &[
                ("question", question),
                ("why", why),
                ("hypothesis", hypothesis),
                ("check", check),
                ("status", status),
                ("answer", answer),
            ],
        );
    }

    cockpit
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Kind;

    fn wednesday() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 11).unwrap()
    }

    #[test]
    fn every_table_opens_with_its_examples() {
        let cockpit = examples(wednesday());
        assert_eq!(cockpit.goals.len(), 3);
        assert_eq!(cockpit.actions.len(), 5);
        assert_eq!(cockpit.signals.len(), 4);
        assert_eq!(cockpit.questions.len(), 4);
        assert_eq!(cockpit.contacts.len(), 5);
    }

    #[test]
    fn examples_tie_actions_to_contacts_and_leave_follow_ups_due() {
        let cockpit = examples(wednesday());
        let lea = cockpit.contact_named("Léa Martin").unwrap().id;
        let sophie = cockpit.contact_named("Sophie Durand").unwrap().id;
        assert_eq!(cockpit.history(lea).len(), 1);
        assert_eq!(cockpit.history(sophie).len(), 1);
        // On Wednesday: Sophie is a day late, Karim and Léa are due before
        // Sunday, so is the message's follow-up; Nina is next week.
        let due = cockpit.follow_ups(wednesday());
        let titles: Vec<&str> = due.iter().map(|f| f.title.as_str()).collect();
        assert_eq!(
            titles,
            vec![
                "Sophie Durand",
                "Karim Benali",
                "Message sent to 5 freelancers",
                "Léa Martin",
            ]
        );
    }

    #[test]
    fn every_contact_link_points_to_a_contact() {
        let cockpit = examples(wednesday());
        for row in &cockpit.actions {
            let link = row.get("contact");
            assert!(link.is_empty() || cockpit.contact(link).is_some(), "{link}");
        }
    }

    #[test]
    fn examples_are_dated_in_the_current_week() {
        let cockpit = examples(wednesday());
        assert_eq!(cockpit.goals[0].get("week"), "W46");
        assert_eq!(cockpit.actions[0].get("date"), "2026-11-09");
        assert_eq!(cockpit.actions[3].get("date"), "2026-11-12");
    }

    /// A typo in an example's status would show it without its colour and
    /// leave it out of every filter.
    #[test]
    fn every_example_choice_is_in_its_list() {
        let cockpit = examples(wednesday());
        for sheet in Sheet::ALL {
            for column in sheet.columns() {
                let Kind::Choice(options) = column.kind else {
                    continue;
                };
                for row in cockpit.rows(sheet) {
                    let value = row.get(column.key);
                    assert!(
                        options.iter().any(|o| o.label == value),
                        "{sheet:?} {}: {value:?}",
                        column.key
                    );
                }
            }
        }
    }

    #[test]
    fn examples_only_fill_columns_that_exist() {
        let cockpit = examples(wednesday());
        for sheet in Sheet::ALL {
            let keys: Vec<&str> = sheet.columns().iter().map(|c| c.key).collect();
            for row in cockpit.rows(sheet) {
                for key in row.cells.keys() {
                    assert!(keys.contains(&key.as_str()), "{sheet:?}: {key}");
                }
            }
        }
    }
}
