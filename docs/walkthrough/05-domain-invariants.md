# Domain Invariants

Domain files:
- [/lite-room/crates/domain/src/image.rs](../../crates/domain/src/image.rs)
- [/lite-room/crates/domain/src/edit.rs](../../crates/domain/src/edit.rs)

## 1. `ImageId` must be positive
`ImageId::new(value)` rejects values `<= 0`.

Impact:
- Prevents invalid IDs from crossing layer boundaries.
- Applies in driver, application, and adapter flows.

## 2. `EditParams` values must be finite
`EditParams::validate()` rejects non-finite numbers.

Impact:
- Protects persistence and preview pipeline from `NaN`/infinite values.

## 3. Image kind comes from extension classification
`detect_image_kind(path)` classifies:
- JPEG: `jpg`, `jpeg`
- RAW: `cr2`, `nef`, `arw`, `dng`
- unsupported: everything else

Impact:
- Controls scanner filtering.
- Controls thumbnail/decoder behavior.
