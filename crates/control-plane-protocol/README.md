# Control-plane protocols

`Protocol` compiles the included `protocol/1` YAML through Canon and returns Canon's real decisions. `engineering.plan/1` completes after current plan validation. `software.change.merge/1` requires current tests, independent review, effect-time merge authority and an observation of the same candidate on its target. It does not require a release or deployment.

The host supplies a current Canon case, `AttestedEvidence`, and `EvaluationContext`. The plan revision must bind the goal, specification and repository inputs. The implementation revision must identify the exact candidate and verification context. Changed inputs need a new revision. Stale evidence stays unknown; contradictory current evidence stays unknown rather than choosing a convenient result.

`EvidenceOrigin` is constructed only by trusted host adapters. It is deliberately not deserializable. Those adapters must actually run validation/tests, establish a separate reviewer context, and verify the repository, target and candidate in a merge receipt. The enum is an assertion by those adapters, not cryptographic proof. The evaluator checks its consistency with the evidence and refuses model-origin evidence. It does not run processes, inspect Git or authenticate a remote. Keep these constructors and authority callbacks outside model-facing tools and HTTP input.

`evaluate` grants no authority. `evaluate_effect` invokes a live host authority callback for the requested action after its preconditions pass, and returns a fresh Canon decision. The host holds its repository lease, supplies a current snapshot and checks the requested action's status immediately before performing it. A returned decision is not a durable permission token. Store effect intent before execution and observed receipts afterward; reconciliation and publication remain the runtime adapter's responsibility.

`ir()` exposes the compiled Canon IR for a Commission governor bridge. This avoids changes to Loom's stock governor and its fixed built-in registry.

The YAML follows the revision-bound evidence, action and authority idioms of Beyond10x Engineering Protocols (`beyond10x/engineering-protocols`, Apache-2.0), particularly `software.change/1`. These are product-specific protocol definitions; Canon owns their semantics.
