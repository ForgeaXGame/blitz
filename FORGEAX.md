# ForgeaX maintained UI source

Upstream base: `74fe1abf090732524c86d96e6cdb6f95d8f8d6dc` from DioxusLabs/Blitz.
The Engine gitlink pins the adopted revision; its `packages/ui-blitz/Cargo.lock`
fixes the resolved headless dependency graph. Upstream licenses remain unchanged.

| Repair | Regression |
| --- | --- |
| `innerHTML` detaches old children without destroying referenced node identities | `mutator::test::inner_html_keeps_detached_node_references_live` |
| Input current values exist before layout, retain the standard dirty-value distinction, and survive cloning | `mutator::test::input_value_precedes_layout_and_survives_default_attribute_changes` |
| Open ShadowRoot uses native Stylo cascades, flattened inheritance/layout and scoped DOM queries; fragments retain real node identity | `shadow::tests::shadow_scope_mutation_detachment_and_drop_keep_one_tree`; Engine real-WASM/browser tests cover retargeting, inherited pixels and control hits |
| Number controls reserve the same intrinsic text height as other single-line editors; buttons use border-box dimensions | Engine UI core glyph/height and browser rectangle regressions |
| Prepared bootstrap fonts populate native script fallbacks as well as generic families; named author fonts can fall back to published subsets without OS discovery | Engine `authored_latin_family_uses_published_bootstrap_for_missing_arrow` and the generated-WASM Latin-only UiAsset arrow comparison |
| Every document registers its builtin Bullet font, including supplied font contexts; disclosure and list markers do not depend on host font coverage | `document::font_face_override_tests::supplied_font_context_retains_builtin_marker_font`; Engine generated-WASM implicit/explicit Bullet pixel comparison covers details, lists and controls |
| Removed font/stylesheet nodes cannot adopt late resource results | Engine `removed_font_stylesheet_rejects_its_late_response` |

`BaseDocument::input_value` and `set_input_value` share the same Parley editor used
for layout and editing. Layout no longer creates a second value authority. Native
editing marks the current value dirty through the existing generated input event.
Embedders explicitly collect unreachable detached subtrees or release the document;
`remove_and_drop_node` remains available for those ownership decisions.

Engine's binding runs sequential style resolution with default features disabled,
and exercises dynamic Grid, input, retained nodes and owned RGBA through the real
WASM module. It does not use a second JavaScript engine.

The upstream headless test prerequisites needed two repairs: the hover fixture
created elements in the empty namespace instead of HTML, and scrolling tests
laid out text without any available font. They now use the HTML namespace and a
licensed prepared [test font](packages/blitz-dom/tests/fonts/README.md). The same
original hover, scroll-clamp and bubbling assertions pass without system fonts.
Engine's consumer regressions and browser differential tests remain separate gates.
