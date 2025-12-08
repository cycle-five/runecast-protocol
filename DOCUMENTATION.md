# Documentation Deployment

This repository uses GitHub Actions to automatically build and publish API documentation to GitHub Pages.

## How It Works

The [docs.yml](.github/workflows/docs.yml) workflow:

1. **Triggers** on every push to `master` branch (or manual workflow dispatch)
2. **Builds** the Rust documentation using `cargo doc --no-deps`
3. **Prepares** the docs folder with:
   - All generated documentation files
   - A redirect `index.html` that points to the main crate docs
   - A `.nojekyll` file to ensure GitHub Pages doesn't ignore files starting with `_`
4. **Commits** the `docs/` folder back to the repository
5. **Publishes** to GitHub Pages (served from the `docs` folder)

## Configuring GitHub Pages

To enable GitHub Pages for this repository:

1. Go to **Settings** → **Pages**
2. Under **Source**, select:
   - **Branch**: `master` (or your main branch)
   - **Folder**: `/docs`
3. Click **Save**

GitHub Pages will then be available at: `https://cycle-five.github.io/runecast-protocol/`

## Local Documentation

To view the documentation locally:

```bash
cargo doc --no-deps --open
```

This will build and open the documentation in your default browser.

## Documentation Standards

All public API items should have documentation comments:

- **Modules**: Describe the purpose and contents
- **Structs/Enums**: Explain what they represent
- **Fields**: Document important fields (especially public ones)
- **Functions**: Explain parameters, return values, and behavior
- **Examples**: Include usage examples where helpful

The codebase follows these standards and `cargo doc` produces no warnings.
