---
id: design-system
kind: design-system
---

# Design System

> **This is a template.** Tokens, component vocabulary, and parity rules belong here. The web app is the canonical realization; every other GUI platform adapts these tokens to its native conventions.
>
> Columns below cover the GUI platforms (web, Apple, Android, Windows, Linux). The **website** reuses the web realization. The **CLIs** (Ratatui TUI, Node CLI) have no design-system surface beyond terminal color and layout — they are exempt from these tables.

## Tokens

Tokens are abstract values that every platform realizes in its own idiom. The token name and intent are stable; the realization differs.

### Color

| Token                  | Intent             | Web (Tailwind v4)           | iOS                                 | Android                                      | Windows (WinUI)                       | Linux (Adwaita)    |
| ---------------------- | ------------------ | --------------------------- | ----------------------------------- | -------------------------------------------- | ------------------------------------- | ------------------ |
| `color.surface.base`   | Default surface    | `bg-surface`                | `UIColor.systemBackground`          | `MaterialTheme.colorScheme.surface`          | `ApplicationPageBackgroundThemeBrush` | `@window_bg_color` |
| `color.surface.raised` | Cards, sheets      | `bg-surface-raised`         | `UIColor.secondarySystemBackground` | `MaterialTheme.colorScheme.surfaceContainer` | `CardBackgroundFillColorDefaultBrush` | `@card_bg_color`   |
| `color.text.primary`   | Main copy          | `text-primary`              | `UIColor.label`                     | `MaterialTheme.colorScheme.onSurface`        | `TextFillColorPrimaryBrush`           | `@theme_fg_color`  |
| `color.text.secondary` | Subtler copy       | `text-secondary`            | `UIColor.secondaryLabel`            | `MaterialTheme.colorScheme.onSurfaceVariant` | `TextFillColorSecondaryBrush`         | `.dim-label`       |
| `color.accent`         | Brand accent       | `text-accent` / `bg-accent` | `UIColor.tintColor`                 | `MaterialTheme.colorScheme.primary`          | `AccentFillColorDefaultBrush`         | `@accent_color`    |
| `color.danger`         | Destructive intent | `text-danger` / `bg-danger` | `UIColor.systemRed`                 | `MaterialTheme.colorScheme.error`            | `SystemFillColorCriticalBrush`        | `@error_color`     |

<!-- Add the actual hex/HSL values once branding is settled. -->

### Typography

| Token          | Intent                     |
| -------------- | -------------------------- |
| `type.display` | Top of page, hero          |
| `type.heading` | Section headings           |
| `type.body`    | Paragraph copy             |
| `type.label`   | Form labels, list metadata |
| `type.caption` | Microcopy                  |

Each platform maps these to its native type scale (Tailwind utility classes on web, `UIFont.preferredFont(forTextStyle:)` on iOS, `MaterialTheme.typography.titleLarge` etc. on Android).

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

| Component    | Intent                          | Web                            | iOS                             | Android                     | Windows (WinUI) | Linux (GTK/Adwaita) |
| ------------ | ------------------------------- | ------------------------------ | ------------------------------- | --------------------------- | --------------- | ------------------- |
| `Button`     | Primary action affordance       | React Aria `Button`            | UIKit `UIButton`                | Compose `Button`            | `Button`        | `Gtk.Button`        |
| `TextField`  | Single-line text input          | React Aria `TextField`         | UIKit `UITextField`             | Compose `OutlinedTextField` | `TextBox`       | `Adw.EntryRow`      |
| `List`       | Vertically scrolling collection | Custom or React Aria `ListBox` | UIKit `UITableView`             | Compose `LazyColumn`        | `ListView`      | `Gtk.ListBox`       |
| `Sheet`      | Modal that slides from edge     | React Aria `Modal`             | `UISheetPresentationController` | Compose `ModalBottomSheet`  | `ContentDialog` | `Adw.Dialog`        |
| `Avatar`     | Person/identity glyph           | Custom                         | Custom                          | Custom                      | `PersonPicture` | `Adw.Avatar`        |
| `EmptyState` | Friendly empty placeholder      | Custom                         | Custom                          | Custom                      | Custom          | `Adw.StatusPage`    |

## Iconography

Use the same icon vocabulary across platforms — names map to Lucide on web, SF Symbols on Apple, Material Icons on Android, Segoe Fluent Icons on Windows, and named/symbolic icons on Linux.

| Intent | Web             | iOS                  | Android           | Windows (Segoe Fluent) | Linux (symbolic)         |
| ------ | --------------- | -------------------- | ----------------- | ---------------------- | ------------------------ |
| Add    | Lucide `Plus`   | SF `plus`            | Material `Add`    | `Add`                  | `list-add-symbolic`      |
| Edit   | Lucide `Pencil` | SF `pencil`          | Material `Edit`   | `Edit`                 | `document-edit-symbolic` |
| Delete | Lucide `Trash2` | SF `trash`           | Material `Delete` | `Delete`               | `user-trash-symbolic`    |
| Search | Lucide `Search` | SF `magnifyingglass` | Material `Search` | `Search`               | `system-search-symbolic` |

## Parity rules

- **Visual parity is not pixel parity.** A UIKit table view looks like a UIKit table view; a Compose list looks like a Compose list; a WinUI `ListView` and a GTK `ListBox` each look native. Don't fight platform conventions.
- **Tokens must agree.** The same color, spacing, type intent must produce the same _role_ on every platform.
- **Layout convergence at the screen level.** A given screen (e.g. an "items list") on any client contains the same primary surfaces (search bar, list, add affordance) as the web reference.
- **Native affordances are encouraged.** Pull-to-refresh, swipe actions, share sheets — use them. Mark them `// SPEC: manual` if no spec applies.

## Accessibility baseline

- All interactive elements must have a label and meet the platform's minimum touch/click target size (44pt iOS, 48dp Android, 44px web, 40px Windows, 24px+ pointer / accessible labels on Linux).
- All text meets WCAG AA contrast against its background.
- Focus order matches reading order on every platform. Expose accessibility metadata natively — XAML `AutomationProperties` on Windows, `Gtk.Accessible` roles on Linux, the platform's accessibility API everywhere else.

## Open design questions

<!-- Tag with the date so they can be revisited. -->

- _(empty)_
