# Gik Distribution & Releases

Gik uses `cargo-dist` for automated cross-platform releases and distribution.

## How to trigger a release

1.  **Update Version**: Ensure `version` in `Cargo.toml` is correct.
2.  **Create Tag**: Create a git tag starting with `v` (e.g., `v0.1.0`).
    ```bash
    git tag v0.1.0
    ```
3.  **Push Tag**: Push the tag to GitHub.
    ```bash
    git push origin v0.1.0
    ```
4.  **GitHub Actions**: The push will trigger the `Release` workflow. It will:
    *   Build binaries for Linux, macOS, and Windows.
    *   Generate Shell and PowerShell installers.
    *   Create a new GitHub Release with all artifacts.

## Installation for Users

Once a release is published, users can install Gik using these commands:

### Linux / macOS (Shell)
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Kyryloloshka/Gik/releases/latest/download/gik-installer.sh | sh
```

### Windows (PowerShell)
```powershell
irm https://github.com/Kyryloloshka/Gik/releases/latest/download/gik-installer.ps1 | iex
```

*Note: Replace `Kyryloloshka` with the actual GitHub username where the repository is hosted.*
