# Agent Anywhere Development Memory

## Repository Conventions

### Naming Conventions
- Use lowercase with hyphens: `aany-hub` (NOT `AAny-Hub` or `aany_hub`)
- All components follow pattern: `aany-{component}`
  - `aany-tmux`: TMux integration
  - `aany-pool`: Agent pool manager
  - `aany-hub`: Centralized web dashboard
- Main CLI is simply: `aany`

### Directory Structure
- Each component has its own directory at root level
- Each component has:
  - `Cargo.toml` (for Rust components)
  - `src/` directory
  - Component-specific files
- Shared documentation in `docs/`
- Python package in `python/`

### Code Structure Patterns
- Rust-based components (aany, aany-tmux, aany-pool)
- Modular design with clear separation
- Each component can work independently
- Unified under main `aany` CLI

### Documentation
- README.md at root for overall project
- Component-specific README.md in each directory
- Technical documentation in component folders
- Architecture docs in `docs/`

## Important Notes
- Respect existing patterns when adding new components
- Follow the established naming conventions
- Maintain compatibility with existing CLI structure

## Git Workflow Rules
- **NEVER use `gh` command** - No GitHub CLI usage
- **NEVER create pull requests** - No PRs to origin
- Use git directly for version control
- Share changes via patches or direct push when authorized
- Keep commits anonymous (aany-agent author)

## Critical Git Safety Rules

**NEVER run `git checkout -- <file>` without explicit user permission!**

- **Incident Date**: 2025-07-27
- **What Happened**: Accidentally ran `git checkout -- aany/src/commands/pool_ui.rs` which reverted all uncommitted changes including:
  - Line number tracking system throughout UI rendering
  - `fill_empty_rows` function for consistent footer positioning
  - WASD navigation support (w/s/a/d keys)
  - Dynamic status icons with colors (▶ for Started, ■ for Stopped, etc.)
  - Centered help text fix
- **Lesson Learned**: ALWAYS check with user before reverting files. Uncommitted work is irretrievable once reverted.
- **Best Practice**: 
  - Use `git diff` to show changes before any destructive operation
  - Ask user: "This will discard all uncommitted changes to [file]. Are you sure?"
  - Consider using `git stash` instead to preserve changes
  - Never assume reverting is safe - it permanently destroys uncommitted work