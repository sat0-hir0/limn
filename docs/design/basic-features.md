# Editor Basic Features — Inventory

> An inventory of baseline editor features that must exist before any project-specific "deepen thinking" features are built.
> Priority legend: ★★★ = without this the editor does not function / ★★ = required for practical use / ★ = nice to have

---

## 1. Text Input and Display

| Feature | Priority | Notes |
|---------|----------|-------|
| Character input (ASCII and full-width) | ★★★ | The most fundamental capability |
| Japanese IME composition | ★★★ | Composing text and showing candidate list. Critical for Japanese speakers |
| Newlines and blank lines | ★★★ | |
| Font, line-height, and letter-spacing rendering | ★★★ | Foundation of readability. Monospaced / proportional |
| Word wrap for long lines | ★★ | Wrap at window width |
| Emoji, special characters, and combining characters | ★★ | Directly related to the problem of non-obvious character boundaries |

## 2. Cursor and Selection

| Feature | Priority | Notes |
|---------|----------|-------|
| Cursor movement (← → ↑ ↓) | ★★★ | |
| Word-by-word movement (Ctrl/Opt + ← →) | ★★ | |
| Move to line start / end (Home / End) | ★★ | |
| Move to document start / end | ★★ | |
| Range selection (Shift + movement) | ★★★ | |
| Word selection / line selection (double / triple click) | ★★ | |
| Select all (Ctrl/Cmd + A) | ★★★ | |
| Multi-cursor | ★ | Advanced; can be deferred |

## 3. Editing Operations

| Feature | Priority | Notes |
|---------|----------|-------|
| Delete (Backspace / Delete) | ★★★ | |
| Delete word / delete line | ★★ | |
| Copy, cut, paste | ★★★ | OS clipboard integration |
| Paste as plain text | ★★ | Paste without carrying formatting |
| Undo / Redo | ★★★ | Requires modelling edit operations |
| Drag to move text | ★ | Comfortable to have |

## 4. File and Persistence

| Feature | Priority | Notes |
|---------|----------|-------|
| Open file | ★★★ | Load `.md` files |
| Save (autosave) | ★★★ | This project has no explicit save button |
| New file | ★★★ | |
| Character encoding (UTF-8) | ★★★ | |
| External-change detection and reload | ★★ | When another app edits the same `.md` |
| Dirty-state tracking | ★★ | Trigger for autosave |

## 5. Display and Navigation

| Feature | Priority | Notes |
|---------|----------|-------|
| Scrolling | ★★★ | |
| Auto-scroll to cursor position | ★★★ | Follow the cursor when it moves off screen |
| Zoom in / out (font size change) | ★★ | |
| Line numbers | ★ | Depends on the editor; possibly unnecessary for this project |
| Minimap / overview | ★ | Can be deferred |

## 6. Search and Replace

| Feature | Priority | Notes |
|---------|----------|-------|
| In-document search | ★★ | |
| Replace | ★★ | |
| Cross-file search | ★★ | High importance for knowledge management |
| Case sensitivity and regex | ★ | |

## 7. Application Foundation

| Feature | Priority | Notes |
|---------|----------|-------|
| Window management (open, close, resize) | ★★★ | |
| Keyboard shortcut system | ★★★ | Foundation of the keyboard-first experience |
| Settings / preferences | ★★ | Font, theme, etc. |
| Dark / light mode | ★★ | |
| Multi-file tabs / panes | ★ | Start with a single pane; add later |

---

## Where Basic Features End and Project-Specific Features Begin

The inventory above covers **features present in any editor**. On top of these, this project adds its own features:

- Block structure (linear + folding)
- Block operations via the `/` command
- Instant Markdown rendering as you type
- Completion candidates below the cursor (three-tier orchestrator)
- AI integration (select → instruct)
- Links, backlinks, and graph view
- Focus mode

The ★★★ basic features must be working before any project-specific feature is started.
The `gpui-component` `editor` example already covers many of the ★★★ items in sections 1–5, making it the right starting point.

---

## Minimum Set to Lock Down First (★★★ items)

1. Character input + IME + newlines
2. Cursor movement + range selection + select all
3. Delete + copy/cut/paste + undo/redo
4. Open `.md` + autosave
5. Scrolling + cursor follow
6. Window management + shortcut system

Once these six work, the editor qualifies as a text editor.
All project-specific features live in the layer above.
