use std::fs;
use std::path::Path;
use serde::Deserialize;
use terse::{compress, CompressConfig, Tier, TokenMethod};

#[derive(Deserialize)]
struct CompressCase {
    id: String,
    #[allow(dead_code)]
    description: String,
    role: String,
    tiers: Vec<String>,
    input: String,
    output: String,
}

fn fixtures_dir() -> std::path::PathBuf {
    // CARGO_MANIFEST_DIR = crates/terse/ → parent → parent = repo root
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()  // crates/
        .parent().unwrap()  // repo root
        .join("fixtures")
}

fn load(path: &str) -> Vec<CompressCase> {
    let p = fixtures_dir().join(path);
    serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap()
}

fn make_config(tiers: &[String]) -> CompressConfig {
    CompressConfig {
        tiers: tiers.iter().map(|t| match t.as_str() {
            "rules" => Tier::Rules,
            "nlp"   => Tier::Nlp,
            other   => panic!("unknown tier: {}", other),
        }).collect(),
        token_method: TokenMethod::Chars,
    }
}

macro_rules! fixture_suite {
    ($name:ident, $path:expr) => {
        #[test]
        fn $name() {
            let mut failures = vec![];
            for c in load($path) {
                let result = compress(&c.input, &c.role, &make_config(&c.tiers)).unwrap();
                if result.text != c.output {
                    failures.push(format!(
                        "[{}]\n  in:  {:?}\n  exp: {:?}\n  got: {:?}",
                        c.id, c.input, c.output, result.text
                    ));
                }
            }
            if !failures.is_empty() { panic!("{}", failures.join("\n\n")); }
        }
    };
}

fixture_suite!(rules_assistant,     "rules/assistant.json");
fixture_suite!(rules_user,          "rules/user.json");
fixture_suite!(rules_structural,    "rules/structural.json");
fixture_suite!(rules_substitutions, "rules/substitutions.json");
fixture_suite!(rules_fillers,       "rules/fillers.json");
fixture_suite!(rules_markdown,      "rules/markdown.json");
fixture_suite!(rules_whitespace,    "rules/whitespace.json");
fixture_suite!(nlp,                 "nlp/nlp.json");
fixture_suite!(pipeline_protected,  "pipeline/protected-blocks.json");
