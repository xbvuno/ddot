# Agent Coding Rules and Guidelines

These rules must be strictly followed by all AI agents contributing to this repository:

- **Git Commit Notation**: Always use Conventional Commits format for commits (e.g., `feat(cli): ...`, `fix(core): ...`, `chore(git): ...`, `docs: ...`). All commits must strictly follow this notation.
- **Clean Output**: CLI tools and commands outputting formatted data (like JSON schema) must write directly and cleanly to `stdout` to support shell piping and redirection (e.g., `> file.json`), with any diagnostic logs/errors written to `stderr`.
