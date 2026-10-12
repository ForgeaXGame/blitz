# ForgeaX maintained UI source

Upstream base: `74fe1abf090732524c86d96e6cdb6f95d8f8d6dc` from DioxusLabs/Blitz.
The Engine gitlink pins the adopted revision; its `packages/ui-blitz/Cargo.lock`
fixes the resolved headless dependency graph. Upstream licenses remain unchanged.

| Repair | Regression |
| --- | --- |
| `innerHTML` detaches old children without destroying referenced node identities | `mutator::test::inner_html_keeps_detached_node_references_live` |
| Input current values exist before layout, retain the standard dirty-value distinction, and survive cloning | `mutator::test::input_value_precedes_layout_and_survives_default_attribute_changes` |
| Open ShadowRoot uses native Stylo cascades, flattened inheritance/layout and scoped DOM queries; fragments retain real node identity | `shadow::tests::shadow_scope_mutation_detachment_and_drop_keep_one_tree`; Engine real-WASM/browser tests cover retargeting, inherited pixels and control hits |
| Text inputs and textarea measure `normal` intrinsic height from an empty Parley line at the current computed font and viewport scale, including leading; value/fallback glyphs do not resize controls. Measurement runs only when Taffy requests it. Buttons use border-box dimensions | Engine UI core glyph/height and complete-guide browser rectangle, empty/fallback value and explicit line-height regressions |
| Prepared bootstrap fonts populate native script fallbacks as well as generic families; named author fonts can fall back to published subsets without OS discovery | Engine `authored_latin_family_uses_published_bootstrap_for_missing_arrow` and the generated-WASM Latin-only UiAsset arrow comparison |
| Every document registers its builtin Bullet font, including supplied font contexts; disclosure and list markers do not depend on host font coverage | `document::font_face_override_tests::supplied_font_context_retains_builtin_marker_font`; Engine generated-WASM implicit/explicit Bullet pixel comparison covers details, lists and controls |
| Checkbox/radio checkedness has one native ElementState, a dirty-default flag and DOM/form-scoped radio exclusion; script state is available before layout | Engine `checked_state_defaults_and_radio_groups_have_native_ownership`; real-WASM/Chromium property, cancellation, pointer, label and keyboard comparisons |
| Removed font/stylesheet nodes cannot adopt late resource results | Engine `removed_font_stylesheet_rejects_its_late_response` |
| Range controls retain native current/default state and min/max/step sanitization; the existing CPU painter draws track and thumb from that same value | Engine `range_value_mutations_and_pointer_defaults_move_native_thumb_pixels`; ordinary settings UiAsset pointer/keyboard and value operations against Chromium |
| CSSOM client rectangles include the resolved node/ancestor transforms used by painting and hit testing; offset geometry stays untransformed | Engine `transformed_range_client_rects_hit_the_same_painted_control` |
| Single select values and dirty option defaults stay on native nodes; layout and the viewport popup share one option projection and pointer rectangle | Engine `select_defaults_values_and_native_popup_commit_share_option_state`; Chromium differential settings consumer |
| Checkbox/radio accent is independent of inherited text color; checkbox tick geometry includes its painted frame origin | Engine `white_label_checkbox_keeps_checked_fill_and_tick_visible_at_its_position` |
| `display:none` cancels CSS animation sets in the flattened subtree after style resolution; hidden loading spinners stop requesting raster work, and showing the same nodes restarts their animations | Engine `hidden_loading_subtree_cancels_ticks_and_showing_restarts_animation` and the matching generated-WASM DOM regression |
| Hidden flattened descendants expose zero CSSOM bounds/offsets and no client rectangles; `display:contents` ancestors preserve child boxes and show restores the same identities | Engine `hidden_shadow_descendants_have_no_cssom_boxes_and_restore_identity`; real game guide landscape/portrait/landscape keyboard regression |
| Layout shrink reconciles existing element and viewport scroll offsets through the scrolling owner's current range before geometry/paint | Engine `layout_shrink_clamps_existing_scroll_offsets_before_geometry_reads`; real game guide scroll/resize/DPR regression |
| A scroller's own CSSOM border box stays fixed while its descendants include ancestor scroll offsets, including writing-mode projection | Engine `scrolling_moves_descendant_client_rects_but_not_the_scrollport`; measured game guide rectangles before/after keyboard scrolling |
| Constraint-validation messages supplied by the standard DOM facade use native shaped text in a viewport popup; input edits, focus retirement and removed controls clear it | Engine `validation_feedback_uses_native_text_and_clears_with_control_lifecycle`; generated-WASM canceled-invalid/default-feedback and game number-form regressions |
| Every focusable control retains the solid UA `:focus` outline. The separate `:focus-visible` matcher reads native `FOCUSRING`; Document owns the last accepted focus modality, keyboard exposure and editable-control visibility. Script transfers and cancelled pointer defaults retain modality | Engine `native_focus_outline_paints_and_retires_with_control_eligibility`, real-WASM `focus-visible.unit.test.ts` and unchanged capability-lab HUD Chromium differential. `:focus-within` remains rejected by Engine's portable source admission |
| Changing a current-value input into checkbox/radio transfers its nonempty value into the existing value attribute before checkedness initialization; text editing rejects non-text nodes | Engine `native_text_defaults_survive_control_type_changes_without_poisoning`; beforeinput retirement and value/default-value round-trip Chromium differentials |
| Input and textarea UA styles pair a black foreground with their existing white background, preserving normal authored color overrides | Engine default-control Chromium/WASM computed-color differential, interior glyph-ink checks and paired screenshots |

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
