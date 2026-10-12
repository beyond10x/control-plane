This is the unmodified ESS 0.59.0 authoring schema from commit
`56bd965b4a54e4f57afdcf1c1868317bcd0237b4`, path
`schemas/generated/ess.schema.json` in beyond10x/ess.

ESS generates it in `crates/edge/ess-xtask/src/main.rs`, `schema`, using
`schemars::schema_for!(ess_domain::spec::RawSpecFile)`. ESS owns all syntax and
semantic validation. Control-plane only indexes and transports bounded fragments.
The resource is used only when `ess specify toolchain which`, run from the
specification directory, selects this exact release. Upgrade the resource from
the ESS release together with its recorded source and digest test; never edit
the schema to repair a refused specification.
