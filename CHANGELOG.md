# Changelog

All notable changes to this project will be documented in this file.

## [0.1.27] - 2026-06-14
### Added
- Auto-scroll during text selection in Continuous Scroll mode — viewport scrolls
  when dragging near the top/bottom edge to reveal hidden content.

### Changed
- Organised all SVG icons into `assets/icons/` folder with updated internal paths.
- Right-click context menu Zoom In/Out now uses the same scale as Ctrl+Scroll wheel.
- Text selection overlay now merges consecutive characters on the same line into
  continuous highlight rects for a seamless appearance.

### Fixed
- Selection `selection_end` now updates for every page the cursor moves over
  during a drag, so selections spanning multiple pages are highlighted immediately
  without waiting for mouse release.

## [0.1.14] - 2026-06-08
### Changed
- Converted React website components and filenames to lowercase/kebab-case.
- Enhanced annotation tool resilience on Windows with panic recovery and retry loops.

### Fixed
- Fixed an issue where the file could get locked and stuck in a "Saving..." state.
- Fixed highlight color exporting as too dark by accurately applying alpha transparency.
- Removed black from highlight color picker options.


## [0.1.12] - 2026-06-08
### Added
- Text annotation tool with customizable font size, bold, italic, underline, and color options.
- Dynamic text box height and manual resizing.

### Changed
- Color picker overhauled with a clean, dynamic gradient interface.

### Fixed
- Fixed bug where text annotations exported to DOCX would cause the file to be corrupted.
- Fixed text size mismatch between the viewer and the exported PDF.
- Fixed a panic caused by uninitialized page sizes.

## [0.1.9] - 2026-06-04
- Added annotation tools: Highlight, Underline, Strikethrough, Redact.
- Custom UI icons for annotation tools.

### Changed
- Customized application UI colors to use a lighter gray selection color for better visibility.
- Improved dark mode compatibility for tool icons.

### Fixed
- Fixed bug where annotations were not being saved correctly to PDF content stream.
- Fixed content alignment issue when pages are rotated.
