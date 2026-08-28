# Contract: Production Accessibility Surface

The S095 contract binds the public site structure and the browser regression that enforces it.

## Primary content and bypass

Every public route MUST expose exactly:

```text
skip href: #main-content
skip text: Skip to main content
primary id: main-content
primary landmarks: 1
primary programmatic focus: allowed
```

The skip link MUST precede persistent navigation in sequential focus order, remain visually off-canvas while idle, become fully visible and contained by the viewport when focused, and transfer fragment navigation and focus to `main-content` when activated.

The home layout's existing primary element owns the destination. Home page bodies MUST NOT introduce a nested primary element. The documentation page's route-specific article owns the primary role; documentation navigation and sidebars remain outside it.

## Generated heading hierarchy

The generated page title is level one. For visible headings that follow it in document order:

```text
current level <= previous level + 1
```

Heading text, order, links, and generated anchor identities remain unchanged. The rule applies to every generated changelog page, including input whose first source heading begins at level three, four, or deeper.

## Light-theme contrast

| Subject | Corrected foreground | Required backgrounds | Minimum ratio |
| --- | --- | --- | --- |
| Shared muted normal text | `#6e6e6e` | `#f1f1f1`, `#f5f5f5`, and every shipped use | 4.5:1 |
| Affected red syntax normal text | `#cc3346` | `#f1f1f1` and every shipped use | 4.5:1 |

The browser gate measures computed colors and nearest opaque backgrounds. Each selector population MUST be nonempty.

## Architecture diagram names

The two hydrated SVGs on `/docs/architecture` MUST preserve Mermaid's graphics-document semantics and expose these distinct names in document order:

1. `Capture packet attribution architecture`
2. `Deep Capture session architecture`

The names come from Mermaid-native titles. The diagram source, visual layout, node labels, and surrounding prose remain unchanged.

## Regression result

One failing route, viewport, contrast pair, heading transition, focus transfer, browser error, console error, or diagram observation fails the production accessibility test. Missing expected populations also fail. The gate reports the smallest route and subject needed to reproduce the failure.
