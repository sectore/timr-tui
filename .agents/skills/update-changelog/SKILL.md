# Skill: Update CHANGELOG

Add an entry to `CHANGELOG.md` for unreleased changes.

## Format

Entries go under `## [unreleased]` at the top of the file. Add the section if not already present.

```markdown
## [unreleased]

### Misc

- {entry} [{pr}](https://github.com/sectore/timr-tui/pull/{pr-number})
```

- Use an existing `[unreleased]` section if present — do not create a duplicate.
- Add new entries at the **top** of the list within their section (newest first).
- The PR link is optional if no PR exists yet.
- Match the category (`### Features`, `### Fix`, `### Misc`, etc.) to the nature of the change.
- Look at recent entries in `CHANGELOG.md` for style reference.
