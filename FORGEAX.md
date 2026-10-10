# ForgeaX maintained UI source

Upstream base: `74fe1abf090732524c86d96e6cdb6f95d8f8d6dc` from DioxusLabs/Blitz.
The Engine gitlink pins the adopted revision; its `packages/ui-blitz/Cargo.lock`
fixes the resolved headless dependency graph. Upstream licenses remain unchanged.

| Repair | Regression |
| --- | --- |
| `innerHTML` detaches old children without destroying referenced node identities | `mutator::test::inner_html_keeps_detached_node_references_live` |
| Input current values exist before layout, retain the standard dirty-value distinction, and survive cloning | `mutator::test::input_value_precedes_layout_and_survives_default_attribute_changes` |

`BaseDocument::input_value` and `set_input_value` share the same Parley editor used
for layout and editing. Layout no longer creates a second value authority. Native
editing marks the current value dirty through the existing generated input event.
Embedders explicitly collect unreachable detached subtrees or release the document;
`remove_and_drop_node` remains available for those ownership decisions.

Engine's binding runs sequential style resolution with default features disabled,
and exercises dynamic Grid, input, retained nodes and owned RGBA through the real
WASM module. It does not use a second JavaScript engine.

The headless `blitz-dom` test run at this base has two existing failures:
`hover_styles_apply_and_clear` and `scroll_by_clamps_and_bubbles`. Both were
reproduced on unmodified upstream under the same resolved features. They remain
failures; they are not evidence of hover or font-layout acceptance. Engine's
specific consumer regressions and browser differential tests are separate gates.
