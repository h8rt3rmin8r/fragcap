// SPDX-License-Identifier: Apache-2.0
import Link from 'next/link';
import { fragcapVersion } from '@/lib/version.generated';
import { Mermaid } from '@/components/mermaid';

// The dependency model, promoted from the Architecture page (slice S057): npcap
// required, Wireshark recommended, the extcap integration optional. It answers
// "what is npcap and why do I need it" where a first-time visitor meets it.
const DEPENDENCY_MODEL = `flowchart TD
  fragcap["fragcap: capture and attribution"]
  npcap["npcap: capture driver (required)"]
  ws["Wireshark: analyzer (recommended)"]
  extcap["Wireshark extcap: optional"]
  fragcap -->|captures through| npcap
  fragcap -->|writes captures opened in| ws
  ws -->|installer bundles| npcap
  extcap -->|ships with| ws
  fragcap -.->|registered by fragcap extcap install| extcap`;

// The landing page, held to specification section 23.1 (as amended, slice S057):
// it opens with the problem fragcap solves that standard tooling does not, stated
// plainly enough that a technically competent visitor who has never thought about
// attribution understands the gap within one screen; it states what fragcap is;
// it shows the tool working with one real command and its real output (the
// `fragcap targets` hero listing) as the primary persuasive asset; it names the
// prerequisites; it carries the dependency diagram and a small number of concrete
// capability statements, each linking to the docs that prove it; and it directs
// the visitor to getting started with a single primary action. It carries no
// testimonials, feature grid, badges, pricing, or sponsorship solicitation; the
// capability statements are plain facts with links, not marketing. Voice per
// section 23.3: precise, dry, no hype.
export default function HomePage() {
  return (
    <main
      className="fc-page"
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
        <a
          href="https://github.com/h8rt3rmin8r/fragcap/releases"
          className="fc-version"
        >
          v{fragcapVersion}
        </a>
      </h1>

      <p style={{ fontSize: '1.375rem', lineHeight: 1.4, fontWeight: 600 }}>
        Your capture recorded 40,000 packets. It cannot tell you which one your
        game sent.
      </p>

      <p style={{ fontSize: '1.125rem', lineHeight: 1.6 }}>
        Capture happens at the network driver, below the socket layer, where the
        operating system has already thrown away the link between a packet and the
        process that produced it. For a game client started by a platform launcher
        that starts a publisher launcher, the process you care about is three hops
        from the thing you launched. fragcap reconstructs that link and writes it
        into the capture file.
      </p>

      <p style={{ lineHeight: 1.6 }}>
        fragcap is a passive network capture tool for Windows that attributes each
        captured flow to the process that produced it, including game clients
        launched indirectly through platform and publisher launchers, and writes
        the result as an extended pcapng file that unmodified analyzers still read
        as ordinary pcapng.
      </p>

      <aside
        className="fc-callout"
        style={{
          borderLeft: '3px solid var(--color-fd-primary)',
          padding: '0.75rem 1rem',
          lineHeight: 1.6,
        }}
      >
        Two prerequisites, up front. Live capture needs the{' '}
        <a href="https://npcap.com/">npcap</a> driver, installed in
        WinPcap-compatible mode; fragcap detects it and never bundles or hosts it.
        To read a capture, open the resulting file in{' '}
        <a href="https://www.wireshark.org/">Wireshark</a> or any pcapng-aware
        analyzer: capture with fragcap, then inspect the result in Wireshark.
      </aside>

      <figure style={{ margin: 0 }}>
        <figcaption
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '0.8rem',
            textTransform: 'uppercase',
            letterSpacing: '0.06em',
            opacity: 0.6,
            marginBottom: '0.5rem',
          }}
        >
          fragcap targets
        </figcaption>
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
          <code>{`$ fragcap targets
  #  TARGET                     CAPTURE          KNOWN
  1  the_elder_scrolls_online   ready            no online mode recorded
  2  the_division_2             ready            Denuvo, EasyAntiCheat

  fragcap capture 1`}</code>
        </pre>
        <figcaption style={{ marginTop: '0.5rem', fontSize: '0.9rem', opacity: 0.8 }}>
          fragcap discovers the capturable titles on your machine and ends by
          naming the next command; the row number is what{' '}
          <code style={{ fontFamily: 'var(--font-mono)' }}>fragcap capture</code>{' '}
          honors.
        </figcaption>
      </figure>

      <figure style={{ margin: 0 }}>
        <Mermaid chart={DEPENDENCY_MODEL} />
        <figcaption style={{ marginTop: '0.5rem', fontSize: '0.9rem', opacity: 0.8 }}>
          The dependency model: npcap is required, Wireshark recommended (its
          installer also provides npcap), and the extcap integration optional.
        </figcaption>
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
          fragcap discovers the installed titles on your machine and registers
          them as capture targets, so a client started indirectly through Steam or
          another launcher is matched without hunting for it.{' '}
          <Link href="/docs/getting-started">Getting started</Link>
        </li>
        <li>
          fragcap observes only. It never modifies, injects, or replays traffic,
          and never reads the memory of another process.{' '}
          <Link href="/docs/glossary/anti-cheat-and-security">
            Security posture
          </Link>
        </li>
      </ul>

      <nav style={{ display: 'flex', gap: '1.5rem', flexWrap: 'wrap' }}>
        <Link href="/docs/getting-started">Get started</Link>
        <Link href="https://github.com/h8rt3rmin8r/fragcap">Repository</Link>
        <Link href="/docs/glossary">Glossary</Link>
        <Link href="/docs/changelog">Changelog</Link>
      </nav>
    </main>
  );
}
