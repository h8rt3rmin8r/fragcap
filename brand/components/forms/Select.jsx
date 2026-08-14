export function Select({ label, hint, error, id, children, ...props }) {
  return (
    <div className="fc-field">
      {label && <label className="fc-label" htmlFor={id}>{label}</label>}
      <select id={id} className="fc-select" aria-invalid={!!error} {...props}>
        {children}
      </select>
      {error ? <span className="fc-field__error">{error}</span>
             : hint && <span className="fc-field__hint">{hint}</span>}
    </div>
  );
}
