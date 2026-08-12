import Link from 'next/link';

// The landing page, held to specification section 23.1: one sentence of
// definition, one worked invocation with its output, the prerequisite named
// plainly, and links to getting started, the repository, and the glossary. No
// testimonials, no feature grid, no call to action.
export default function HomePage() {
  return (
    <main
      style={{
        maxWidth: '48rem',
        margin: '0 auto',
        padding: '4rem 1.5rem',
        display: 'flex',
        flexDirection: 'column',
        gap: '2rem',
      }}
    >
      <h1
        style={{
          fontFamily: 'var(--font-display)',
          fontSize: '2rem',
          fontWeight: 600,
          lineHeight: 1.2,
        }}
      >
        fragcap
      </h1>

      <p style={{ fontSize: '1.125rem', lineHeight: 1.6 }}>
        fragcap is a passive network capture tool for Windows that attributes
        each captured flow to the process that produced it, including game
        clients launched indirectly through platform and publisher launchers.
      </p>

      <figure style={{ margin: 0 }}>
        <pre
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '0.9rem',
            padding: '1rem 1.25rem',
            borderRadius: '0.5rem',
            overflowX: 'auto',
          }}
          className="fd-codeblock"
        >
          <code>{`$ fragcap run --profile eso --out capture.fcapng
armed: waiting for eso.exe
stage matched: eso.exe (pid 8124)
captured 4127 packets, 4127 attributed
wrote capture.fcapng`}</code>
        </pre>
      </figure>

      <p style={{ lineHeight: 1.6 }}>
        Capture requires the npcap driver, installed with WinPcap-compatible
        mode. fragcap detects it and never installs it.
      </p>

      <nav style={{ display: 'flex', gap: '1.5rem', flexWrap: 'wrap' }}>
        <Link href="/docs/getting-started">Get started</Link>
        <Link href="https://github.com/h8rt3rmin8r/fragcap">Repository</Link>
        <Link href="/docs/glossary">Glossary</Link>
      </nav>

      <footer
        style={{
          marginTop: '2rem',
          fontFamily: 'var(--font-mono)',
          fontSize: '0.75rem',
          textTransform: 'uppercase',
          letterSpacing: '0.05em',
          opacity: 0.6,
        }}
      >
        A ShruggieTech project
      </footer>
    </main>
  );
}
