# Changelog

## [0.1.9] - 2026-11-22

### Changed

- Importing contacts now also leaves out people whose email you already
  have, not only those with the same name. Someone who changed their name,
  or who appears as "Bob" in one file and "Robert Smith" in another, is no
  longer added twice.

## [0.1.8] - 2026-11-17

### Changed

- Build and packaging only — no user-visible change.

## [0.1.7] - 2026-11-16

### Added

- MiniCRM is now also published as an .msi for IT departments, to deploy with
  Intune, Group Policy or SCCM: `msiexec /i minicrm.msi ALLUSERS=1 /qn` installs
  it for every user. Double-clicked, it installs for you alone with no admin
  prompt, and it replaces an install made with the setup .exe.
- IT departments can manage MiniCRM on Windows with Group Policy or Intune:
  turn off the update check, self-update or usage statistics, and set
  the licence number for every user, under
  `HKLM\SOFTWARE\Policies\Mayorana\MiniCRM`.
- Your network: bring in a whole LinkedIn export with Import to network,
  beside Import CSV in Contacts, without burying the people you actually
  work with. Those connections stay out of Contacts, the counts and
  Follow-ups, and only show up when you search. Open one and click Add to
  Contacts, or log an action with them, and they become a Lead.

### Fixed

- On Windows, MiniCRM no longer opens a black command window next to its
  own, and closing that window no longer quits the app.
- Installing over an existing MiniCRM on Windows no longer gets stuck on
  "Setup was unable to automatically close all applications". Setup now
  closes MiniCRM and its helper processes itself before replacing it.

## [0.1.6] - 2026-11-16

### Added

- Archive a line instead of deleting it: a contact set to Not now, last
  month's goals, a question cleared up. Archive and Restore sit next to
  Duplicate in the line's editor. An archived line keeps everything, a
  contact what you did with them too, but leaves the table, the counts in
  the sidebar and the guide, and Follow-ups. Each table shows an Archived
  chip beside its filters once something is in it.
- Claude can now archive and restore lines too, several at once, and read a
  long table page after page, newest first if you like. A list of
  contacts no longer stops at the first 200.

## [0.1.5] - 2026-11-15

### Changed

- Logging an action with no follow-up date no longer clears the follow-up
  the contact had due: it stays on Follow-ups until you mark it done or
  change it yourself. An action with a follow-up date still becomes the
  contact's next follow-up.

## [0.1.4] - 2026-11-15

### Added

- MiniCRM fits a smaller window. Once the table no longer has room for its
  columns, every table shows its lines as cards instead, one per line, with
  its name first: a contact's card gives their next step, follow-up date and
  stage. Click a card to open it, as you would a row.
- In a narrow window the editor opens over the table instead of squeezing
  it; click beside it or press Esc to close it. The sidebar shrinks to its
  icons, each page's name showing when you point at it.
- Logging an action with a contact now updates the contact too, so you no
  longer type it twice. A message sent makes a lead Contacted, and a call or
  an interview puts them In conversation; a contact never moves back, and
  Proposal sent, Client and Not now stay yours to set. The action's follow-up
  date and what to do then become the contact's next follow-up and next step,
  and Follow-ups lists it once, under their name. Log an action with no
  follow-up date and the follow-up they had due is marked done.

### Changed

- Contacts reads name first, then the next step, the next follow-up and the
  stage, so what to do next with each person is the first thing you see. The
  next step also stays in the table while a contact is open in the editor,
  and CSV exports list the columns in this order.
- Columns in every table keep room for about twenty characters, so a name
  or a sentence is no longer broken every few letters; a table with more
  columns than the window holds scrolls sideways instead. In Contacts and
  Open questions the first column stays in place as you scroll, as a frozen
  column does in a spreadsheet.

## [0.1.3] - 2026-11-15

### Fixed

- A post planned in the content calendar before you pick its network now reads
  "Social post" in the Video or post list and in the history, instead of
  "Social post post".

## [0.1.2] - 2026-11-15

### Changed

- Build and packaging only — no user-visible change.

## [0.1.1] - 2026-11-14

### Added

- The cockpit: four tables to fill in each week, Weekly goals, Field progress,
  Weak signals and Open questions, each answering one question about how the
  project is going.
- How it works, the page MiniCRM opens on: what the cockpit is for, the
  weekly routine in four steps, and how many priorities are done, under way
  and blocked.
- Example lines in every table on a first launch, from a made-up project, so
  no page starts blank. Change them or delete them.
- An editor beside the table for the line you select. Statuses are one click
  and coloured; everything is saved as you type.
- A filter on each table's status, type or importance, a search across every
  column, and sorting by any column heading.
- Duplicate and delete for a line, with a confirmation before deleting.
- Settings shows the file the cockpit is kept in: copy it to duplicate the
  cockpit.
- Contacts, a table of the people you talk to: who they are, where you found
  them, their profile and email, and the stage they are at, from Lead to
  Client. The guide counts how many are at each stage; a click shows them.
- A contact on each line of Field progress, so every message, call and
  interview is tied to the person it was with. A contact's editor lists what
  you did with them, and logs the next action in one click.
- Follow-up dates on contacts and on actions, and a Follow-ups page with
  everyone to get back to before Sunday, the late ones first. Push one back a
  week or mark it done from there; the sidebar shows how many are due, in red
  when one is late.
- Import CSV and Export CSV on every table. Bring contacts over from a
  spreadsheet, Notion or LinkedIn's connections export; contacts already in
  the cockpit are not added twice. Files with semicolons, as Excel writes them
  in some countries, are read too.
- Use MiniCRM from Claude or Codex: the button in the header shows how to
  connect them. Claude can then read your week, list and update contacts, log
  what you did with someone, set or clear follow-ups, and import or export a
  table as CSV. It asks you before deleting anything, and the window shows its
  changes within moments.
- Posts you publish from PublishKit show up in Field progress on their own, on
  the day they went out, so your weekly review counts your content too. Each
  is logged once; write down what came of it. Posts from before you started
  using this aren't added.
- A "Video or post" field on each action, to say which video or post from
  PolishCast, TrackCut or PublishKit it was about, such as a demo you made for
  a client. It shows where that piece stands and opens it in the app that made
  it, and a contact's history shows it next to the action.
- Claude sees this week's videos and posts too, planned, scheduled or out, and
  can tie an action to one of them.
- A "Video or post" field on each weekly goal too, so a priority like "Publish
  the demo" points at the video planned for it. Once PolishCast records it or
  PublishKit writes the post, the goal follows, and shows whether it is ready
  or out.
