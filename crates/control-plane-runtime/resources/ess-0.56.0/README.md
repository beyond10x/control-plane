This is the unmodified ESS 0.56.0 authoring schema from commit
`84ee8d38eb69e3a4507de801fdcb248232ed6e6e`, path
`schemas/generated/ess.schema.json` in beyond10x/ess.

ESS generates it in `crates/edge/ess-xtask/src/main.rs`, `schema`, using
`schemars::schema_for!(ess_domain::spec::RawSpecFile)`. ESS owns all syntax and
semantic validation. Control-plane only indexes and transports bounded fragments.
The resource is used only when `ess specify toolchain which`, run from the
specification directory, selects this exact release. Upgrade the resource from
the ESS release together with its recorded source and digest test; never edit
the schema to repair a refused specification.
