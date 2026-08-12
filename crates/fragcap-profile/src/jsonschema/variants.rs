// SPDX-License-Identifier: Apache-2.0

//! Structural validation of a parsed document against the master schema.
//!
//! Hand-rolled over [`serde_json::Value`] rather than executed through a JSON
//! Schema validator crate: the ecosystem value is in publishing the schema
//! document, and taking a validator crate here would add dozens of transitive
//! crates for `$ref` and `format` machinery this schema does not use (see the
//! slice research). The conformance corpus test binds this code to the
//! published document so they cannot drift.
//!
//! Every check accumulates into [`SchemaDiagnostics`]; nothing short-circuits.

use serde_json::{Map, Value};

use super::diagnostic::{SchemaCode, SchemaDiagnostics};

const FIDELITY_TIERS: [&str; 4] = ["authored", "verified", "heuristic-unverified", "observed"];
const KINDS: [&str; 4] = ["profile", "package", "hint", "export"];
const LIFECYCLES: [&str; 3] = ["transient", "session", "service"];
const MODES: [&str; 3] = ["file", "stream", "ring"];
const MATCH_PREDICATES: [&str; 5] = [
    "exe",
    "path_contains",
    "path_regex",
    "cmdline_contains",
    "descends_from",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Strict, // profile, package
    Hint,
    Export,
}

/// Validate a parsed document. Returns every structural violation found.
pub fn check(root: &Value) -> SchemaDiagnostics {
    let mut d = SchemaDiagnostics::new();
    let obj = match root.as_object() {
        Some(o) => o,
        None => {
            d.report(
                SchemaCode::NotAnObject,
                "",
                "the document root must be a JSON object",
            );
            return d.finish();
        }
    };

    check_schema_version(obj, &mut d);
    let kind = check_kind(obj, &mut d);
    check_fidelity(obj, "/fidelity", true, &mut d);
    check_unknown_top_keys(obj, kind, &mut d);

    // Shared components are validated wherever they appear, regardless of kind.
    if let Some(g) = obj.get("game") {
        check_game(g, "/game", strict_game(kind), &mut d);
    }
    if let Some(c) = obj.get("capture") {
        check_capture(c, "/capture", &mut d);
    }
    if let Some(s) = obj.get("stage") {
        check_stage_array(s, "/stage", &mut d);
    }
    if let Some(p) = obj.get("provenance") {
        check_provenance(p, "/provenance", &mut d);
    }
    if let Some(n) = obj.get("notes") {
        if !n.is_string() {
            d.report(SchemaCode::WrongType, "/notes", "`notes` must be a string");
        }
    }

    // Per-variant required-field rules.
    match kind {
        Some(Kind::Strict) => {
            if !obj.contains_key("game") {
                d.report(
                    SchemaCode::MissingField,
                    "/game",
                    "a profile requires `game`",
                );
            }
            match obj.get("stage") {
                None => d.report(
                    SchemaCode::MissingField,
                    "/stage",
                    "a profile requires at least one `stage`",
                ),
                Some(Value::Array(a)) if a.is_empty() => {
                    d.report(SchemaCode::EmptyStages, "/stage", "`stage` is empty");
                }
                _ => {}
            }
        }
        Some(Kind::Hint) => {
            if !obj.contains_key("provenance") {
                d.report(
                    SchemaCode::MissingProvenance,
                    "/provenance",
                    "a hint requires `provenance`",
                );
            }
        }
        Some(Kind::Export) => {
            if !obj.contains_key("provenance") {
                d.report(
                    SchemaCode::MissingProvenance,
                    "/provenance",
                    "an export requires `provenance`",
                );
            }
            if let Some(r) = obj.get("records") {
                check_records(r, "/records", &mut d);
            }
        }
        None => {}
    }

    d.finish()
}

fn strict_game(kind: Option<Kind>) -> bool {
    matches!(kind, Some(Kind::Strict))
}

fn check_schema_version(obj: &Map<String, Value>, d: &mut SchemaDiagnostics) {
    match obj.get("schema") {
        None => d.report(SchemaCode::MissingField, "/schema", "`schema` is required"),
        Some(v) => {
            // Accept any numeric value equal to one, including the `1.0` spelling.
            // A Draft 2020-12 validator compares `const: 1` numerically, so `1.0`
            // satisfies it; matching that here keeps the published schema and this
            // validator in agreement. A non-numeric or non-integral value (a
            // string `"1"`, or `1.5`) is still unsupported.
            let ok = v.is_number() && v.as_f64() == Some(1.0);
            if !ok {
                d.report(
                    SchemaCode::UnsupportedSchema,
                    "/schema",
                    "unsupported schema version; this build supports version 1",
                );
            }
        }
    }
}

