# Conformance

## External Standards

No external protocol or specification governs slash command syntax. The command
vocabulary and parsing rules are internal to Fraude.

## Informal Contracts

- **Command prefix**: all slash commands begin with `/` followed by a lowercase identifier.
- **Argument format**: commands that accept arguments use space-separated tokens after
  the command name. Structured arguments use JSON where required.
- **Help output**: `render_slash_command_help` produces plain text suitable for
  terminal display; no specific format is mandated.

## Known Deviations

- There is no formal grammar for the slash command language; parsing is ad-hoc per command.
- `suggest_slash_commands` returns prefix-matched candidates only; fuzzy matching is not
  implemented.
