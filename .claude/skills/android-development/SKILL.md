---
name: android-development
description: Use when writing or modifying Android app code under `apps/android/`. Covers Jetpack Compose + Material 3 + Kotlin coroutines/Flow idioms, and points at Google's and JetBrains' first-party docs. Complementary to `implementing-a-spec` (process) and `android-emulator-control` (emulator driving).
---

# Android Development

This skill covers **how to write Android code** in this repo. For the _workflow_ of implementing a spec, see `implementing-a-spec`. For _driving the emulator_ (boot, install, screenshot, tap), see `android-emulator-control`. For _what to build_, see the spec.

## Stack at a glance

| Concern             | Choice                                                          | First-party docs                                                                                                                               |
| ------------------- | --------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Language            | Kotlin (latest stable)                                          | [kotlinlang.org/llms.txt](https://kotlinlang.org/llms.txt)                                                                                     |
| UI                  | Jetpack Compose                                                 | [developer.android.com/jetpack/compose](https://developer.android.com/jetpack/compose)                                                         |
| Design language     | Material 3                                                      | [m3.material.io](https://m3.material.io/)                                                                                                      |
| Compose Material 3  | androidx.compose.material3                                      | [developer.android.com/jetpack/androidx/releases/compose-material3](https://developer.android.com/jetpack/androidx/releases/compose-material3) |
| View models         | `androidx.lifecycle.ViewModel`                                  | [developer.android.com/topic/libraries/architecture/viewmodel](https://developer.android.com/topic/libraries/architecture/viewmodel)           |
| Reactive primitives | `kotlinx.coroutines.flow.*` (`StateFlow`, `SharedFlow`, `Flow`) | [kotlinlang.org/docs/flow](https://kotlinlang.org/docs/flow.html)                                                                              |
| Concurrency         | Kotlin coroutines                                               | [kotlinlang.org/docs/coroutines-overview](https://kotlinlang.org/docs/coroutines-overview.html)                                                |
| Tests               | `kotlin.test` (JUnit5 backend)                                  | [kotlinlang.org/api/latest/kotlin.test](https://kotlinlang.org/api/latest/kotlin.test/)                                                        |
| Linter / formatter  | ktlint                                                          | [pinterest.github.io/ktlint](https://pinterest.github.io/ktlint/)                                                                              |
| Build system        | Gradle (Kotlin DSL)                                             | [docs.gradle.org/current/userguide/kotlin_dsl.html](https://docs.gradle.org/current/userguide/kotlin_dsl.html)                                 |
| Convex client       | Convex's official Kotlin client (do **not** hand-roll a transport over HTTP/WebSocket) | (see `services/convex/CLAUDE.md`)                                                                                       |

Kotlin publishes `/llms.txt`; Google and Android do not. Use WebFetch against canonical URLs when looking things up.

## Idioms (read these before writing code)

### `ViewModel` + `StateFlow`

```kt
// SPEC: vm.items.list
class ItemsListViewModel(
    private val client: ConvexClient,
) : ViewModel() {

    private val _state = MutableStateFlow<UiState>(UiState.Idle)
    val state: StateFlow<UiState> = _state.asStateFlow()

    fun load() {
        viewModelScope.launch {
            _state.value = UiState.Loading
            _state.value = runCatching { client.list() }
                .fold(
                    onSuccess = { UiState.Loaded(it) },
                    onFailure = { UiState.Error(it.message ?: "Unknown error") },
                )
        }
    }

    sealed interface UiState {
        data object Idle : UiState
        data object Loading : UiState
        data class Loaded(val items: List<Item>) : UiState
        data class Error(val message: String) : UiState
    }
}
```

- UI state is `StateFlow<UiState>`.
- Sealed interface for state variants — exhaustive `when` in the composable.
- User actions are public suspend (or non-suspend that launches inside `viewModelScope`) functions.
- One-shot events (snackbars, navigation) use a `Channel` exposed as a `Flow`.

### No LiveData, no RxJava

Coroutines and Flow only. The exception is interoperating with an Android framework API that returns LiveData — convert immediately with `.asFlow()` and don't let LiveData leak further.

### Compose state from `StateFlow`

```kt
@Composable
fun ItemsListScreen(viewModel: ItemsListViewModel) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    LaunchedEffect(Unit) { viewModel.load() }

    when (val current = state) {
        UiState.Idle, UiState.Loading -> LoadingIndicator()
        is UiState.Loaded -> ItemsList(current.items)
        is UiState.Error -> ErrorState(current.message, onRetry = viewModel::load)
    }
}
```

- `collectAsStateWithLifecycle()` (not `collectAsState()`) — it's lifecycle-aware on Android.
- Composables are dumb. They render state and forward user actions; no business logic in the composable body.
- Reverse pointer lives on the **view model**, not the composable.

### Tests at the view-model layer

```kt
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Tag

@Tag("spec:vm.items.list")
class ItemsListViewModelTest {

    @Test
    @DisplayName("[scenario.items.list.empty] shows empty state when no items exist")
    fun emptyState() = runTest {
        val client = MockConvexClient(returning = emptyList())
        val vm = ItemsListViewModel(client)

        vm.load()
        advanceUntilIdle()

        assertEquals(UiState.Loaded(emptyList()), vm.state.value)
    }
}
```

- `@Tag("spec:<id>")` carries the spec ID.
- `@DisplayName("[scenario.<id>] ...")` carries the scenario sub-ID.
- Use `runTest` from `kotlinx-coroutines-test` — never `runBlocking` in tests.

### Material 3 affordances

Use Material 3 components by default. Don't reach for AndroidView or compose alternatives unless Material 3 genuinely can't express what the spec requires.

Common Material 3 references:

- [Components](https://m3.material.io/components)
- [Styles (color, typography, shape)](https://m3.material.io/styles)
- [Foundations (motion, layout)](https://m3.material.io/foundations)

Pull-to-refresh, swipe-to-dismiss, FABs, snackbars — all Material idioms. Mark `// SPEC: manual` if no cross-platform spec applies.

### Dependency injection

Hilt or manual constructor injection. Pick one project-wide and document the choice in `apps/android/CLAUDE.md`. **No service locator pattern.**

## File layout (within apps/android/)

See `apps/android/CLAUDE.md` for the canonical layout. Summary:

```
apps/android/app/src/main/kotlin/com/sdd/app/
├── app/                           ← Application class, MainActivity, root nav
├── feature/<slug>/                ← Feature-scoped: ViewModel, Composable(s)
├── domain/                        ← Data classes and validation
└── client/                        ← Convex client wrapper
```

## When to invoke a more specific skill

- About to write tests? → `test-driven-development`
- About to claim work is done? → `verification-before-completion`
- Booting the emulator, screenshotting, tapping? → `android-emulator-control`
- Debugging something unexpected? → `systematic-debugging`
- Implementing a spec end-to-end? → `implementing-a-spec`

## Commit

Land focused, atomic commits as the work hits natural boundaries — typically per spec ID, per ViewModel + its tests, or per cohesive refactor. See `.claude/rules/commit-discipline.md`.

Android-specific notes:

- **Gradle changes go alone.** Edits to `build.gradle.kts`, `settings.gradle.kts`, or `libs.versions.toml` belong in their own commit (`chore: bump <dependency>` or `chore: add <module>`). Don't bundle with feature code.
- **Don't commit generated build outputs.** `apps/android/**/build/` is gitignored.
- **Resource additions are separate.** New drawables, strings, or theme entries belong in their own commit so the diff is reviewable.