fn check_kind(obj: &Map<String, Value>, d: &mut SchemaDiagnostics) -> Option<Kind> {
    match obj.get("kind") {
        None => {
            d.report(SchemaCode::MissingKind, "/kind", "`kind` is required");
            None
        }
        Some(Value::String(s)) => match s.as_str() {
            "profile" | "package" => Some(Kind::Strict),
            "hint" => Some(Kind::Hint),
            "export" => Some(Kind::Export),
            _ => {
                d.report(
                    SchemaCode::UnknownKind,
                    "/kind",
                    format!("unknown kind; expected one of {}", KINDS.join(", ")),
                );
                None
            }
        },
        Some(_) => {
            d.report(SchemaCode::WrongType, "/kind", "`kind` must be a string");
            None
        }
    }
}

fn check_fidelity(
    obj: &Map<String, Value>,
    pointer: &str,
    required: bool,
    d: &mut SchemaDiagnostics,
) {
    match obj.get("fidelity") {
        None => {
            if required {
                d.report(
                    SchemaCode::MissingFidelity,
                    pointer,
                    "`fidelity` is required",
                );
            }
        }
        Some(Value::String(s)) => {
            if !FIDELITY_TIERS.contains(&s.as_str()) {
                d.report(
                    SchemaCode::InvalidFidelity,
                    pointer,
                    format!(
                        "invalid fidelity; expected one of {}",
                        FIDELITY_TIERS.join(", ")
                    ),
                );
            }
        }
        Some(_) => d.report(
            SchemaCode::WrongType,
            pointer,
            "`fidelity` must be a string",
        ),
    }
}

fn allowed_top_keys(kind: Option<Kind>) -> &'static [&'static str] {
    match kind {
        Some(Kind::Export) => &[
            "schema",
            "kind",
            "fidelity",
            "notes",
            "provenance",
            "game",
            "capture",
            "stage",
            "records",
        ],
        _ => &[
            "schema",
            "kind",
            "fidelity",
            "notes",
            "provenance",
            "game",
            "capture",
            "stage",
        ],
    }
}

fn check_unknown_top_keys(obj: &Map<String, Value>, kind: Option<Kind>, d: &mut SchemaDiagnostics) {
    check_unknown_keys(obj, "", allowed_top_keys(kind), d);
}

fn check_unknown_keys(
    obj: &Map<String, Value>,
    base: &str,
    allowed: &[&str],
    d: &mut SchemaDiagnostics,
) {
    for key in obj.keys() {
        if !allowed.contains(&key.as_str()) {
            d.report(
                SchemaCode::UnknownKey,
                join(base, key),
                format!("unknown key `{key}`"),
            );
        }
    }
}

fn check_game(v: &Value, pointer: &str, require_id_name: bool, d: &mut SchemaDiagnostics) {
    let obj = match v.as_object() {
        Some(o) => o,
        None => {
            d.report(SchemaCode::WrongType, pointer, "`game` must be an object");
            return;
        }
    };
    check_unknown_keys(obj, pointer, &["id", "name", "platform", "app_id"], d);

    match obj.get("id") {
        None => {
            if require_id_name {
                d.report(
                    SchemaCode::MissingField,
                    join(pointer, "id"),
                    "`game.id` is required",
                );
            }
        }
        Some(Value::String(s)) => {
            if !is_slug(s) {
                d.report(
                    SchemaCode::InvalidSlug,
                    join(pointer, "id"),
                    "`game.id` must match [a-z0-9_-]+",
                );
            }
        }
        Some(_) => d.report(
            SchemaCode::WrongType,
            join(pointer, "id"),
            "`game.id` must be a string",
        ),
    }

    match obj.get("name") {
        None => {
            if require_id_name {
                d.report(
                    SchemaCode::MissingField,
                    join(pointer, "name"),
                    "`game.name` is required",
                );
            }
        }
        Some(Value::String(s)) => {
            if s.is_empty() {
                d.report(
                    SchemaCode::EmptyString,
                    join(pointer, "name"),
                    "`game.name` must not be empty",
                );
            }
        }
        Some(_) => d.report(
            SchemaCode::WrongType,
            join(pointer, "name"),
            "`game.name` must be a string",
        ),
    }

    for key in ["platform", "app_id"] {
        if let Some(val) = obj.get(key) {
            if !val.is_string() {
                d.report(
                    SchemaCode::WrongType,
                    join(pointer, key),
                    format!("`game.{key}` must be a string"),
                );
            }
        }
    }
}

