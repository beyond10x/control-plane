---
format: aep.planning-md/3
id: executable-system-specification:control-plane
kind: executable-system-specification
status: draft
title: Control-plane ess/22 domain
revision: 1
---
The specification in ess/system.yaml and ess/domains/host.yaml uses format ess/22 with ESS 0.53.0. It defines Workspace, RepositoryRegistration, Goal and Assignment. validate passes; synthesis produces 120 scenarios with zero refusals. Generated contracts are in generated/model. AEP remains the planning authority; the standalone control-plane product owns the service and runtime integration.
