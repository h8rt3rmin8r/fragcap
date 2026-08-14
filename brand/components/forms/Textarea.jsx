export function Textarea({ label, hint, error, id, ...props }) {
  return (
    <div className="fc-field">
      {label && <label className="fc-label" htmlFor={id}>{label}</label>}
      <textarea id={id} className="fc-textarea" aria-invalid={!!error} {...props} />
      {error ? <span className="fc-field__error">{error}</span>
             : hint && <span className="fc-field__hint">{hint}</span>}
    </div>
  );
}
