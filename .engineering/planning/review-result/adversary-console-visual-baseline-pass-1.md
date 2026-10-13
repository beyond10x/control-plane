---
format: aep.planning-md/3
id: review-result:adversary-console-visual-baseline-pass-1
kind: review-result
status: active
title: 'Adversary pass 1 on story:console-visual-baseline (05f7c8e): needs change, 3 introduced'
relations:
- reviews: story:console-visual-baseline
revision: 1
---
Adversary pass 1 on story:console-visual-baseline at 05f7c8e: NEEDS-CHANGE, 3 introduced, 0 pre-existing.

Cases added: stylesheet_reader_sees_rules_inside_supports_container_and_layer (red), text_meets_contrast_where_a_rule_sets_only_the_background (green, red on a mutant with button.danger at 2.72:1), adversary_projected_command_line_keeps_credential_values_off_the_page (red).

Attacked and not broken: the script-element boundary of the embedded snapshot (`</script>`, `</SCRIPT >`, `<!--`, `<script`, U+2028/2029, control characters); the error page and form paths; leftover server-rendered goal markup or inline styles on `/`, `/live`, `/workspaces/{id}`, `/workspaces/{id}/live`, evidence; snapshot against first SSE frame ordering; unknown workspace; `var()`, `rgb()`, `em`/`rem`, gradients, shorthand and media queries in the style tests; badge and danger mapping; size bounds of projected arguments.

All three were fixed in 99c5c7a; each case passes there unchanged.

```findings
- file: frontend/src/stylesheet.js
  line: 22
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the stylesheet reader skips @supports, @container and @layer blocks, so text_meets_contrast and text_meets_minimum_size pass an 8px #ddd rule written inside one
- file: frontend/src/style.test.js
  line: 26
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: text_meets_contrast never measures a rule that sets only a background over inherited text, so button.danger at 2.72:1 white-on-background stays green
- file: crates/control-plane-app/src/live.rs
  line: 387
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: command_line projects argument values unfiltered into SSE frames and the embedded snapshot, but the implementor run grammar admits only fixed inspection commands, so no path to a credential-carrying argument was found
```
