export function SectionHeading({ eyebrow, title, sub, as: Tag = "h2" }) {
  return (
    <header className="fc-section">
      {eyebrow && <div className="fc-section__eyebrow">{eyebrow}</div>}
      <Tag className="fc-section__title">{title}</Tag>
      {sub && <p className="fc-section__sub">{sub}</p>}
    </header>
  );
}
