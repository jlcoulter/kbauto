# kbauto

Knowledge Base Playbook Automation — generate, rebase, and diff client-specific knowledge base playbooks from versioned Docusaurus templates.

## Overview

kbauto automates the creation and maintenance of client knowledge bases. It takes versioned playbook templates, substitutes client-specific values from a static details file, optionally rewrites sections using AI (via Ollama) guided by a discovery document, tracks the provenance of every paragraph, and supports rebasing onto new template versions while preserving customisations.

**Key features:**

- **Generate** client KBs from templates with placeholder substitution
- **AI rewriting** via Ollama — sections are rewritten for the client's voice using discovery document context (optional; works without Ollama in substitution-only mode)
- **Provenance tracking** at paragraph level (template / substituted / rewritten)
- **TUI for missing values** — when required placeholders are missing from the details file, an interactive form collects them (text mode); in JSON mode, missing values are reported with exit code 6
- **Rebase** client KBs onto new template versions, preserving customised text
- **Incremental update** when only the details or discovery document changes (unchanged pages stay byte-identical)
- **Diff reports** between playbook versions
- **Config file** at `~/.config/kbauto/config.toml` (XDG convention) for persistent settings — CLI flags override config, config overrides hardcoded defaults, zero-config works out of the box
- **Single binary** — no Node.js, Python, or external runtime required

## Workspace Structure

```
kbauto/
├── crates/
│   ├── kbauto-placeholder/   # Placeholder extraction, resolution, defaults
│   ├── kbauto-provenance/    # Frontmatter, paragraph splitting, provenance merge
│   ├── kbauto-template/      # Template loading, details/discovery parsing, generation pipeline
│   ├── kbauto-customise/     # AI rewriter trait, Ollama implementation, retry/fallback, prompts
│   ├── kbauto-rebase/        # Diff engine, conflict detection, rebase pipeline
│   ├── kbauto-config/        # TOML config file loading, XDG path resolution, CLI override merging
│   └── kbauto-cli/           # CLI entry point (single binary)
└── Cargo.toml                # Workspace root
```

## Template Directory Convention

Templates follow a directory convention:

```
my-template/
├── defaults.json              # Version and default placeholder values
└── docs/
    ├── welcome.md             # Pages with placeholder markers
    ├── services.md
    └── onboarding.md
```

**defaults.json:**

```json
{
  "version": "1.0.0",
  "defaults": [
    {"key": "TEAM_NAME", "value": "Default Team", "type": "text"},
    {"key": "INDUSTRY", "value": "Technology", "type": "text"}
  ]
}
```

**Pages** use three placeholder formats (all resolve to the same canonical key):

- `{{PLACEHOLDER}}` — Mustache format
- `<<placeholder>>` — Angle bracket format
- `PLACEHOLDER` — Bare format (uppercase, 3+ chars, underscores allowed)

```markdown
# Welcome

Welcome {{TEAM_NAME}} to our organisation.

We serve the {{INDUSTRY}} industry.
```

## Two-Document Input Model

kbauto uses two separate input documents:

### Static Details (`--details`)

A Markdown file with `##` headings as keys and body text as values for placeholder substitution (steps 1-5):

```markdown
## FIRM_NAME
Example Corp

## TEAM_MEMBERS
- Jane Smith (Director)
- John Doe (Manager)
```

Headings are canonicalised to uppercase keys (spaces and hyphens become underscores). `## Firm Name` resolves to `FIRM_NAME`.

### Discovery Document (`--discovery`)

A Markdown file with `##` headings as questions and body text as answers, providing context for AI rewriting (steps 6-10):

```markdown
## What is the client's primary service focus?
The client specialises in cloud infrastructure for small businesses.

## What tone should the knowledge base convey?
Professional yet approachable.
```

When `--discovery` is omitted, only substitution is performed (no AI rewriting). When both `--details` and `--discovery` are omitted, a default-only preview is generated.

## Configuration File

kbauto reads settings from `~/.config/kbauto/config.toml` (XDG convention). The file is optional — zero-config works with hardcoded defaults.

```toml
ollama_url = "http://localhost:11434"
ollama_model = "deepseek-v4-flash:cloud"
retry_count = 3
output_format = "text"
```

All fields are optional — a partial config file is valid. Missing keys fall back to defaults.

**Precedence**: CLI flags > config file > hardcoded defaults.

Malformed config files produce an error and exit code 1.

## CLI Usage

### generate

Generate a client KB from a template, static details, and optional discovery document:

```bash
kbauto generate \
  --template-dir ./my-template \
  --details ./details.md \
  --discovery ./discovery.md \
  --output ./client-output
```

Options:
- `--template-dir` (`-t`) — Path to the template directory (required)
- `--details` (`-d`) — Path to the client static details Markdown file (optional)
- `--discovery` — Path to the client discovery document Markdown file (optional)
- `--output` (`-o`) — Output directory (default: `./output`)
- `--output-format` — Output format: `text` (default) or `json`
- `--ollama-url` — Ollama server URL (overrides config)
- `--ollama-model` — Ollama model name (overrides config)
- `--retry-count` — Number of retry attempts for AI rewriting (overrides config)

