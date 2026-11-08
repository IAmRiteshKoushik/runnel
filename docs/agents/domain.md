# Domain docs

This repo uses a single-context domain layout.

## Read before exploring

Read root `GLOSSARY.md` for domain terms and relevant decisions under
`docs/adr/` before exploring the code.

If these files or directories are absent, proceed silently.
Do not suggest creating them solely because they are missing.
The domain-modeling skill creates them when terms or decisions
need to be recorded.

## Layout

- `GLOSSARY.md`: domain vocabulary for Runnel.
- `docs/adr/`: architecture decision records.

## Use the glossary's vocabulary

Use glossary terms in issue titles, proposals, hypotheses, and tests.
Avoid synonyms that conflict with the glossary.

If a concept is missing, reconsider whether it belongs to the domain.
Record real vocabulary gaps for domain-modeling work.

## Flag decision conflicts

When a proposal contradicts an ADR, identify the ADR and explain
why the decision should be reconsidered.
