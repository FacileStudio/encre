# encre — Agents

The short form of Facile Markdown and the agent operations, sized to paste into an MCP tool
description, the way Notion ships its markdown spec to agents.

## How to use it

An app that exposes encre documents to agents (Docs is the first) copies the block below into
the description of its read and write tools, as is. It is a summary: `spec/FORMAT.md` sections
2 and 4 are the normative rules, and every claim in the block comes from them. When FORMAT.md
changes a rule the block states, change the block in the same commit.

The block assumes the tool returns `read_blocks` output as `[{id, type, markdown}]` and takes an
operation as the JSON object `ops::Op` deserializes.

## The block

`````text
Documents are Facile Markdown: CommonMark with GFM tables, task lists (- [ ] / - [x]) and
strikethrough (~~a~~). Raw HTML is plain text, except <br> as a line break. Headings go to
level 3 (#, ##, ###). No footnotes, math or front matter.

Custom blocks are backtick fences whose info string starts with the block name:

````callout tone="warning" title="Before you deploy"
Markdown inside, any blocks.
````

```stat
{"label": "Revenue", "value": "12 400 €", "delta": "+4%"}
```

```chart
{"kind": "bar", "title": "Signups", "labels": ["Mon", "Tue"], "series": [{"name": "web", "data": [3, 5]}]}
```

- callout: tone is neutral, info, success, warning or danger; title is optional. Its fence is
  one backtick longer than the longest backtick run inside it.
- stat and chart hold one JSON object. chart kind is bar or line; every series has one number
  per label.
- Attribute values use double quotes; \" and \\ are the only escapes. A malformed custom block
  stays a code block.

Read with read (whole document) or read_blocks (top-level blocks with their ids). Write with
one operation per call:

- append {markdown} / prepend {markdown}: add blocks at the end or start
- insert_after {block_id, markdown}: add blocks after one block
- replace_block {block_id, markdown}: replace one block; a heading's section is not touched
- delete_block {block_id}: remove one block
- patch {find, replace, expect=1}: replace exact text of a block's markdown, as read_blocks
  shows it; fails unless exactly `expect` matches exist; a match never spans two blocks
- replace_all {markdown, expect_version}: replace everything, only if the version still matches

Operations address top-level blocks only: a nested id is refused with its parent's id. A
refused operation changes nothing, and its error says what matched and how to narrow the
request. Writes answer the ids of the blocks they inserted. Block ids never appear in markdown.
Prefer insert_after, replace_block and patch over replace_all: they keep other people's cursors.
`````

## Why these rules

- **Top-level blocks only.** An id is stable while people type. Text offsets are not, so an
  agent never addresses inside a block except through `patch`, whose `find` must be exact.
- **`expect` on `patch`.** A find string that matches more than once would edit a place the agent
  did not mean. The operation fails and lists every match with its block id instead of guessing.
- **`expect_version` on `replace_all`.** It is the only operation that rewrites every block, so
  it must prove the agent read the latest version.
- **Errors that teach.** Every `OpError` message names the fix, for example
  `read_blocks lists the block ids` or `add more surrounding text to find`, following
  https://iwe.md/blog/designing-edit-operations-for-ai-agents.

Back to the [documentation index](README.md).
