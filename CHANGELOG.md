# Changelog

All notable changes to this project will be documented in this file.

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
