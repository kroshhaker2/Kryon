# Instructions for Agents

Follow `CONTRIBUTING.md`.

- Answer only the question asked. Do not add next steps or related suggestions
  unless requested.
- Start with the main answer or result.
- The first lines must be understandable without reading the rest of the message.
- Do not leave important limitations, errors, or questions until the end.
- Use short paragraphs without long introductions.
- Do not modify files without an explicit command such as "implement", "change",
  "apply", or an equivalent.
- A question such as "how can this be done?" is not a command to modify files.
- Do not provide a complete implementation of a specific task unless explicitly
  requested.
- Do not simplify or omit required data. When integrating systems, pass and model the complete source structure unless the user explicitly requests a reduced subset.
- You may provide alternative code styles, small structures, isolated examples,
  and brief incomplete pseudocode.
- Do not combine examples and pseudocode into a ready-made solution unless
  requested.
- If a request allows multiple options with different behavior, ask one short
  question. Otherwise, do not ask unnecessary questions.
- Before making a change, inspect the existing code relevant to the task.
- Make only the requested change. Do not fix adjacent code.
- Use the most direct approach that matches the project's style.
- Do not add extensibility, configuration, dependencies, interfaces, factories,
  helpers, or handling for hypothetical cases without a current need.
- Do not perform unrelated refactoring or formatting.
- Do not describe a successful build as functional verification.
- To answer one narrow question, use no more than 6 search or read actions. If
  that is insufficient, stop and ask for permission to continue when continuing
  could materially change the result.
- Follow the latest request. Do not continue a previous topic or plan unless it
  is clearly necessary.
- A clarification from the user supersedes conflicting earlier instructions.
- Do not repeat information already given unless requested.
- After making a change, report only:
  - what changed;
  - what was verified;
  - what was not verified.
- Do not publish code or messages on the user's behalf without separate explicit
  permission.
- Suggest a new chat only when stale context is causing repeated confusion or
  conflicting instructions. Do not use it to avoid continuing a difficult task.

## Third-Party Libraries

- First, determine the library version used by the project.
- For APIs and normal behavior, use the official documentation for that version.
- Do not inspect the library's source code if the documentation answers the
  question.
- Inspect the source code only when the behavior is undocumented, the
  documentation conflicts with the observed result, or the user explicitly asks
  you to investigate the implementation.
- Do not study the entire library. Inspect only the specific symbol or execution
  path required for the current question.
