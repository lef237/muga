# Standard Formatting Helpers

Status: `std::fmt` is implemented as a narrow pure formatting package over
explicit `String` values.

This slice improves CLI, report, and generated-app output ergonomics without
adding language interpolation, implicit conversion, format-builder state,
localization, or runtime host effects.

## Goals

Short-term: make common status lines, fixed-width columns, and clipped labels
readable without long `.concat(...)` chains or ad hoc loops in app code.

Medium-term: keep formatting APIs explicit so callers decide where `to_string`
conversion happens and generated apps can use artifact-backed stdlib packages
without private source fallback.

Long-term: reserve template/interpolation syntax, localization, format
specifiers, and builder APIs for later contracts that can define escaping,
argument typing, allocation behavior, and error policy together.

Final goal: move Muga toward practical adoption by improving everyday output
quality while keeping the v1 standard-library boundary small and predictable.

## Public Shape

```muga
pub fn repeat(text: String, count: Int): String
pub fn pad_left(text: String, width: Int, fill: String): String
pub fn pad_right(text: String, width: Int, fill: String): String
pub fn truncate_chars(text: String, max_chars: Int): String
pub enum FormatError {
  MissingValue(Int)
  UnclosedPlaceholder(Int)
  UnexpectedClose(Int)
}
pub fn format_values(template: String, values: List[String]): Result[String, FormatError]
```

`repeat` returns `text` repeated `count` times; zero or negative counts return
the empty string. `pad_left` and `pad_right` use Unicode scalar-value
`char_count()` as the width unit. They use the first scalar of `fill` as the
padding unit; an empty `fill` produces no padding. Widths shorter than the
input return the original text. `truncate_chars` keeps the first `max_chars`
Unicode scalar values; zero or negative limits return the empty string.
`format_values` substitutes each `{}` placeholder with the next explicit
`String` value. `{{` writes a literal `{`, and `}}` writes a literal `}`.
Missing values, unclosed placeholders, and stray `}` braces return
`FormatError` with the zero-based scalar offset or missing value index. Extra
values are ignored.

The package is implemented as ordinary virtual stdlib source. It depends only
on existing `String` helpers, `while`, and explicit `Int` arithmetic.

## Candidates Compared

| Candidate | Benefit | Cost | Decision |
|---|---|---|---|
| Add explicit text-layout helpers | Covers padding, rulers, and clipping with no new runtime or error type. | Does not provide placeholder substitution or localization. | Select |
| Add explicit `fmt::format_values(template, values)` | Reduces many string assembly call sites while keeping all conversion explicit. | Supports only `{}` and escaped braces; no named fields, width specifiers, or localization. | Select |
| Add language interpolation syntax | Most ergonomic for app authors. | Syntax, escaping, type conversion, and formatter dispatch are too broad for this slice. | Defer |
| Add mutable format builders | Useful for large generated text. | Requires builder ownership, allocation, and later buffer semantics. | Defer |

## Deferred Policy

- non-string conversion remains explicit through `to_string()`.
- language interpolation syntax, named placeholders, format specifiers,
  localization, and builders remain separate slices.
- grapheme-cluster display width, terminal-width awareness, ANSI styling, and
  table rendering are not part of this first layout package.

## Validation

- `package_std_fmt_sample_runs`
- `standard_fmt_layout_helpers_run_as_virtual_package`
- `standard_fmt_format_values_runs_as_virtual_package`
- `standard_fmt_format_values_reports_template_errors`
- `standard_fmt_helpers_report_type_mismatches`
- `standard_fmt_missing_import_suggests_import`
- `standard_fmt_artifact_run_uses_emitted_std_implementations`
- `std_fmt_text_layout_helpers_are_implemented_and_covered`
