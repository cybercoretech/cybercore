# Recovery and validation

Before release, validate the Rust workspace, compatibility dry-run, updater
plan, live fleet diagnostics, services, and theme totals.

## Recovery

The updater creates a sibling backup before replacing an installed binary.
Rollback restores the most recent backup for a selected task. Repair retries a
blocked or failed task after its diagnostic condition has been resolved.

## Troubleshooting

1. Run a dry-run and inspect the first blocked diagnostic.
2. Resolve missing sources, dependency, or dirty-worktree conditions.
3. Re-run the plan before applying.
4. Apply only after the plan is clear.
5. Record the result in the operation history.
