# Cargo.toml Configuration for SSH Multiplayer Poker Game

## Main Cargo.toml

```toml
[package]
name = "ssh-poker-game"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "SSH-accessible terminal-based multiplayer Texas Hold'em poker game"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/ssh-poker-game"
documentation = "https://docs.rs/ssh-poker-game"
homepage = "https://github.com/yourusername/ssh-poker-game"
keywords = ["poker", "ssh", "multiplayer", "terminal", "game"]
categories = ["games", "network-programming"]
readme = "README.md"
rust-version = "1.75.0"

[dependencies]
# Async runtime
tokio = { version = "1.38", features = ["full"] }
futures = "0.3"

# SSH server implementation
russh = "0.44"
russh-keys = "0.44"

# Terminal UI
ratatui = "0.28"
crossterm = { version = "0.28", features = ["event-stream"] }

# Database integration
tokio-postgres = { version = "0.7", features = ["with-serde_json-1", "with-chrono-0_4"] }
deadpool-postgres = "0.14"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Poker hand evaluation
poker = "0.7"

# Configuration management
config = "0.14"
toml = "0.8"

# Logging and tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
tracing-opentelemetry = "0.25"
opentelemetry = { version = "0.24", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.17", features = ["tokio"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Cryptography and security
argon2 = "0.5"
rand = "0.8"
uuid = { version = "1.10", features = ["v4", "serde"] }

# Time handling
chrono = { version = "0.4", features = ["serde"] }

# CLI argument parsing
clap = { version = "4.5", features = ["derive", "env"] }

# Testing utilities
proptest = { version = "1.4", optional = true }

[dev-dependencies]
# Testing
tokio-test = "0.4"
proptest = "1.4"
criterion = { version = "0.5", features = ["html_reports"] }
tempfile = "3.8"
rstest = "0.22"

# Mocking
mockall = "0.13"

# Test utilities
serial_test = "3.1"
wiremock = "0.6"

[features]
default = ["security-hardening"]
security-hardening = []
development = ["proptest"]
benchmarks = []

[profile.dev]
# Faster builds for development
opt-level = 0
debug = true
split-debuginfo = "unpacked"
debug-assertions = true
overflow-checks = true

[profile.release]
# Optimized for production
opt-level = 3
debug = false
strip = "symbols"
debug-assertions = false
overflow-checks = false
lto = "thin"
codegen-units = 1
panic = "abort"

[profile.test]
# Optimized for testing
opt-level = 1
debug = true

[profile.bench]
# Optimized for benchmarking
opt-level = 3
debug = false
lto = true

[[bin]]
name = "ssh-poker-server"
path = "src/bin/server.rs"

[[bin]]
name = "ssh-poker-client"
path = "src/bin/client.rs"

[[bin]]
name = "ssh-poker-admin"
path = "src/bin/admin.rs"

[[example]]
name = "basic-game"
path = "examples/basic_game.rs"

[[example]]
name = "ai-bot-demo"
path = "examples/ai_bot_demo.rs"

[[bench]]
name = "hand_evaluation"
harness = false

[[bench]]
name = "game_performance"
harness = false

[workspace]
members = [
    "crates/poker-engine",
    "crates/ssh-server",
    "crates/terminal-ui",
    "crates/database",
    "crates/ai-bot",
]

[workspace.dependencies]
# Shared workspace dependencies
tokio = { version = "1.38", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
uuid = { version = "1.10", features = ["v4", "serde"] }
```

## Workspace Crate Configurations

### crates/poker-engine/Cargo.toml
```toml
[package]
name = "poker-engine"
version = "0.1.0"
edition = "2021"
description = "Core poker game engine and logic"

[dependencies]
serde = { workspace = true }
uuid = { workspace = true }
poker = "0.7"
rand = "0.8"
thiserror = { workspace = true }

[dev-dependencies]
proptest = "1.4"
criterion = { version = "0.5", features = ["html_reports"] }
```

