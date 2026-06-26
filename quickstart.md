# kbauto Quickstart Guide

This guide walks you through setting up kbauto from scratch — installing Ollama, creating a template, preparing client input documents, configuring settings, and generating your first client knowledge base.

## Prerequisites

- Rust toolchain (1.95+, 2024 edition)
- [Ollama](https://ollama.com) installed and running (optional — kbauto works without it in substitution-only mode)
- A terminal

## Step 1: Build kbauto

```bash
cd kbauto
cargo build --release
```

The binary is at `./target/release/kbauto`. Verify it works:

```bash
./target/release/kbauto --version
# kbauto 0.1.0

./target/release/kbauto --help
# Generate, rebase, and diff client KB playbooks
```

On NixOS, you need OpenSSL environment variables. If they are not already in your shell profile:

```bash
export OPENSSL_DIR=/nix/store/...-openssl-3.x-dev
export OPENSSL_LIB_DIR=/nix/store/...-openssl-3.x/lib
export PKG_CONFIG_PATH=/nix/store/...-openssl-3.x-dev/lib/pkgconfig
cargo build --release
```

## Step 2: Set Up Ollama (Optional)

Ollama provides AI rewriting — it takes the substituted template text and rewrites it for the client's voice and tone using a discovery document. Without Ollama, kbauto produces substitution-only output (still fully usable, just no AI customisation).

Install Ollama from [ollama.com](https://ollama.com), then start it:

```bash
# Start the Ollama server (runs on localhost:11434 by default)
ollama serve
```

In another terminal, pull a model:

```bash
# Pull a model (this is the default kbauto uses)
ollama pull deepseek-v4-flash:cloud

# Or use a different model
ollama pull llama3
```

Verify Ollama is running:

```bash
curl http://localhost:11434/api/tags
# Should return JSON listing available models
```

If you skip this step, kbauto will work in substitution-only mode (no AI rewriting).

## Step 3: Create a Configuration File (Optional)

kbauto works zero-config with hardcoded defaults. To persist your settings, create a config file:

```bash
mkdir -p ~/.config/kbauto
cat > ~/.config/kbauto/config.toml << 'EOF'
ollama_url = "http://localhost:11434"
ollama_model = "deepseek-v4-flash:cloud"
retry_count = 3
output_format = "text"
EOF
```

All fields are optional. A partial config file is valid:

```bash
# Minimal config — just set the model, everything else uses defaults
echo 'ollama_model = "llama3"' > ~/.config/kbauto/config.toml
```

Precedence: CLI flags > config file > hardcoded defaults.

To verify your config is valid, run any command — if the config is malformed, kbauto will report an error and exit:

```bash
./target/release/kbauto schema --template-dir /any/template
# If config is malformed: "Error: invalid config file at ..." exit code 1
```

## Step 4: Create a Playbook Template

A playbook template is a directory with a `defaults.json` file and a `docs/` folder of markdown pages containing placeholders.

### Directory Structure

```
my-playbook/
├── defaults.json
└── docs/
    ├── welcome.md
    ├── services.md
    └── onboarding.md
```

### defaults.json

This file defines the playbook version and default values for every placeholder:

```json
{
  "version": "1.0.0",
  "defaults": [
    {
      "key": "FIRM_NAME",
      "value": "Your Firm Name",
      "type": "text",
      "description": "The name of the accounting firm"
    },
    {
      "key": "FIRM_TAGLINE",
      "value": "Trusted accounting you can count on",
      "type": "text",
      "description": "Short tagline for the firm"
    },
    {
      "key": "CONTACT_EMAIL",
      "value": "hello@example.com",
      "type": "text",
      "description": "Primary contact email"
    },
    {
      "key": "PHONE",
      "value": "(555) 000-0000",
      "type": "text",
      "description": "Primary phone number"
    },
    {
      "key": "CLIENT_INDUSTRY",
      "value": "small business",
      "type": "text",
      "description": "The client industry the firm specialises in"
    }
  ]
}
```

### Placeholder Formats

Templates can use three placeholder formats. All three resolve to the same canonical key (uppercase, no delimiters):

| Format | Example | Resolves to |
|--------|---------|-------------|
| Mustache | `{{FIRM_NAME}}` | `FIRM_NAME` |
| Angle bracket | `<<firm_name>>` | `FIRM_NAME` |
| Bare | `FIRM_NAME` | `FIRM_NAME` |

You can mix formats in the same template. Mustache is the most common and readable.

### Template Pages

Each page is a markdown file with Docusaurus frontmatter and placeholder markers:

`docs/welcome.md`:

```markdown
---
slug: welcome
title: Welcome
---

# Welcome to {{FIRM_NAME}}

At **{{FIRM_NAME}}**, our mission is simple: {{FIRM_TAGLINE}}. We have built our practice around serving the {{CLIENT_INDUSTRY}} community with dedication and expertise.

## Getting Help

If you have questions at any point, reach out:

- Email: {{CONTACT_EMAIL}}
- Phone: {{PHONE}}
```

`docs/services.md`:

```markdown
---
slug: services
title: Our Services
---

# Our Services

{{FIRM_NAME}} provides a full range of services tailored to {{CLIENT_INDUSTRY}} businesses.

Contact us at {{CONTACT_EMAIL}} or call {{PHONE}}.
```

### Verify Your Template

Extract the placeholder schema to see what your template requires:

```bash
./target/release/kbauto schema --template-dir my-playbook
```

Output:

```
Placeholder Schema for my-playbook (v1.0.0)
==================================================

  FIRM_NAME        text    Default: "Your Firm Name"
  FIRM_TAGLINE     text    Default: "Trusted accounting you can count on"
  CLIENT_INDUSTRY  text    Default: "small business"
  CONTACT_EMAIL    text    Default: "hello@example.com"
  PHONE            text    Default: "(555) 000-0000"

Total placeholders: 5
```

JSON format for scripting:

```bash
./target/release/kbauto schema --template-dir my-playbook --output-format json
```

## Step 5: Create Client Input Documents

kbauto uses two input documents: a **static details** file for placeholder substitution, and a **discovery document** for AI rewriting context. You can use one or both.

### Static Details File (`--details`)

A markdown file with `##` headings as keys and body text as values. Headings are canonicalised to uppercase keys (spaces and hyphens become underscores).

`clients/acme/details.md`:

```markdown
## Firm Name
Acme Accounting

## Firm Tagline
Your finances, our passion

## Client Industry
restaurant and hospitality

## Contact Email
hello@acmeaccounting.com

## Phone
(555) 234-5678
```

`## Firm Name` resolves to `FIRM_NAME`, `## Contact Email` resolves to `CONTACT_EMAIL`, etc. The heading text is matched case-insensitively and canonicalised.

### Discovery Document (`--discovery`)

A markdown file with `##` headings as questions and body text as answers. This provides context for AI rewriting — it tells the model about the client's voice, tone, and positioning.

`clients/acme/discovery.md`:

```markdown
## What is the client's primary service focus?
The client specialises in tax advisory for small businesses, with a strong emphasis on personalized service and year-round support.

## What tone should the knowledge base convey?
Professional yet approachable. Avoid jargon where possible. The firm prides itself on being accessible to non-financial people.

## What sets this firm apart from competitors?
They offer a fixed-fee model with no surprises, and they proactively reach out to clients throughout the year, not just at tax time.
```

Empty answers are valid — they mean "no context provided for this question, skip AI rewriting for relevant paragraphs."

## Step 6: Generate a Client KB

### Substitution-Only (No AI)

When you provide only `--details` (no `--discovery`), placeholders are filled from the details file and defaults. No AI rewriting occurs:

```bash
./target/release/kbauto generate \
  --template-dir my-playbook \
  --details clients/acme/details.md \
  --output clients/acme/output
```

Output:

```
Generated 2 pages, 5 placeholders resolved in 0.0s
```

Check the output:

```bash
cat clients/acme/output/welcome.md
```

```markdown
---
slug: welcome
title: Welcome
playbook_version: 1.0.0
provenance:
  0: substituted
  1: substituted
  2: substituted
---

# Welcome to Acme Accounting

At **Acme Accounting**, our mission is simple: Your finances, our passion. We have built our practice around serving the restaurant and hospitality community with dedication and expertise.

## Getting Help

If you have questions at any point, reach out:

- Email: hello@acmeaccounting.com
- Phone: (555) 234-5678
```

Each paragraph is tagged with provenance: `substituted` means placeholders were filled from the details file.

### Full AI Rewriting

When you provide both `--details` and `--discovery` (and Ollama is running), each paragraph is rewritten for the client's voice after substitution:

```bash
./target/release/kbauto generate \
  --template-dir my-playbook \
  --details clients/acme/details.md \
  --discovery clients/acme/discovery.md \
  --ollama-url http://localhost:11434 \
  --ollama-model deepseek-v4-flash:cloud \
  --output clients/acme/output-ai
```

Rewritten paragraphs get provenance `rewritten` instead of `substituted`.

If `--ollama-url` is provided but Ollama is unreachable, kbauto exits with code 5.

If `--ollama-url` is not provided and Ollama is not running, kbauto proceeds substitution-only silently (no error).

### Default-Only Preview

When neither `--details` nor `--discovery` is provided, a preview is generated using only the defaults from `defaults.json`:

```bash
./target/release/kbauto generate \
  --template-dir my-playbook \
  --output clients/preview
```

All placeholders resolve to default values. All provenance is `template`.

### JSON Output

For scripting, use `--output-format json`:

```bash
./target/release/kbauto generate \
  --template-dir my-playbook \
  --details clients/acme/details.md \
  --output clients/acme/output \
  --output-format json
```

```json
{"pages_generated": 2, "placeholders_resolved": 5, "elapsed_seconds": 0.000744}
```

### Missing Values and the TUI

If required placeholders have no value in the details file or defaults:

- **Text mode**: An interactive TUI launches showing each missing key with its description and a text input field. Fill in the values and press Enter to continue generation. Press Esc or q to cancel (exit code 1).

- **JSON mode**: Missing values are reported as JSON with exit code 6:

```json
{
  "error": "missing_values",
  "missing_values": [
    {
      "key": "WEBSITE",
      "description": "Placeholder {WEBSITE} was not resolved",
      "default": null
    }
  ]
}
```

## Step 7: Rebase a Client KB

When your playbook template updates to a new version, rebase existing client KBs onto the new version. Template-origin text is updated; substituted and rewritten text is preserved.

First, create a v2 template (bump version in `defaults.json`, modify pages, add new pages):

```
my-playbook-v2/
├── defaults.json    # version: "2.0.0", adds WEBSITE placeholder
└── docs/
    ├── welcome.md   # modified — new paragraph
    ├── services.md  # modified — new section
    └── onboarding.md # new page
```

Then rebase:

```bash
./target/release/kbauto rebase \
  --client-kb-dir clients/acme/output \
  --old-version 1.0.0 \
  --new-version 2.0.0 \
  --template-dir my-playbook-v2
```

Output (JSON):

```json
{
  "pages_updated": 2,
  "conflicts": 0,
  "output_dir": "clients/acme/output"
}
```

- Template-origin paragraphs are updated to the v2 text
- Substituted/rewritten paragraphs are preserved
- Conflicts (where both base and client text changed) are flagged for review
- New pages from v2 are added with defaults

## Step 8: Incremental Update

When only the client's details or discovery document changes (template stays the same), use incremental update. Only pages containing changed placeholders are regenerated; unaffected pages stay byte-identical.

`clients/acme/details-v2.md` (firm name changed):

```markdown
## Firm Name
Acme Accounting & Advisory

## Firm Tagline
Your finances, our passion

## Client Industry
restaurant and hospitality

## Contact Email
hello@acmeaccounting.com

## Phone
(555) 234-5678
```

Run the update:

```bash
./target/release/kbauto update \
  --template-dir my-playbook \
  --old-details clients/acme/details.md \
  --new-details clients/acme/details-v2.md \
  --output clients/acme/output
```

Output (JSON):

```json
{
  "pages_updated": 2,
  "pages_unchanged": 0,
  "placeholders_updated": 1,
  "changed_keys": ["FIRM_NAME"],
  "added_keys": [],
  "removed_keys": []
}
```

Only `FIRM_NAME` changed, so only pages containing `{{FIRM_NAME}}` are regenerated. Pages without that placeholder remain byte-identical.

You can also pass `--old-discovery` and `--new-discovery` to update based on discovery document changes.

## Step 9: Diff Between Template Versions

See what changed between two playbook versions:

```bash
./target/release/kbauto diff \
  --old-version-dir my-playbook \
  --new-version-dir my-playbook-v2
```

Output (JSON):

```json
{
  "old_version": "1.0.0",
  "new_version": "2.0.0",
  "changes": [
    {"added": {"filename": "onboarding.md"}},
    {"modified": {"filename": "welcome.md", "paragraph_changes": [...]}}
  ]
}
```

## Step 10: Guided Wizard (Two-Phase Flow)

The simplest way to use kbauto is the guided wizard. Running `kbauto` with no subcommand launches an interactive wizard that walks you through the entire client lifecycle in two phases.

### Phase 1: Scaffold a New Client

```bash
./target/release/kbauto
# Or explicitly point at a new directory:
./target/release/kbauto
# > Enter the path to the client directory: clients/newclient
```

The wizard detects that the directory doesn't have client files yet and enters **scaffold phase**. It prompts for a template directory, then creates:

```text
clients/newclient/
├── details.md         # Skeleton with placeholder keys + default values
├── discovery.md       # Skeleton with canonical discovery questions
├── kb/                # Empty output directory
└── .template-path     # Records the template directory path
```

The wizard exits, telling you to edit `details.md` and `discovery.md` and return when ready. Take your time — the wizard is stateless and will detect the right phase when you return.

### Phase 2: Generate the Knowledge Base

After editing the skeleton files (days, weeks, or longer later):

```bash
./target/release/kbauto
# > Enter the path to the client directory: clients/newclient
```

The wizard detects that skeleton files exist but `kb/` is empty, and enters **generate phase**. It reads `.template-path` to locate the template, loads your edited `details.md` and `discovery.md`, and generates the KB.

If the template directory recorded in `.template-path` has moved or been renamed, the wizard reports the missing path and prompts you for the correct location.

### Existing Client: Rebase or Update

When you run `kbauto` on a directory that already has a generated KB, the wizard offers three options:

1. **Regenerate** — full generation from scratch
2. **Rebase** — update template-origin text to a new playbook version while preserving customised content
3. **Incremental update** — re-process only pages affected by changes to your details/discovery files

### How Phase Detection Works

The wizard is fully stateless — it detects the phase by inspecting directory contents:

| Directory State | Phase |
|---|---|
| No `details.md` or `discovery.md` | Scaffold (phase 1) |
| Skeleton files exist, `kb/` is empty | Generate (phase 2) |
| `kb/` has generated content | Rebase or update |

No session files, no marker files — the `.template-path` file is a configuration reference, not a progress tracker.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (missing files, malformed config, TUI cancelled) |
| 2 | CLI argument error |
| 3 | Version mismatch |
| 5 | Ollama explicitly requested (`--ollama-url` provided) but unreachable |
| 6 | Missing placeholder values (JSON mode — unresolved placeholders reported) |

## Config File Reference

`~/.config/kbauto/config.toml`:

```toml
# Ollama server URL (default: http://localhost:11434)
ollama_url = "http://localhost:11434"

# Ollama model name (default: deepseek-v4-flash:cloud)
ollama_model = "deepseek-v4-flash:cloud"

# Max retry attempts for AI rewriting (default: 3)
retry_count = 3

# Output format: "text" or "json" (default: text)
output_format = "text"
```

All fields optional. Missing keys use hardcoded defaults. Malformed TOML produces an error and exit code 1.

CLI flags override config values for that invocation only. The config file is auto-created with default values on first run if it doesn't exist.