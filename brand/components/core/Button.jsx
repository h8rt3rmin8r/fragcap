export function Button({ variant = "default", children, ...props }) {
  const cls = ["fc-btn", variant !== "default" && `fc-btn--${variant}`]
    .filter(Boolean).join(" ");
  return <button className={cls} {...props}>{children}</button>;
}