### crates/ssh-server/Cargo.toml
```toml
[package]
name = "ssh-server"
version = "0.1.0"
edition = "2021"
description = "SSH server implementation for poker game"

[dependencies]
tokio = { workspace = true }
russh = "0.44"
russh-keys = "0.44"
anyhow = { workspace = true }
tracing = { workspace = true }
argon2 = "0.5"
```

### crates/terminal-ui/Cargo.toml
```toml
[package]
name = "terminal-ui"
version = "0.1.0"
edition = "2021"
description = "Terminal user interface for poker game"

[dependencies]
ratatui = "0.28"
crossterm = { version = "0.28", features = ["event-stream"] }
serde = { workspace = true }
tokio = { workspace = true }
```

### crates/database/Cargo.toml
```toml
[package]
name = "database"
version = "0.1.0"
edition = "2021"
description = "Database integration and models"

[dependencies]
tokio = { workspace = true }
tokio-postgres = { version = "0.7", features = ["with-serde_json-1", "with-chrono-0_4"] }
deadpool-postgres = "0.14"
serde = { workspace = true }
uuid = { workspace = true }
chrono = { version = "0.4", features = ["serde"] }
anyhow = { workspace = true }
```

### crates/ai-bot/Cargo.toml
```toml
[package]
name = "ai-bot"
version = "0.1.0"
edition = "2021"
description = "AI bot implementation for poker players"

[dependencies]
poker-engine = { path = "../poker-engine" }
serde = { workspace = true }
rand = "0.8"
tokio = { workspace = true }
tracing = { workspace = true }
```

## Configuration Files

### .cargo/config.toml
```toml
[build]
rustflags = ["-D", "warnings"]

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]

[alias]
# Development aliases
dev = "run --bin ssh-poker-server"
test-all = "test --workspace --all-features"
check-all = "check --workspace --all-features"
fmt-all = "fmt --all"
clippy-all = "clippy --workspace --all-features --all-targets"

# Release building
build-release = "build --release --all-features"
build-docker = "build --release --target x86_64-unknown-linux-musl"

# Benchmarking
bench-all = "bench --workspace"
bench-hands = "bench --bench hand_evaluation"

# Testing shortcuts
test-unit = "test --lib"
test-integration = "test --test '*'"
test-doc = "test --doc"
```

### rust-toolchain.toml
```toml
[toolchain]
channel = "1.75.0"
components = ["rustfmt", "clippy", "rust-src"]
targets = ["x86_64-unknown-linux-gnu", "x86_64-unknown-linux-musl"]
```

### .rustfmt.toml
```toml
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
indent_style = "Block"
wrap_comments = true
format_code_in_doc_comments = true
normalize_comments = true
normalize_doc_attributes = true
license_template_path = ""
format_strings = false
format_macro_matchers = true
format_macro_bodies = true
empty_item_single_line = true
struct_lit_single_line = true
fn_single_line = false
where_single_line = false
imports_indent = "Block"
imports_layout = "Mixed"
merge_imports = false
reorder_imports = true
reorder_modules = true
reorder_impl_items = false
type_punctuation_density = "Wide"
space_before_colon = false
space_after_colon = true
spaces_around_ranges = false
binop_separator = "Front"
remove_nested_parens = true
combine_control_expr = true
overflow_delimited_expr = false
struct_field_align_threshold = 0
enum_discrim_align_threshold = 0
match_arm_blocks = true
force_multiline_blocks = false
fn_args_layout = "Tall"
brace_style = "SameLineWhere"
control_brace_style = "AlwaysSameLine"
trailing_semicolon = true
trailing_comma = "Vertical"
match_block_trailing_comma = false
blank_lines_upper_bound = 1
blank_lines_lower_bound = 0
edition = "2021"
version = "Two"
inline_attribute_width = 0
merge_derives = true
use_try_shorthand = false
use_field_init_shorthand = false
force_explicit_abi = true
condense_wildcard_suffixes = false
color = "Auto"
required_version = "1.5.1"
unstable_features = false
disable_all_formatting = false
skip_children = false
hide_parse_errors = false
error_on_line_overflow = false
error_on_unformatted = false
report_todo = "Never"
report_fixme = "Never"
ignore = []
emit_mode = "Files"
make_backup = false
```