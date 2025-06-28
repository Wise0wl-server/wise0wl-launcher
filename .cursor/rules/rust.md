# Rust & Tauri Rules

## Rust Guidelines
- Follow Rust naming conventions (snake_case for variables/functions, PascalCase for types)
- Use proper error handling with `Result<T, E>` types
- Implement `Display` and `Debug` traits where appropriate
- Use meaningful variable names that describe their purpose
- Keep functions small and focused on a single responsibility

## Tauri Best Practices
- Follow Tauri security model and capabilities
- Use proper command invocations between frontend and backend
- Validate all inputs from the frontend
- Use proper error handling for Tauri commands
- Keep backend logic separate from frontend concerns

## Error Handling
- Use `Result` types for functions that can fail
- Provide meaningful error messages
- Use `?` operator for propagating errors
- Implement custom error types when needed
- Handle errors gracefully in Tauri commands

## Code Organization
- Group related functionality in modules
- Use proper visibility modifiers (`pub`, `pub(crate)`, etc.)
- Keep main.rs focused on application setup
- Extract business logic into separate modules

## Performance
- Use efficient data structures
- Avoid unnecessary allocations
- Use references when appropriate
- Profile code for performance bottlenecks

## Documentation
- Add documentation comments for public functions
- Use `///` for doc comments
- Include examples in documentation
- Document complex algorithms and business logic 