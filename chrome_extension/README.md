# PDF Comment Analyzer Chrome Extension

## Install in Chrome

1. Open Chrome and navigate to `chrome://extensions`.
2. Enable `Developer mode` in the top-right corner.
3. Click `Load unpacked` and select the `chrome_extension` folder.

## How to use

1. Open the extension by clicking its icon.
2. Choose a PDF file using the file picker.
3. Click `Analyze PDF`.
4. Review the generated comments in the popup.

## Notes

- The extension parses PDF text locally in the browser using `pdf.js`.
- It currently provides heuristic comments on page length, paragraph length, repetition, and readability patterns.
- If the PDF is scanned or image-only, text extraction may fail.
