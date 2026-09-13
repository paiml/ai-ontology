# Ontologies for AI Engineers

**Knowledge Graphs, SHACL, and Grounded Agents**

Code from the course on grounding agent output in a domain model

The constraint that makes an agent's output trustworthy sits OUTSIDE the model
and never enters the prompt. Each example here is self-contained and runnable,
and each one fails closed: a check that crashes, or never runs, does not return
a pass.

## Examples

| # | example | what it covers |
|---|---|---|
| 1 | [`examples/gate-the-write`](examples/gate-the-write) | constraints compiled from an ontology, applied to output the model never saw them with |

## What is not here

Course media and course metadata live elsewhere, deliberately:

| | |
|---|---|
| this repository | learner-facing code examples, and nothing else |
| private catalogue | transcripts, outlines, narration scripts, course config |
| network storage | rendered video, audio, frame sequences |

`.gitignore` enforces the split mechanically. That is not tidiness — a public
repository once received course metadata and transcripts by accident, and the
three-home rule exists because of it.

---

<sub>Generated — do not edit by hand. Edit `course.toml` and re-render.</sub>
