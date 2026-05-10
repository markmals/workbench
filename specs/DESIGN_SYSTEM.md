---
id: design-system
kind: design-system
---

# Design System

> **This is a template.** Tokens, component vocabulary, and parity rules belong here. The web app is the canonical realization; iOS and Android adapt these tokens to their native conventions.

## Tokens

Tokens are abstract values that every platform realizes in its own idiom. The token name and intent are stable; the realization differs.

### Color

| Token                  | Intent             | Web (Tailwind v4)           | iOS                                 | Android                                      |
| ---------------------- | ------------------ | --------------------------- | ----------------------------------- | -------------------------------------------- |
| `color.surface.base`   | Default surface    | `bg-surface`                | `Color(.systemBackground)`          | `MaterialTheme.colorScheme.surface`          |
| `color.surface.raised` | Cards, sheets      | `bg-surface-raised`         | `Color(.secondarySystemBackground)` | `MaterialTheme.colorScheme.surfaceContainer` |
| `color.text.primary`   | Main copy          | `text-primary`              | `Color.primary`                     | `MaterialTheme.colorScheme.onSurface`        |
| `color.text.secondary` | Subtler copy       | `text-secondary`            | `Color.secondary`                   | `MaterialTheme.colorScheme.onSurfaceVariant` |
| `color.accent`         | Brand accent       | `text-accent` / `bg-accent` | _accent color_                      | `MaterialTheme.colorScheme.primary`          |
| `color.danger`         | Destructive intent | `text-danger` / `bg-danger` | `Color.red`                         | `MaterialTheme.colorScheme.error`            |

<!-- Add the actual hex/HSL values once branding is settled. -->

### Typography

| Token          | Intent                     |
| -------------- | -------------------------- |
| `type.display` | Top of page, hero          |
| `type.heading` | Section headings           |
| `type.body`    | Paragraph copy             |
| `type.label`   | Form labels, list metadata |
| `type.caption` | Microcopy                  |

Each platform maps these to its native type scale (Tailwind utility classes on web, `Font.title2` etc. on iOS, `MaterialTheme.typography.titleLarge` etc. on Android).

### Spacing

A 4-px base scale: `space.0` … `space.12` (= 0, 4, 8, 12, 16, 20, 24, 32, 40, 48, 56, 64).

### Radius

`radius.sm` (4px), `radius.md` (8px), `radius.lg` (16px), `radius.full` (pill).

### Motion

| Token           | Duration | Easing      |
| --------------- | -------- | ----------- |
| `motion.fast`   | 120 ms   | ease-out    |
| `motion.medium` | 240 ms   | ease-in-out |
| `motion.slow`   | 400 ms   | ease-out    |

## Component vocabulary

These are abstract components every platform must provide. The name and intent are stable; the realization is idiomatic.

| Component    | Intent                          | Web                            | iOS                 | Android                     |
| ------------ | ------------------------------- | ------------------------------ | ------------------- | --------------------------- |
| `Button`     | Primary action affordance       | React Aria `Button`            | SwiftUI `Button`    | Compose `Button`            |
| `TextField`  | Single-line text input          | React Aria `TextField`         | SwiftUI `TextField` | Compose `OutlinedTextField` |
| `List`       | Vertically scrolling collection | Custom or React Aria `ListBox` | SwiftUI `List`      | Compose `LazyColumn`        |
| `Sheet`      | Modal that slides from edge     | React Aria `Modal`             | SwiftUI `.sheet`    | Compose `ModalBottomSheet`  |
| `Avatar`     | Person/identity glyph           | Custom                         | Custom              | Custom                      |
| `EmptyState` | Friendly empty placeholder      | Custom                         | Custom              | Custom                      |

## Iconography

Use the same icon vocabulary across platforms — names map to SF Symbols on iOS, Material Icons on Android, and Lucide on web.

| Intent | Web             | iOS                  | Android           |
| ------ | --------------- | -------------------- | ----------------- |
| Add    | Lucide `Plus`   | SF `plus`            | Material `Add`    |
| Edit   | Lucide `Pencil` | SF `pencil`          | Material `Edit`   |
| Delete | Lucide `Trash2` | SF `trash`           | Material `Delete` |
| Search | Lucide `Search` | SF `magnifyingglass` | Material `Search` |

## Parity rules

- **Visual parity is not pixel parity.** A SwiftUI list looks like a SwiftUI list; a Compose list looks like a Compose list. Don't fight platform conventions.
- **Tokens must agree.** The same color, spacing, type intent must produce the same _role_ on every platform.
- **Layout convergence at the screen level.** A given screen (e.g. an "items list") on iOS contains the same primary surfaces (search bar, list, add affordance) as on web and Android.
- **Native affordances are encouraged.** Pull-to-refresh, swipe actions, share sheets — use them. Mark them `// SPEC: manual` if no spec applies.

## Accessibility baseline

- All interactive elements must have a label and meet the platform's minimum touch target size (44pt iOS, 48dp Android, 44px web).
- All text meets WCAG AA contrast against its background.
- Focus order matches reading order on every platform.

## Open design questions

<!-- Tag with the date so they can be revisited. -->

- _(empty)_
