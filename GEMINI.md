# Gik Project Instructions & Conventions

## Architecture
- **Layered Modularity**: Keep a clear separation between CLI, Business Logic (Commands), Core Objects (Git-standard), and Data Storage.
- **Domain-Driven Storage**: Encapsulate `redb` details within the `storage` module. Higher-level logic should not touch raw table definitions.
- **Service Pattern**: Use structs with methods for complex operations (e.g., `ObjectManager`, `CommitService`) to allow easier testing and state management.

## Coding Standards (Rust Best Practices)
- **Error Handling**: 
  - Use `thiserror` for library-level errors.
  - Return `Result<T>` from all fallible functions.
  - Avoid `unwrap()` and `expect()` in production code; use proper error propagation.
- **Modularity**:
  - Prefer small, focused modules. 
  - Use `pub(crate)` to hide internals from users but keep them accessible within the project.
- **Data Integrity**: 
  - Ensure all mutable operations are wrapped in `redb` ACID transactions.
  - Use a `TransactionContext` or similar pattern to pass transactions across logic steps.
- **Performance**:
  - Always use streaming IO (`Read`, `Write`) for object processing to maintain a low memory footprint.
  - Avoid cloning large buffers; use references or slices where possible.

## Naming Conventions
- **Structs/Enums**: CamelCase (e.g., `CommitMeta`).
- **Functions/Variables**: snake_case (e.g., `hash_blob`).
- **Traits**: Capable adjectives where possible (e.g., `ObjectReader`).

## Documentation
- Document public modules, structs, and functions with doc-comments (`///`).
- Include examples in doc-comments for complex logic.

## Testing
- **Unit Tests**: Place in the same file as the code under `#[cfg(test)]`.
- **Integration Tests**: Place in the `tests/` directory.
- **Test-Driven Development**: Always write failing tests for new features or bug fixes.
