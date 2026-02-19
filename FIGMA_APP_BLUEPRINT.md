# lite-room Figma Blueprint (v1)

Use this to build the full app design in Figma with consistent structure for future feature work.

## 1. File Structure in Figma

Pages:
- `00 Foundations`
- `01 App Shell`
- `02 Import Flow`
- `03 Grid View`
- `04 Edit View`
- `05 Export Flow`
- `06 Components`
- `07 Prototypes`

## 2. Foundations (`00 Foundations`)

Create styles/variables first.

Color tokens:
- `bg/app`: `#0F1115`
- `bg/panel`: `#171A21`
- `bg/elevated`: `#1F2430`
- `text/primary`: `#F2F4F8`
- `text/secondary`: `#A6AEBD`
- `text/muted`: `#7E8798`
- `accent/blue`: `#4C8DFF`
- `accent/warn`: `#FFB347`
- `accent/error`: `#FF6B6B`
- `stroke/default`: `#2A3040`
- `stroke/focus`: `#6AA8FF`

Spacing scale:
- `4, 8, 12, 16, 20, 24, 32, 40`

Radius:
- `6, 10, 14`

Typography:
- Family: `Inter` (or `SF Pro` on macOS preview)
- `Display`: 28/34 semibold
- `H1`: 22/28 semibold
- `H2`: 18/24 semibold
- `Body`: 14/20 regular
- `Body Strong`: 14/20 medium
- `Meta`: 12/16 regular

## 3. App Shell (`01 App Shell`)

Desktop frame:
- Size: `1440 x 900`
- Regions:
  - Top bar: 56h
  - Left nav: 240w
  - Main content: fill
  - Bottom filmstrip/status: 88h (optional in v1)

Top bar items:
- App name + catalog path
- Search field
- View mode switch: `Grid | Edit`
- Right actions: `Import`, `Export`, profile/settings icon

Left nav:
- `All Photos`
- `Last Import`
- `Flagged`
- `Rated >= 4`
- `Rejected`

## 4. Import Flow (`02 Import Flow`)

Frames:
- `Import Modal - Empty`
- `Import Modal - Scanning`
- `Import Modal - Complete`

Modal contents:
- Folder picker row
- Supported formats note (`CR2, NEF, ARW, DNG, JPEG`)
- Toggle: `Generate thumbnails during import`
- Progress bar + counters:
  - scanned
  - supported
  - newly imported

## 5. Grid View (`03 Grid View`)

Frame:
- `Grid - Default`

Layout:
- Toolbar row (sort/filter)
- Responsive grid cards (4 columns on desktop)

Image card component:
- Thumbnail 3:2
- Bottom metadata row:
  - capture date
  - camera model
  - iso
- Top-right badges:
  - rating (stars)
  - flag state

States:
- default
- hover
- selected
- processing (thumbnail pending)
- error (unsupported/corrupt)

## 6. Edit View (`04 Edit View`)

Frame:
- `Edit - Default`

Columns:
- Left: filmstrip or image list (220w)
- Center: preview canvas (fluid)
- Right: controls panel (320w)

Right panel sections:
- Histogram
- Basic edits sliders:
  - Exposure
  - Contrast
  - Temperature
  - Tint
  - Highlights
  - Shadows
- Buttons:
  - `Reset`
  - `Copy Settings`
  - `Paste Settings`

Status indicators:
- `Preview: 34 ms`
- `Autosave: Saved`

## 7. Export Flow (`05 Export Flow`)

Frames:
- `Export Modal - Default`
- `Export Modal - In Progress`
- `Export Modal - Complete`

Fields:
- Output folder picker
- JPEG quality slider
- Resolution selector:
  - Original
  - Long edge presets
- File naming pattern

## 8. Component Library (`06 Components`)

Build as reusable variants:
- Buttons: primary/secondary/ghost/destructive + disabled/loading
- Inputs: text/select/slider/toggle
- Image card variants
- Sidebar item variants
- Modal shell
- Toast/inline alert
- Progress bar
- Histogram panel

Naming convention:
- `cmp/button/primary`
- `cmp/input/slider`
- `cmp/card/image`

## 9. Prototype Flows (`07 Prototypes`)

Connect these primary flows:
1. Launch -> `Grid`
2. `Import` -> scanning -> completion -> updated grid
3. Select image -> `Edit`
4. Adjust sliders -> live preview state change
5. `Export` -> completion modal

## 10. Handoff Notes for Agents

When implementing features from this design:
- Business rules in `/lite-room/crates/domain`
- Use-cases/ports in `/lite-room/crates/application`
- IO implementations in `/lite-room/crates/adapters`
- CLI/UI wiring in `/lite-room/crates/drivers`

Use `/lite-room/APP_MOCK_DESIGN.md` as the code-placement source of truth.
