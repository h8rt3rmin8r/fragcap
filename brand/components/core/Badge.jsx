// tone: "neutral" | "signal" | "capture" | "fault"
// Each toned badge prints a glyph as well as a color, so capture state
// survives greyscale printing and color-vision deficiency.
export function Badge({ tone = "neutral", children, ...props }) {
  const cls = ["fc-badge", tone !== "neutral" && `fc-badge--${tone}`]
    .filter(Boolean).join(" ");
  return <span className={cls} {...props}>{children}</span>;
}
