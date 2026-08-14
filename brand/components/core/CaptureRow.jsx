// fragcap's signature component: one observed packet in a trace.
//
// `highlight` marks one endpoint as the active observation path, the way
// ScheduleRow tints one cron field in the sibling go-schedule system.
// `state` is rendered as a Badge, never as a bare color.
import { Badge } from "./Badge.jsx";

const TONE = { captured: "signal", inspect: "capture", failed: "fault" };
const LABEL = { captured: "Captured", inspect: "Inspect", failed: "Failed" };

export function CaptureRow({
  direction = "OUT",
  src,
  dst,
  process,
  bytes,
  state = "captured",
  highlight,          // "src" | "dst" | undefined
  selected = false,
}) {
  const endpoint = (value, key) =>
    highlight === key ? <b>{value}</b> : value;

  return (
    <div className="fc-capture-row" data-selected={selected}>
      <span className="fc-capture-row__dir">{direction}</span>
      <span className="fc-capture-row__flow">
        {endpoint(src, "src")} <em>&rarr;</em> {endpoint(dst, "dst")}
      </span>
      <span className="fc-capture-row__proc">{process}</span>
      <span className="fc-capture-row__bytes">{bytes} B</span>
      <Badge tone={TONE[state]}>{LABEL[state]}</Badge>
    </div>
  );
}
