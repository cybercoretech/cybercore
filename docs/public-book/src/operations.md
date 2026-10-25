# Updater operations

The compatibility command is dry-run by default:

```text
cybercore-update
cybercore-update --apply
cybercore-update --tui
```

The Rust surfaces provide planning, inspection, apply, repair, service restart,
rollback, theme health, GitHub read-only status, repository diff review, and
bounded operation history.

## Safety rules

- Dirty worktrees are blocked unless generated outputs are explicitly
  allowlisted.
- Apply, repair, restart, and rollback require explicit confirmation.
- Binary replacement creates a backup before installation.
- User services use `systemctl --user`; system escalation is never silent.
- The updater does not commit or push repository changes automatically.