When required placeholders are missing from the details file and defaults:
- In **text mode**: an interactive TUI launches to collect the missing values
- In **JSON mode**: missing values are reported as JSON with exit code 6

Output files are written to the output directory with Docusaurus frontmatter containing provenance markers.

### schema

Display placeholder schema for a template directory:

```bash
kbauto schema --template-dir ./my-template
```

Options:
- `--template-dir` (`-t`) — Path to the template directory (required)
- `--output-format` — Output format: `text` (default) or `json`

### diff

Show changes between two playbook versions:

```bash
kbauto diff \
  --old-version-dir ./my-template-v1 \
  --new-version-dir ./my-template-v2
```

Reports added, removed, and modified pages with paragraph-level detail.

Options:
- `--old-version-dir` — Path to the old version template directory (required)
- `--new-version-dir` — Path to the new version template directory (required)
- `--output-format` — Output format: `text` (default) or `json`

### rebase

Rebase a client KB onto a new playbook version:

```bash
kbauto rebase \
  --client-kb-dir ./client-output \
  --old-version 1.0.0 \
  --new-version 2.0.0 \
  --template-dir ./my-template-v2
```

Updates template-origin paragraphs to the new base text while preserving substituted and rewritten text. Conflicts (where both base and client text changed) are flagged.

Options:
- `--client-kb-dir` — Path to the client KB directory (required)
- `--old-version` — Old playbook version, e.g. "1.0.0" (required)
- `--new-version` — New playbook version, e.g. "2.0.0" (required)
- `--template-dir` (`-t`) — Path to the new template directory (required)
- `--output-format` — Output format: `text` (default) or `json`
- `--ollama-url` — Ollama server URL (overrides config)
- `--ollama-model` — Ollama model name (overrides config)
- `--retry-count` — Number of retry attempts for AI rewriting (overrides config)

### update

Incrementally update a client KB when only the details or discovery document has changed:

```bash
kbauto update \
  --template-dir ./my-template \
  --old-details ./details-old.md \
  --new-details ./details-new.md \
  --old-discovery ./discovery-old.md \
  --new-discovery ./discovery-new.md \
  --output ./client-output
```

Only pages containing changed placeholder values are regenerated. Unchanged pages remain byte-identical.

Options:
- `--template-dir` (`-t`) — Path to the template directory (required)
- `--old-details` — Path to the previous details Markdown file (required)
- `--new-details` — Path to the new details Markdown file (required)
- `--old-discovery` — Path to the previous discovery document (optional)
- `--new-discovery` — Path to the new discovery document (optional)
- `--output` (`-o`) — Output directory (default: `./output`)
- `--output-format` — Output format: `text` (default) or `json`
- `--ollama-url` — Ollama server URL (overrides config)
- `--ollama-model` — Ollama model name (overrides config)
- `--retry-count` — Number of retry attempts for AI rewriting (overrides config)

## Provenance

Every paragraph in a generated KB is tagged with a provenance classification:

| Classification | Meaning |
|---|---|
| `template` | Unmodified text from the base playbook |
| `substituted` | Text with placeholders resolved from the static details |
| `rewritten` | Text rewritten by AI customisation |

Provenance is stored in Docusaurus frontmatter as a paragraph-index-to-classification mapping.

## Build

```bash
cargo build --release
```

On NixOS, OpenSSL environment variables are required. If they are not already exported in your shell profile:

```bash
export OPENSSL_DIR=/nix/store/...-openssl-3.x-dev
export OPENSSL_LIB_DIR=/nix/store/...-openssl-3.x/lib
export PKG_CONFIG_PATH=/nix/store/...-openssl-3.x-dev/lib/pkgconfig
cargo build --release
```

## Test

```bash
cargo test --all              # Run all 250 tests
cargo fmt --all -- --check    # Check formatting
```

## Architecture

The project follows a **library-first** architecture. Every feature is implemented as a standalone library crate before the CLI wires it together:

| Crate | Responsibility |
|---|---|
| `kbauto-placeholder` | Extract placeholders from markdown, resolve with details values, parse defaults, semver versioning |
| `kbauto-provenance` | Parse/write Docusaurus frontmatter, split paragraphs, compute anchors, merge provenance |
| `kbauto-template` | Load template directories, parse static details & discovery documents, generate playbooks, incremental updates |
| `kbauto-customise` | AI rewriter trait (Ollama), retry with fallback, customisation prompts |
| `kbauto-rebase` | Diff pages/playbooks, detect conflicts, resolve conflicts, rebase pipeline |
| `kbauto-config` | Load TOML config from XDG path, merge CLI overrides, validate config values |
| `kbauto-cli` | Clap-based CLI binary wiring all libraries together |

## License

MIT