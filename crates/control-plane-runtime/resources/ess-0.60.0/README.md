This is the unmodified ESS 0.60.0 authoring schema from commit
`19d601013b867f0e6f5061f097fa81d55a734bb1`, path
`schemas/generated/ess.schema.json` in beyond10x/ess.

ESS generates it in `crates/edge/ess-xtask/src/main.rs`, `schema`, using
`schemars::schema_for!(ess_domain::spec::RawSpecFile)`. ESS owns all syntax and
semantic validation. Control-plane only indexes and transports bounded fragments.
The resource is used only when `ess specify toolchain which`, run from the
specification directory, selects this exact release. Upgrade the resource from
the ESS release together with its recorded source and digest test; never edit
the schema to repair a refused specification.
