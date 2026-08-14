export function Card({ raised = false, children, ...props }) {
  const cls = ["fc-card", raised && "fc-card--raised"].filter(Boolean).join(" ");
  return <div className={cls} {...props}>{children}</div>;
}
