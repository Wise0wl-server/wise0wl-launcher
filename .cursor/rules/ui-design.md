# UI Design & Material UI Rules

## Material UI Implementation
- Use [Material UI](https://mui.com/material-ui/) as the primary component library
- Install required dependencies: `@mui/material @emotion/react @emotion/styled`
- Follow Material Design principles and guidelines
- Use MUI's comprehensive component library for consistent UI

## Design System

### Color Palette
- Use MUI's default color palette or create custom theme colors
- Implement proper contrast ratios for accessibility
- Use semantic colors (primary, secondary, error, warning, info, success)
- Support both light and dark themes

### Typography
- Use MUI's typography system with appropriate variants
- Maintain consistent font hierarchy (h1-h6, body1, body2, etc.)
- Use proper font weights and sizes for different contexts
- Ensure readable text with adequate contrast

### Layout & Spacing
- Use MUI's spacing system (theme.spacing) for consistent margins and padding
- Implement responsive design using MUI's breakpoint system
- Use `Box`, `Stack`, and `Grid` components for layout
- Follow Material Design spacing guidelines

## Component Guidelines

### Form Components
- Use `TextField` for all input fields with proper validation
- Implement `FormControl` and `FormHelperText` for form validation
- Use `Checkbox`, `Radio`, and `Switch` for selection controls
- Use `Select` and `MenuItem` for dropdown selections

### Navigation
- Use `AppBar` and `Toolbar` for main navigation
- Implement `Drawer` for sidebar navigation
- Use `Breadcrumbs` for hierarchical navigation
- Use `Tabs` for tabbed interfaces

### Data Display
- Use `Table` and `TableRow` for data tables
- Implement `Card` and `CardContent` for content containers
- Use `List` and `ListItem` for list displays
- Use `Chip` for tags and labels

### Feedback & Notifications
- Use `Snackbar` for temporary notifications
- Implement `Alert` for important messages
- Use `Dialog` and `Modal` for confirmations and forms
- Use `CircularProgress` and `LinearProgress` for loading states

## Responsive Design
- Use MUI's responsive breakpoints (xs, sm, md, lg, xl)
- Implement mobile-first design approach
- Use responsive utilities for different screen sizes
- Test on various device sizes and orientations

## Accessibility
- Use MUI's built-in accessibility features
- Implement proper ARIA labels and descriptions
- Ensure keyboard navigation works correctly
- Maintain proper color contrast ratios
- Use semantic HTML elements when possible

## Performance
- Use MUI's optimized components
- Implement lazy loading for large lists
- Use virtualization for long scrollable content
- Optimize bundle size by importing only needed components

## Customization
- Create custom themes using MUI's theming system
- Use `sx` prop for component-specific styling
- Implement custom component variants when needed
- Maintain consistency across custom components 