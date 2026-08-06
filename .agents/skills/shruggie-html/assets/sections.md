# Section Primitives

Reusable HTML fragments for composing a ShruggieTech page. Each fragment assumes `assets/brand.css` is already inlined in the document head. Copy the fragment, swap the content, and place it inside `<main>` in the order the deliverable calls for.

All fragments use only the brand classes defined in `brand.css`. Do not introduce new utility classes; if a layout need is not covered here, extend `brand.css` rather than inlining ad-hoc styles in the page.

## Hero

Use once, at the top of the page. Sets the first-impression frame.

```html
<section class="section hero">
  <div class="container">
    <span class="eyebrow">CATEGORY OR CONTEXT</span>
    <h1>One sharp headline that speaks to the reader.</h1>
    <p class="lede">A single supporting sentence that names the value and the audience.</p>
    <p>
      <a href="#cta" class="button-primary">Primary action</a>
      <a href="#learn-more" class="button-secondary">Secondary action</a>
    </p>
  </div>
</section>
```

## Value Props Grid

Three or four cards. Each card is a single concise claim.

```html
<section class="section section-services">
  <div class="container">
    <span class="eyebrow">What you get</span>
    <h2>Three things we deliver.</h2>
    <div class="grid grid-3" style="margin-top: 2rem;">
      <article class="card">
        <h3>First value prop</h3>
        <p>One or two sentences. State the outcome, not the activity.</p>
      </article>
      <article class="card">
        <h3>Second value prop</h3>
        <p>One or two sentences. Concrete language. No consulting-speak.</p>
      </article>
      <article class="card">
        <h3>Third value prop</h3>
        <p>One or two sentences. End on a verifiable claim where possible.</p>
      </article>
    </div>
  </div>
</section>
```

## Two-Column Feature

Image or visual on one side, copy on the other. Stacks on narrow viewports automatically via the grid utility.

```html
<section class="section section-work">
  <div class="container">
    <div class="grid grid-2" style="align-items: center;">
      <div>
        <span class="eyebrow">Feature name</span>
        <h2>One headline. One promise.</h2>
        <p>Two or three sentences of explanation. Avoid hedging. State capability plainly.</p>
        <p><a href="#" class="button-secondary">Read more</a></p>
      </div>
      <div>
        <img src="{{IMAGE_URL}}" alt="{{IMAGE_ALT}}" style="border-radius: var(--radius-lg);">
      </div>
    </div>
  </div>
</section>
```

## Single-Column Article Body

For internal reports, write-ups, and prose-heavy deliverables. The `prose` container caps line length and applies comfortable rhythm to headings and lists. Avoid using `prose` inside marketing sections; it is for long-form text only.

```html
<section class="section">
  <div class="container" style="max-width: 760px;">
    <span class="eyebrow">REPORT</span>
    <h1>Report title</h1>
    <p class="lede">One-sentence summary the reader can scan in a second.</p>

    <h2>Section heading</h2>
    <p>Body paragraph. Plain English. Second person where appropriate.</p>
    <ul>
      <li>Bullet one.</li>
      <li>Bullet two.</li>
    </ul>

    <h2>Another section</h2>
    <p>More body copy.</p>
    <pre><code>// code samples use Geist Mono via the brand stylesheet
function example() { return true; }
</code></pre>
  </div>
</section>
```

## CTA Block

Closes the page. Pairs the primary CTA with the brand tagline beneath. The tagline is exact text; do not paraphrase. The shruggie emoticon is decorative and must carry `aria-hidden="true"` so screen readers do not stumble over the ASCII art.

```html
<section class="section section-cta" id="cta">
  <div class="container" style="text-align: center;">
    <h2>One clear ask.</h2>
    <p class="lede" style="margin-inline: auto;">A single sentence that names the next step.</p>
    <p style="margin-top: 2rem;">
      <a href="mailto:hello@shruggie.tech" class="button-primary">Get in touch</a>
    </p>
    <p class="tagline" style="margin-top: 1rem;">
      <span class="shruggie" aria-hidden="true">¯\_(ツ)_/¯</span>
      We'll figure it out.
    </p>
  </div>
</section>
```

## Footer Variant: Minimal

Default footer in `page-template.html` already carries the attribution and tagline. Use this denser variant if the deliverable needs contact details or supplementary links.

```html
<footer class="site-footer">
  <div class="container">
    <div>
      <strong>By ShruggieTech</strong><br>
      <span style="color: var(--text-muted); font-size: var(--font-size-body-xs);">
        Shruggie LLC, d/b/a ShruggieTech
      </span>
    </div>
    <nav aria-label="Footer" style="display: flex; gap: 1.25rem;">
      <a href="https://shruggie.tech">shruggie.tech</a>
      <a href="mailto:hello@shruggie.tech">hello@shruggie.tech</a>
    </nav>
    <div class="tagline">
      <span class="shruggie" aria-hidden="true">¯\_(ツ)_/¯</span>
      We'll figure it out.
    </div>
  </div>
</footer>
```

## Composition Notes

- Alternate section background tokens (`section-services`, `section-work`, `section-research`, `section-cta`) to create depth. Two adjacent sections should never share the same surface token unless a divider is intentional.
- The hero always sits flush against the site header; do not add a section surface class to it.
- The CTA section is the only one that should be center-aligned. Other sections read left-aligned by default.
- Cap one accent color per section. Mixing Green Bright and Orange in the same viewport pulls attention in two directions and undermines the CTA hierarchy.
