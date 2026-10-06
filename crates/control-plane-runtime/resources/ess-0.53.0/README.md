This is the unmodified ESS 0.53.0 authoring schema from commit
`a81a8729dc252830d4e0557176522b59be6ff253`, path
`schemas/generated/ess.schema.json` in beyond10x/ess.

ESS generates it in `crates/edge/ess-xtask/src/main.rs`, `schema`, using
`schemars::schema_for!(ess_domain::spec::RawSpecFile)`. ESS owns all syntax and
semantic validation. Control-plane only indexes and transports bounded fragments.
The resource is used only when `ess specify toolchain which`, run from the
specification directory, selects this exact release. Upgrade the resource from
the ESS release together with its recorded source and digest test; never edit
the schema to repair a refused specification.
