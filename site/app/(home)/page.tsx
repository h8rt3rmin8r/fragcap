// SPDX-License-Identifier: Apache-2.0
import Link from 'next/link';

// The landing page, held to specification section 23.1 (as amended for issue
// #42): it leads with the problem fragcap solves that standard tooling does not,
// states what fragcap is, shows one worked invocation with its output, names a
// small number of concrete capabilities each linking into the docs that prove
// them, and names the prerequisite plainly. It still carries no testimonials, no
// feature grid, and no call to action; the capability statements are plain facts
// with links, not marketing. Voice per section 23.3: precise, dry, no hype.
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
          display: 'flex',
          alignItems: 'center',
          gap: '1rem',
          margin: 0,
        }}
      >
        <img
          src="/logos/fragcap-mark-color.svg"
          alt=""
          style={{ height: '3rem', width: 'auto' }}
        />
        <img
          src="/logos/fragcap-wordmark-white.svg"
          alt="fragcap"
          className="fc-wordmark-dark"
          style={{ height: '2rem', width: 'auto' }}
        />
        <img
          src="/logos/fragcap-wordmark-cyan.svg"
          alt=""
          className="fc-wordmark-light"
          style={{ height: '2rem', width: 'auto' }}
        />
      </h1>

      <p style={{ fontSize: '1.125rem', lineHeight: 1.6 }}>
        Standard capture happens at the network driver, below the socket layer,
        where the association between a packet and the process that produced it
        has already been discarded. For a game client started indirectly through
        a platform or publisher launcher, the process that owns the traffic is
        not the launcher that started it, so ordinary capture cannot say which
        process a flow belongs to.
      </p>

      <p style={{ lineHeight: 1.6 }}>
        fragcap is a passive network capture tool for Windows that reconstructs
        that association, attributing each captured flow to the process that
        produced it, including game clients launched indirectly through platform
        and publisher launchers.
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

      <ul
        style={{
          margin: 0,
          paddingLeft: '1.25rem',
          display: 'flex',
          flexDirection: 'column',
          gap: '0.75rem',
          lineHeight: 1.6,
        }}
      >
        <li>
          Each flow is attributed to the process that owns it, and the
          attribution rides in packet comments, so an unmodified analyzer still
          reads the file as ordinary pcapng.{' '}
          <Link href="/docs/reference/output-formats">Output formats</Link>
        </li>
        <li>
          Clients started indirectly through Steam and other launchers are
          matched by a profile rather than by the launcher that spawned them.{' '}
          <Link href="/docs/guides/writing-a-profile">Writing a profile</Link>
        </li>
        <li>
          fragcap observes only. It never modifies, injects, or replays traffic,
          and never reads the memory of another process.{' '}
          <Link href="/docs/glossary/anti-cheat-and-security">
            Security posture
          </Link>
        </li>
      </ul>

      <p style={{ lineHeight: 1.6 }}>
        Capture requires the npcap driver, installed with WinPcap-compatible
        mode. fragcap detects it and never installs it.
      </p>

      <nav style={{ display: 'flex', gap: '1.5rem', flexWrap: 'wrap' }}>
        <Link href="/docs/getting-started">Get started</Link>
        <Link href="https://github.com/h8rt3rmin8r/fragcap">Repository</Link>
        <Link href="/docs/glossary">Glossary</Link>
      </nav>
    </main>
  );
}
