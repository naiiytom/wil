# Issue Tracker: Local Markdown

Issues, maps, and tickets for this repo live as markdown files under `.scratch/`.

## Conventions

- One feature or effort per directory: `.scratch/<effort>/`
- The Wayfinder map is `.scratch/<effort>/map.md`
- Tickets are one file per issue at `.scratch/<effort>/issues/<NN>-<slug>.md`, numbered from `01`
- Type is tracked via `Type: <research|prototype|grilling|task>`
- Status is tracked via `Status: <open|claimed|resolved>`
- Blocking relationships are recorded as `Blocked by: NN, NN` or `Blocked by: none`
- Discussions, comments, and answers append under `## Comments` or `## Answer`

## Wayfinding Operations

- **Map**: `.scratch/<effort>/map.md` (Destination, Notes, Decisions so far, Not yet specified, Out of scope)
- **Frontier**: open, unblocked, unclaimed tickets in `.scratch/<effort>/issues/` sorted by number
- **Claim**: set `Status: claimed`
- **Resolve**: post answer under `## Answer`, set `Status: resolved`, and append context pointer in `map.md`