fn check_capture(v: &Value, pointer: &str, d: &mut SchemaDiagnostics) {
    let obj = match v.as_object() {
        Some(o) => o,
        None => {
            d.report(
                SchemaCode::WrongType,
                pointer,
                "`capture` must be an object",
            );
            return;
        }
    };
    check_unknown_keys(
        obj,
        pointer,
        &["mode", "duration", "roles", "loopback", "payload"],
        d,
    );
    if let Some(m) = obj.get("mode") {
        match m.as_str() {
            Some(s) if MODES.contains(&s) => {}
            Some(_) => d.report(
                SchemaCode::InvalidMode,
                join(pointer, "mode"),
                format!("`capture.mode` must be one of {}", MODES.join(", ")),
            ),
            None => d.report(
                SchemaCode::WrongType,
                join(pointer, "mode"),
                "`capture.mode` must be a string",
            ),
        }
    }
    if let Some(v) = obj.get("duration") {
        if !v.is_string() {
            d.report(
                SchemaCode::WrongType,
                join(pointer, "duration"),
                "`capture.duration` must be a string",
            );
        }
    }
    if let Some(v) = obj.get("roles") {
        match v.as_array() {
            None => d.report(
                SchemaCode::WrongType,
                join(pointer, "roles"),
                "`capture.roles` must be an array",
            ),
            Some(a) => {
                for (i, item) in a.iter().enumerate() {
                    if !item.is_string() {
                        d.report(
                            SchemaCode::WrongType,
                            format!("{}/roles/{i}", pointer),
                            "each role must be a string",
                        );
                    }
                }
            }
        }
    }
    for key in ["loopback", "payload"] {
        if let Some(val) = obj.get(key) {
            if !val.is_boolean() {
                d.report(
                    SchemaCode::WrongType,
                    join(pointer, key),
                    format!("`capture.{key}` must be a boolean"),
                );
            }
        }
    }
}

fn check_stage_array(v: &Value, pointer: &str, d: &mut SchemaDiagnostics) {
    let arr = match v.as_array() {
        Some(a) => a,
        None => {
            d.report(SchemaCode::WrongType, pointer, "`stage` must be an array");
            return;
        }
    };
    for (i, item) in arr.iter().enumerate() {
        check_stage(item, &format!("{pointer}/{i}"), d);
    }
}

fn check_stage(v: &Value, pointer: &str, d: &mut SchemaDiagnostics) {
    let obj = match v.as_object() {
        Some(o) => o,
        None => {
            d.report(SchemaCode::WrongType, pointer, "a stage must be an object");
            return;
        }
    };
    check_unknown_keys(obj, pointer, &["role", "lifecycle", "terminal", "match"], d);

    match obj.get("role") {
        None => d.report(
            SchemaCode::MissingField,
            join(pointer, "role"),
            "`role` is required",
        ),
        Some(Value::String(s)) if s.is_empty() => d.report(
            SchemaCode::EmptyString,
            join(pointer, "role"),
            "`role` must not be empty",
        ),
        Some(Value::String(_)) => {}
        Some(_) => d.report(
            SchemaCode::WrongType,
            join(pointer, "role"),
            "`role` must be a string",
        ),
    }

    match obj.get("lifecycle") {
        None => d.report(
            SchemaCode::MissingField,
            join(pointer, "lifecycle"),
            "`lifecycle` is required",
        ),
        Some(v) => match v.as_str() {
            Some(s) if LIFECYCLES.contains(&s) => {}
            Some(_) => d.report(
                SchemaCode::InvalidLifecycle,
                join(pointer, "lifecycle"),
                format!("`lifecycle` must be one of {}", LIFECYCLES.join(", ")),
            ),
            None => d.report(
                SchemaCode::WrongType,
                join(pointer, "lifecycle"),
                "`lifecycle` must be a string",
            ),
        },
    }

    if let Some(t) = obj.get("terminal") {
        if !t.is_boolean() {
            d.report(
                SchemaCode::WrongType,
                join(pointer, "terminal"),
                "`terminal` must be a boolean",
            );
        }
    }

    match obj.get("match") {
        None => d.report(
            SchemaCode::MissingField,
            join(pointer, "match"),
            "`match` is required",
        ),
        Some(m) => check_match(m, &join(pointer, "match"), d),
    }
}

