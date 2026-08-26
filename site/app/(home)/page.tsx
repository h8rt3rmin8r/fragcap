// SPDX-License-Identifier: Apache-2.0
import Link from 'next/link';
import { fragcapVersion } from '@/lib/version.generated';
import { Mermaid } from '@/components/mermaid';

// The dependency model, promoted from the Architecture page (slice S057): npcap
// required for live packet capture, Wireshark recommended, and the extcap
// integration optional. It answers "what is npcap and why do I need it" where a
// first-time visitor meets it.
const DEPENDENCY_MODEL = `flowchart TD
  fragcap["fragcap: Capture and Deep Capture"]
  npcap["npcap: capture driver (required)"]
  ws["Wireshark: analyzer (recommended)"]
  extcap["Wireshark extcap: optional"]
  fragcap -->|captures through| npcap
  fragcap -->|writes captures opened in| ws
  ws -->|installer bundles| npcap
  extcap -->|ships with| ws
  fragcap -.->|registered by fragcap extcap install| extcap`;

// The landing page, held to specification section 23.1 (corrected by S078): it
// leads with the result, explains flow attribution as correlation across separate
// observations, distinguishes passive Capture from explicit scoped Deep Capture,
// and shows synthetic current CLI output as its primary evidence. It names the
// live-capture dependency, recommends an analyzer, keeps one primary action, and
// retains section 23.3's precise instrument voice without universal claims.
export default function HomePage() {
  return (
    <main
      className="fc-page"
      style={{
        width: '100%',
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
        Game traffic, attributed to the process responsible for it.
      </p>

      <p style={{ fontSize: '1.125rem', lineHeight: 1.6 }}>
        Packet captures preserve network frames, not process ownership. fragcap
        correlates captured flows with Windows socket and process-lifecycle
        observations, including clients started through platform and publisher
        launchers. Resolved flows carry their observed attribution and fidelity;
        unresolved traffic remains visible rather than being discarded.
      </p>

      <p style={{ lineHeight: 1.6 }}>
        <strong>Capture</strong> passively records packets and process attribution.
        {' '}<strong>Deep Capture</strong> runs that same capture alongside an
        explicit, target-scoped local proxy for compatible targets, adding
        correlated application records and optional proxy-owned TLS key logs when
        the proxy can inspect the traffic.
      </p>

      <aside
        className="fc-callout"
        style={{
          borderLeft: '3px solid var(--color-fd-primary)',
          padding: '0.75rem 1rem',
          lineHeight: 1.6,
        }}
      >
        Live packet capture requires the <a href="https://npcap.com/">Npcap</a>{' '}
        driver in WinPcap-compatible mode; fragcap never bundles, hosts, embeds,
        or redistributes it. <a href="https://www.wireshark.org/">Wireshark</a>{' '}
        is recommended for live or post-session analysis, but any pcapng-aware
        analyzer can read Capture output. Run <code>fragcap doctor</code> for
        Capture and Deep Capture readiness.
      </aside>

      <figure style={{ margin: 0, minWidth: 0 }}>
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
            maxWidth: '100%',
            overflowX: 'auto',
          }}
          className="fd-codeblock"
        >
          <code>{`$ fragcap targets
  #  TARGET            CAPTURE         ENGINE         SENSITIVITIES
  1  sample_adventure  ready           Sample Engine  not scanned
  2  sample_arena      needs a target  not scanned    Sample Protection

Next command:  fragcap capture 1`}</code>
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
          Resolved flows carry process attribution and fidelity in packet comments;
          unresolved flows remain in the capture. An unmodified analyzer still
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
          Capture observes passively. Deep Capture runs only when explicitly
          selected and uses a session-scoped local proxy; neither mode injects
          code or reads target-process memory.{' '}
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
