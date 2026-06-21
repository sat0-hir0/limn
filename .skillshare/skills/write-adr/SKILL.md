---
name: write-adr
description: Draft a new Architecture Decision Record (ADR) under docs/adr/. Use when the user types /write-adr, asks to "write an ADR", or asks to record a design decision.
---

# write-adr

Collect the background, decision, and considered alternatives for a
design decision, then draft a new ADR under `docs/adr/`.

---

## When to use

- The user types `/write-adr`.
- The user asks to "write an ADR" or "record a design decision".
- A PR or commit contains a design decision worth recording for the
  future.

---

## Procedure

### Step 1. Determine the next sequence number

```sh
ls docs/adr/
```

Find the largest `NNNN-*.md` and use `max + 1` as the next number.
Sequence starts at `0001`, zero-padded to four digits.

### Step 2. Capture recent changes

```sh
git diff HEAD~5..HEAD --stat
git diff HEAD~5..HEAD
```

Use the summary to identify candidate decisions worth recording.

### Step 3. Review recent commit messages and PR titles

```sh
git log --oneline -10
```

Read Conventional Commit types and scopes for design shifts. Do not
overlook decisions hidden inside `feat` / `refactor` / `chore` commits.

### Step 4. Check related existing ADRs

```sh
grep -r "Title\|Status\|Decision" docs/adr/ --include="*.md" -l
```

Read the headers of each ADR and determine whether the new ADR relates
to any existing one. Decide whether `Superseded by` or `see also`
links are needed.

### Step 5. Check the Open Questions in ARCHITECTURE.md

```sh
grep -A 30 "## Open Questions" ARCHITECTURE.md
```

If an Open Question matches, quote it in the ADR's Context. Once the
ADR is Accepted, propose removing the item from Open Questions.

### Step 6. Draft the ADR

Create `docs/adr/NNNN-<slug>.md` based on `docs/adr/template.md`:

- `NNNN` = the sequence number from Step 1.
- `<slug>` = a short kebab-case identifier derived from the title
  (e.g. `adopt-gpui`).

**Always create with `Status: Proposed`.** Wait for user review and
agreement before changing it to `Accepted`.

---

## How to write each section

### Title

`ADR-NNNN: <verb> <object>`.
Example: `Adopt gpui as the GUI Framework`.

### Context

Describe why this decision was needed:

- Project constraints and requirements.
- The event that triggered the decision.
- Background information that was evaluated.

Quote concrete facts from code or diffs. Do not speculate.

### Decision

State what was chosen and why in 1–3 paragraphs. Do not list rejected
alternatives here — they belong in "Considered Alternatives".

### Consequences

Be honest about positives, negatives, and trade-offs. Do not hide the
downsides — future contributors (including your future self) deserve
to know.

### Considered Alternatives

For each rejected alternative:

- A 1–2 line summary.
- A concrete reason for rejection.

The goal is to preserve the fact that alternatives were considered.
Rejected options carry value too.

---

## Review request after drafting

After drafting, ask the user to confirm:

1. Is the **Context** accurate? Any misreading of the background?
2. Does the **Decision** match what was actually decided?
3. Are there trade-offs missing from **Consequences**?
4. Are all the considered options listed in **Considered Alternatives**?
5. May the Status change from `Proposed` to `Accepted`?

---

## After acceptance

Once the ADR is `Accepted`:

1. Add an entry to the index table in `docs/adr/README.md`.
2. Add an entry to the ADR table in `docs/README.md`.
3. Remove the corresponding Open Question from `ARCHITECTURE.md` if
   any exists.
4. Commit with `docs(adr): add ADR-NNNN <title>`.

---

## Never do

- Mark the Status as `Accepted` without user confirmation.
- Write anything that differs from what was actually decided.
- Invent alternatives that were not actually considered.
- Place ADR files anywhere other than `docs/adr/`.
