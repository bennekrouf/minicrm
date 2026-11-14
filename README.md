# MiniCRM

A weekly cockpit for someone starting a business and promoting it on their
own. It answers four questions each week, one table each, and keeps the people
the work is done with: no automation and no connection to other tools, only a
CSV file in and out.

| Page | The question it answers |
| --- | --- |
| Weekly goals | What did I mean to do? |
| Field progress | What did I actually do? |
| Weak signals | What did I notice? |
| Open questions | What do I need to clear up now? |

Beside them:

| Page | What it keeps |
| --- | --- |
| Contacts | Who each person is, where I found them, and their stage, from *Lead* to *Client* |
| Follow-ups | Every contact and action whose follow-up date is this week or past, the late ones first |

**How it works**, the page it opens on, explains the routine in four steps,
counts the priorities that are done, under way and blocked, the contacts at
each stage, and the follow-ups due before Sunday.

![MiniCRM](docs/screenshot.png)

A new cockpit opens with example lines from a made-up project (a coaching offer
that helps freelancers organise their prospecting), dated in the current week.

## Using it

- Pick a page on the left; **+ Goal** (or Action, Signal, Question) adds a
  line and opens it in the editor on the right.
- Statuses and other lists are one click in the editor, and coloured in the
  table: neutral for *To do*, blue for *In progress*, green for *Done*, red for
  *Blocked*.
- Filter a table by its list column with the chips above it, search every
  column with the box, and sort by clicking a column heading.
- A line of Field progress names its contact; a contact's editor lists what
  was done with them and logs the next action.
- **Import CSV** adds lines from a spreadsheet, Notion or LinkedIn's
  connections export, matching the file's headings to the table's columns.
  **Export CSV** saves a table, one column per column, contacts by name.
- Everything is saved as you type, to `cockpit.json` in the app's data folder
  (Settings shows the file). Copying that file duplicates the cockpit.

## From Claude or Codex

`minicrm mcp` serves the cockpit to an MCP client over stdio, with no window.
The header's Claude button shows the config entry and the one-line commands
for Claude Desktop, Claude Code and Codex. Its tools:

| Tool | What it does |
| --- | --- |
| `overview` | This week's priorities, contacts per stage, follow-ups due, and every table's columns |
| `list_lines`, `get_line` | A table's lines, filtered by stage, status or type, contact or text; a contact with what was done with them |
| `add_line`, `update_line`, `delete_line` | Change any table, contacts by name, lists checked against their values |
| `follow_ups`, `set_follow_up` | Who to get back to by Sunday (or within N days); set, push back or clear a follow-up |
| `import_csv`, `export_csv` | A table in from a CSV file (LinkedIn's export included) or out to one |

Each call reads `cockpit.json`, changes it, and writes it back; an open window
notices the change and shows it.

## Building

MiniCRM is a [Dioxus](https://dioxuslabs.com) desktop app and sits beside
`desktop-kit`, which it takes the header, Settings, the update check and the
notice from, like the other mayorana desktop apps.

```bash
cargo run
```

```bash
cargo test
```

The design is GitAgent's: the same palette, type and layout, in
`assets/main.css`.
