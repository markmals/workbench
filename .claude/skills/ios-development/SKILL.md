---
name: ios-development
description: Use when writing or modifying iOS app code under `apps/ios/`. Covers SwiftUI + `@Observable` + Swift Testing idioms and points at Apple's first-party docs. Complementary to `implementing-a-spec` (process) and `ios-simulator-control` (simulator driving).
---

# iOS Development

This skill covers **how to write iOS code** in this repo. For the _workflow_ of implementing a spec, see `implementing-a-spec`. For _driving the simulator_ (boot, install, screenshot, tap), see `ios-simulator-control`. For _what to build_, see the spec.

## Stack at a glance

| Concern            | Choice                                               | First-party docs                                                                                                          |
| ------------------ | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| Language           | Swift (latest stable)                                | [docs.swift.org/swift-book](https://docs.swift.org/swift-book/)                                                           |
| UI                 | SwiftUI                                              | [developer.apple.com/documentation/swiftui](https://developer.apple.com/documentation/swiftui)                            |
| Reactive state     | `@Observable` macro (Observation framework)          | [developer.apple.com/documentation/observation](https://developer.apple.com/documentation/observation)                    |
| Concurrency        | Swift Concurrency (`async`/`await`, `Task`, `Actor`) | [docs.swift.org concurrency](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/concurrency/) |
| Tests              | Swift Testing                                        | [developer.apple.com/documentation/testing](https://developer.apple.com/documentation/testing)                            |
| Design language    | Apple Human Interface Guidelines                     | [developer.apple.com/design/human-interface-guidelines](https://developer.apple.com/design/human-interface-guidelines)    |
| Package manager    | Swift Package Manager                                | [swift.org/package-manager](https://www.swift.org/package-manager/)                                                       |
| Linter / formatter | swift-format                                         | [github.com/swiftlang/swift-format](https://github.com/swiftlang/swift-format)                                            |
| Project generation | Tuist (Swift manifests, no checked-in `.xcodeproj`)  | [docs.tuist.dev](https://docs.tuist.dev)                                                                                  |
| Convex client      | Convex's official Swift client (do **not** hand-roll a transport over HTTP/WebSocket) | (see `services/convex/CLAUDE.md`)                                                                        |

Apple doesn't publish a `/llms.txt` for any of these. Use WebFetch against the canonical doc URLs above when you need to look something up.

## Idioms (read these before writing code)

### `@Observable` for view models, not Combine

```swift
// SPEC: vm.items.list
import Observation

@Observable
@MainActor
final class ItemsListViewModel {
    private(set) var items: [Item] = []
    private(set) var status: Status = .idle

    enum Status: Equatable {
        case idle
        case loading
        case loaded
        case error(String)
    }

    private let client: ConvexClient

    init(client: ConvexClient) {
        self.client = client
    }

    func load() async {
        status = .loading
        do {
            items = try await client.list()
            status = .loaded
        } catch {
            status = .error(error.localizedDescription)
        }
    }
}
```

- `@Observable` macro generates the observation plumbing — no manual `@Published` or `ObservableObject`.
- View models are **reference types** (`final class`), `@MainActor`-pinned so UI reads are safe by default.
- Use Combine **only** when interoperating with an existing Combine API. Reach for it last.

### Async/await for everything I/O

No completion handlers, no `DispatchQueue.main.async`. If you need to bridge a completion-handler API, wrap it once with `withCheckedContinuation` and forget about it.

### Dependencies via initializer or `@Environment`

No singletons. The Convex client and other services are passed via initializer or SwiftUI's `Environment`. This is what makes testing tractable — a view model under test gets a mock client.

### Tests at the view-model layer

```swift
import Testing

@Suite("vm.items.list")
struct ItemsListViewModelTests {

    @Test("[scenario.items.list.empty] shows empty state when no items exist")
    func emptyState() async {
        let client = MockConvexClient(returning: [])
        let vm = ItemsListViewModel(client: client)

        await vm.load()

        #expect(vm.items.isEmpty)
        #expect(vm.status == .loaded)
    }
}
```

- `@Suite` name = the spec ID.
- `@Test` display name starts with `[scenario.<id>]`. Drift tooling reads this prefix.
- Use `#expect` and `#require` (Swift Testing macros) instead of XCTest's `XCTAssert*`.

### SwiftUI views are dumb

A SwiftUI view consumes a view model and renders. No business logic in `body`.

```swift
struct ItemsListView: View {
    @State private var viewModel: ItemsListViewModel

    var body: some View {
        Group {
            switch viewModel.status {
            case .idle, .loading:
                ProgressView()
            case .loaded:
                List(viewModel.items) { item in
                    ItemRow(item: item)
                }
            case .error(let message):
                ItemsErrorState(message: message, onRetry: { Task { await viewModel.load() } })
            }
        }
        .task { await viewModel.load() }
    }
}
```

The reverse pointer lives on the **view model**, not the view. Views carry `// SPEC: manual` (no cross-platform behavioral contract) or `// SPEC: <vm-id> (deviates: <ui reason>)` if there's a UI-level deviation.

### HIG-driven affordances

When in doubt about a design choice, read the relevant HIG section. Common ones:

- [Navigation](https://developer.apple.com/design/human-interface-guidelines/navigation-and-search)
- [Selection and input](https://developer.apple.com/design/human-interface-guidelines/inputs)
- [Lists and tables](https://developer.apple.com/design/human-interface-guidelines/lists-and-tables)
- [Modality](https://developer.apple.com/design/human-interface-guidelines/modality)

Native iOS affordances (pull-to-refresh, swipe actions, share sheets, system back gestures) are encouraged. Mark them `// SPEC: manual` if no cross-platform spec applies.

## File layout (within apps/ios/)

See `apps/ios/CLAUDE.md` for the canonical layout. Summary:

```
apps/ios/App/
├── App/                            ← App entry, root view, root container
├── Features/<Feature>/             ← Feature-scoped: VM, VM tests, View(s)
├── Domain/                         ← Plain types and validation (one file per domain.* spec)
├── Client/                         ← Convex client wrapper
└── Resources/                      ← Assets, localizable strings, Info.plist
```

## When to invoke a more specific skill

- About to write tests? → `test-driven-development`
- About to claim work is done? → `verification-before-completion`
- Booting the simulator, screenshotting, tapping? → `ios-simulator-control`
- Debugging something unexpected? → `systematic-debugging`
- Implementing a spec end-to-end? → `implementing-a-spec`
