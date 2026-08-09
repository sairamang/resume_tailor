# Resume Tailor

Resume Tailor is a Rust CLI tool that reads a resume or text file, extracts text from PDFs, and optionally analyzes the content with an AI provider.

## Build

```bash
cargo build --release
```

## Run

```bash
cargo run -- <input-file>
```

## Analysis mode

Use `--analyze` to send resume content to the AI provider and print a score, summary, strengths, and improvements:

```bash
cargo run -- <input-file> --analyze
```

Example:

```bash
cargo run -- Sai_Raman_Gowthaman_Resume_v2.pdf --analyze
```

The CLI supports both plain text files and PDF files. For PDFs, text is extracted automatically.

## Environment variables

The tool reads AI provider configuration from environment variables.

### Required

- `API_KEY` — the AI provider API key. This can be an OpenAI-compatible token or a Gemini-style key.

### Optional

- `AI_BASE_URL` — custom base URL for the provider API. Defaults to `https://api.openai.com/v1`.
- `AI_ENDPOINT` — full chat completions endpoint URL. Useful for Gemini or custom gateway endpoints.
- `AI_MODEL` — model name to use. Defaults to `gpt-4o-mini`.

## Gemini-style keys

If you use a Gemini-style API key that starts with `AQ.`, set `AI_ENDPOINT` or use a Gemini-compatible gateway URL in `AI_BASE_URL`.

Example:

```bash
export API_KEY="AQ.your_gemini_key_here"
export AI_ENDPOINT="https://generativelanguage.googleapis.com/v1beta/openai/"
export AI_MODEL="gemini-3.5-flash"
```

## Example setup

Create a shell environment and run the CLI:

```bash
export API_KEY="your_api_key"
export AI_BASE_URL="https://api.openai.com/v1"
export AI_MODEL="gpt-4o-mini"

cargo run -- <input-file> --analyze
```

Or for Gemini keys:

```bash
export API_KEY="AQ.your_gemini_key_here"
export AI_ENDPOINT="https://generativelanguage.googleapis.com/v1beta/openai/"
export AI_MODEL="gemini-3.5-flash"

cargo run -- <input-file> --analyze
```

## Notes

- If `API_KEY` is missing, the CLI will fail with an error.
- If using a Gemini-style key without `AI_ENDPOINT`, the CLI will prompt you to set it.