fn check_match(v: &Value, pointer: &str, d: &mut SchemaDiagnostics) {
    let obj = match v.as_object() {
        Some(o) => o,
        None => {
            d.report(SchemaCode::WrongType, pointer, "`match` must be an object");
            return;
        }
    };
    check_unknown_keys(obj, pointer, &MATCH_PREDICATES, d);

    let mut predicates = 0usize;
    for key in MATCH_PREDICATES {
        if let Some(val) = obj.get(key) {
            predicates += 1;
            if !val.is_string() {
                d.report(
                    SchemaCode::WrongType,
                    join(pointer, key),
                    format!("`{key}` must be a string"),
                );
            }
        }
    }
    if predicates == 0 {
        d.report(
            SchemaCode::EmptyMatch,
            pointer,
            "`match` needs at least one predicate; an empty match would match every process",
        );
    }
}

fn check_provenance(v: &Value, pointer: &str, d: &mut SchemaDiagnostics) {
    let obj = match v.as_object() {
        Some(o) => o,
        None => {
            d.report(
                SchemaCode::WrongType,
                pointer,
                "`provenance` must be an object",
            );
            return;
        }
    };
    check_unknown_keys(obj, pointer, &["source", "seeded_at"], d);
    match obj.get("source") {
        None => d.report(
            SchemaCode::MissingField,
            join(pointer, "source"),
            "`provenance.source` is required",
        ),
        Some(Value::String(s)) if s.is_empty() => d.report(
            SchemaCode::EmptyString,
            join(pointer, "source"),
            "`provenance.source` must not be empty",
        ),
        Some(Value::String(_)) => {}
        Some(_) => d.report(
            SchemaCode::WrongType,
            join(pointer, "source"),
            "`provenance.source` must be a string",
        ),
    }
    if let Some(val) = obj.get("seeded_at") {
        if !val.is_string() {
            d.report(
                SchemaCode::WrongType,
                join(pointer, "seeded_at"),
                "`provenance.seeded_at` must be a string",
            );
        }
    }
}

fn check_records(v: &Value, pointer: &str, d: &mut SchemaDiagnostics) {
    let arr = match v.as_array() {
        Some(a) => a,
        None => {
            d.report(SchemaCode::WrongType, pointer, "`records` must be an array");
            return;
        }
    };
    for (i, item) in arr.iter().enumerate() {
        let rp = format!("{pointer}/{i}");
        let obj = match item.as_object() {
            Some(o) => o,
            None => {
                d.report(
                    SchemaCode::WrongType,
                    &rp,
                    "each export record must be an object",
                );
                continue;
            }
        };
        check_unknown_keys(
            obj,
            &rp,
            &[
                "fidelity",
                "provenance",
                "notes",
                "game",
                "capture",
                "stage",
            ],
            d,
        );
        check_fidelity(obj, &join(&rp, "fidelity"), true, d);
        match obj.get("provenance") {
            None => d.report(
                SchemaCode::MissingProvenance,
                join(&rp, "provenance"),
                "each export record requires `provenance`",
            ),
            Some(p) => check_provenance(p, &join(&rp, "provenance"), d),
        }
        if let Some(g) = obj.get("game") {
            check_game(g, &join(&rp, "game"), false, d);
        }
        if let Some(c) = obj.get("capture") {
            check_capture(c, &join(&rp, "capture"), d);
        }
        if let Some(s) = obj.get("stage") {
            check_stage_array(s, &join(&rp, "stage"), d);
        }
        if let Some(n) = obj.get("notes") {
            if !n.is_string() {
                d.report(
                    SchemaCode::WrongType,
                    join(&rp, "notes"),
                    "`notes` must be a string",
                );
            }
        }
    }
}

fn is_slug(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Join a JSON-pointer base with a child key, escaping per RFC 6901.
fn join(base: &str, key: &str) -> String {
    let escaped = key.replace('~', "~0").replace('/', "~1");
    format!("{base}/{escaped}")
}
