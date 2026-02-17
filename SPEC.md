<idea>
# lite-room

## Lightweight Non Destructive RAW Photo Editor

---

# 1. Vision

Build a high performance desktop RAW photo editor with:

* Non destructive editing
* GPU accelerated rendering
* Local catalog management
* Fast preview pipeline

Version 1 focuses on core Lightroom style functionality without AI or cloud features.

---

# 2. Goals

## Functional Goals

* Import images from folder
* Display image grid
* View single image
* Apply basic edits:

  * Exposure
  * Contrast
  * Temperature
  * Tint
  * Highlights
  * Shadows
* Store edits non destructively
* Export edited image to JPEG
* Persist catalog in SQLite

## Non Functional Goals

* Smooth slider response under 50 ms preview latency
* Handle at least 5000 images in catalog
* No destructive changes to original RAW files
* Cross platform with macOS as primary target

---

# 3. Out of Scope for v1

* AI masking
* Brush tools
* Healing tools
* HDR merge
* Panorama
* Face recognition
* Cloud sync
* Mobile support

---

# 4. Tech Stack

## Core Engine

Rust

## GPU Rendering

wgpu

## UI

Option A: Slint
Option B: egui
Final decision after spike phase

## Database

SQLite via rusqlite

## RAW Decoding

libraw bindings

---

# 5. System Architecture

UI Layer
↕
Application Controller
↕
Image Engine

* RAW Decoder
* Edit Pipeline
* GPU Renderer
* Preview Generator
  ↕
  Catalog Database

---

# 6. Functional Specification

## 6.1 Import Images

User selects a folder.

System will:

* Scan for supported formats: CR2, NEF, ARW, DNG, JPEG
* Extract metadata:

  * File path
  * Capture date
  * Camera model
  * ISO
* Generate thumbnail
* Insert record into SQLite

### Acceptance Criteria

* Images appear in grid within 2 seconds for 100 images
* Thumbnails are cached on disk

---

## 6.2 Grid View

Displays:

* Thumbnail
* Rating
* Flag
* Capture date

Supports:

* Sort by date
* Click to open image

---

## 6.3 Edit View

Displays:

* Full preview
* Histogram
* Sliders:

  * Exposure
  * Contrast
  * Temperature
  * Tint
  * Highlights
  * Shadows

Requirements:

* Real time preview updates
* Original RAW file remains unchanged

---

## 6.4 Non Destructive Editing Model

Each image stores:

```
struct EditParams {
    exposure: f32,
    contrast: f32,
    temperature: f32,
    tint: f32,
    highlights: f32,
    shadows: f32,
}
```

These parameters are:

* Serialized as JSON
* Stored in database
* Applied dynamically during rendering

---

## 6.5 Rendering Pipeline

Processing order:

1. Demosaic RAW
2. Linear RGB buffer
3. White balance
4. Exposure adjustment
5. Highlights and shadows recovery
6. Tone curve
7. Gamma correction
8. Convert to display color space
9. Render via GPU framebuffer

### Performance Requirement

Preview must update under 50 ms for a 24 MP image at scaled resolution.

---

## 6.6 Export

User selects:

* JPEG quality
* Output resolution

System will:

* Load RAW
* Apply full resolution pipeline
* Save final JPEG

Export does not modify stored edit parameters.

---

# 7. Data Model

## images table

* id
* file_path
* import_date
* rating
* flag
* metadata_json

## edits table

* image_id
* edit_params_json
* updated_at

## thumbnails table

* image_id
* file_path
* width
* height

---

# 8. Performance Strategy

* Keep RAW buffer in memory during editing session
* Use GPU compute shaders for pixel transforms
* Cache scaled previews
* Recompute only when parameters change
* Avoid unnecessary CPU image copies

---

# 9. Error Handling

* Graceful failure for unsupported RAW formats
* Detect corrupted files
* Auto save edits
* Crash safe catalog commits

---

# 10. Milestones

## Milestone 1

Load and display JPEG

## Milestone 2

Load RAW and display demosaiced image

## Milestone 3

Add exposure slider

## Milestone 4

Move pipeline to GPU

## Milestone 5

Add SQLite catalog

## Milestone 6

Add export

## Milestone 7

Optimization pass

---

# 11. Risks

* RAW decoding complexity
* GPU shader debugging difficulty
* Color science accuracy
* Memory usage with large RAW files

---

# 12. Future Roadmap

* Tone curve UI
* LUT support
* Local adjustments
* Batch editing
* Plugin system

---

# 13. Success Criteria

* Can edit and export a RAW image
* Edits are fully non destructive
* UI feels responsive
* Handles large catalogs without noticeable lag
</idea>
