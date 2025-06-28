# TypeScript & React Rules

## TypeScript Guidelines
- Always use strict TypeScript configuration
- Define proper interfaces and types for all data structures
- Use `interface` for object shapes, `type` for unions and complex types
- Avoid `any` type - use `unknown` or proper typing instead
- Use generic types when appropriate

## React Best Practices
- Use functional components with hooks
- Prefer `useState` and `useEffect` over class components
- Use proper dependency arrays in `useEffect`
- Implement proper cleanup in `useEffect` when needed
- Use React.memo for expensive components
- Prefer controlled components over uncontrolled

## Material UI (MUI) Guidelines
- Use Material UI components as the primary UI library
- Follow Material Design principles and patterns
- Use MUI's theming system for consistent styling
- Leverage MUI's responsive design utilities
- Use `sx` prop for component-specific styling
- Prefer MUI components over custom HTML elements when possible
- Use MUI's spacing system (theme.spacing) for consistent layouts
- Implement proper accessibility with MUI's built-in ARIA support

### MUI Component Usage
- Use `Box`, `Stack`, and `Grid` for layout components
- Use `Typography` for all text elements with appropriate variants
- Use `Button` with proper variants (contained, outlined, text)
- Use `Card` and `CardContent` for content containers
- Use `TextField` for form inputs with proper validation
- Use `Dialog` and `Modal` for overlays and popups
- Use `Snackbar` and `Alert` for notifications and feedback
- Use `CircularProgress` and `LinearProgress` for loading states

### MUI Theming
- Create a custom theme that matches the application's design
- Use theme tokens for colors, spacing, and typography
- Implement dark/light mode support using MUI's theme system
- Use `ThemeProvider` to wrap the application
- Customize component default props through theme overrides

## State Management
- Use local state for component-specific data
- Consider context for shared state across components
- Keep state as close to where it's used as possible
- Use immutable state updates

## Component Structure
- One component per file
- Export components as default exports
- Use PascalCase for component names
- Keep components focused and single-purpose
- Extract reusable logic into custom hooks

## Error Handling
- Use proper error boundaries
- Handle async operations with try-catch
- Provide meaningful error messages
- Use TypeScript to prevent runtime errors 