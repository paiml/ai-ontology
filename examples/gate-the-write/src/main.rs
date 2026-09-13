//! Gate a proposed write against constraints compiled from a domain model.
//!
//! The argument this example exists to make: the constraints live in
//! `ontology.txt`, the proposal lives in `proposed-*.txt`, and nothing in the
//! proposal can talk its way past the constraints because the two never meet
//! until this program puts them together.
//!
//! Exit codes are the interface:
//!
//! | code | meaning |
//! |------|---------|
//! | 0    | every constraint satisfied — the write may land |
//! | 1    | at least one violation — the write is blocked |
//! | 2    | the gate could not run — ALSO blocked |
//!
//! Code 2 is the one worth dwelling on. A checker that cannot read its own
//! rules has not found the write acceptable; it has found nothing at all, and
//! reporting that as success is how a gate becomes decoration.

use std::collections::BTreeMap;
use std::process::ExitCode;

#[derive(Debug)]
enum Constraint {
    Requires(String),
    OneOf(String, Vec<String>),
    Max(String, i64),
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let proposal_path = args.get(1).map(String::as_str).unwrap_or("proposed-ok.txt");
    let ontology_path = args.get(2).map(String::as_str).unwrap_or("ontology.txt");

    let ontology = match std::fs::read_to_string(ontology_path) {
        Ok(s) => s,
        Err(e) => {
            // FAIL CLOSED. Not "no violations found".
            eprintln!("BLOCKED: cannot read the domain model {ontology_path}: {e}");
            return ExitCode::from(2);
        }
    };
    let proposal = match std::fs::read_to_string(proposal_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("BLOCKED: cannot read the proposal {proposal_path}: {e}");
            return ExitCode::from(2);
        }
    };

    let constraints = compile(&ontology);
    if constraints.is_empty() {
        // An empty rule set would accept everything. That is not a pass.
        eprintln!("BLOCKED: {ontology_path} compiled to zero constraints");
        return ExitCode::from(2);
    }
    let facts = parse_proposal(&proposal);

    println!("domain model : {ontology_path} -> {} constraint(s)", constraints.len());
    println!("proposal     : {proposal_path} -> {} field(s)", facts.len());

    let violations = check(&constraints, &facts);
    if violations.is_empty() {
        println!("\nPASS — every constraint satisfied; the write may land.");
        ExitCode::SUCCESS
    } else {
        println!();
        for v in &violations {
            println!("VIOLATION  {v}");
        }
        println!("\nBLOCKED — {} violation(s). One is enough.", violations.len());
        ExitCode::from(1)
    }
}

/// Compile the domain model into constraints. This is the step the lesson calls
/// "an ontology compiles into constraints".
fn compile(src: &str) -> Vec<Constraint> {
    let mut out = Vec::new();
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split_whitespace().collect();
        match f.as_slice() {
            [_class, "requires", prop] => out.push(Constraint::Requires((*prop).to_string())),
            [_class, prop, "in", list] => out.push(Constraint::OneOf(
                (*prop).to_string(),
                list.split(',').map(|s| s.trim().to_string()).collect(),
            )),
            [_class, prop, "max", n] => {
                if let Ok(v) = n.parse::<i64>() {
                    out.push(Constraint::Max((*prop).to_string(), v));
                }
            }
            _ => {}
        }
    }
    out
}

fn parse_proposal(src: &str) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once(char::is_whitespace) {
            m.insert(k.to_string(), v.trim().to_string());
        }
    }
    m
}

fn check(constraints: &[Constraint], facts: &BTreeMap<String, String>) -> Vec<String> {
    let mut v = Vec::new();
    for c in constraints {
        match c {
            Constraint::Requires(p) => {
                if !facts.contains_key(p) {
                    v.push(format!("missing required property `{p}`"));
                }
            }
            Constraint::OneOf(p, allowed) => {
                if let Some(got) = facts.get(p) {
                    if !allowed.contains(got) {
                        v.push(format!(
                            "`{p}` is `{got}`, which is not one of {}",
                            allowed.join(", ")
                        ));
                    }
                }
            }
            Constraint::Max(p, max) => {
                if let Some(got) = facts.get(p) {
                    match got.parse::<i64>() {
                        Ok(n) if n > *max => {
                            v.push(format!("`{p}` is {n}, over the maximum of {max}"))
                        }
                        // A value the rule cannot evaluate is a violation, not a pass.
                        Err(_) => v.push(format!("`{p}` is `{got}`, which is not a number")),
                        _ => {}
                    }
                }
            }
        }
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clean_proposal_passes() {
        let c = compile("Dataset requires owner\nDataset retention_days max 365");
        let f = parse_proposal("type Dataset\nowner me\nretention_days 90");
        assert!(check(&c, &f).is_empty());
    }

    #[test]
    fn each_rule_kind_can_actually_fail() {
        let c = compile(
            "Dataset requires owner\nDataset license in CC-BY,MIT\nDataset retention_days max 365",
        );
        let f = parse_proposal("type Dataset\nlicense WTFPL\nretention_days 400");
        let v = check(&c, &f);
        assert_eq!(v.len(), 3, "missing owner, bad licence, over cap: {v:?}");
    }

    /// The anti-vacuity arm. A rule set that accepts everything is the failure
    /// mode this whole course is about, so the gate refuses an empty one.
    #[test]
    fn an_empty_domain_model_compiles_to_nothing_and_must_not_be_treated_as_a_pass() {
        assert!(compile("# only comments\n\n").is_empty());
    }

    /// A value the rule cannot evaluate is a violation. Reporting "no problem
    /// found" for "could not check" is the defect in miniature.
    #[test]
    fn an_unparseable_value_is_a_violation_not_a_pass() {
        let c = compile("Dataset retention_days max 365");
        let f = parse_proposal("type Dataset\nretention_days forever");
        assert_eq!(check(&c, &f).len(), 1);
    }
}
