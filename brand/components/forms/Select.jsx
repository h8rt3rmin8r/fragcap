import { useId } from 'react';

export function Select({ label, hint, error, id, children, ...props }) {
  const autoId = useId();
  const fieldId = id ?? autoId;
  const message = error ?? hint;
  const messageId = message ? `${fieldId}-message` : undefined;
  return (
    <div className="fc-field">
      {label && <label className="fc-label" htmlFor={fieldId}>{label}</label>}
      <select
        id={fieldId}
        className="fc-select"
        aria-invalid={!!error}
        aria-describedby={messageId}
        {...props}
      >
        {children}
      </select>
      {error ? <span id={messageId} className="fc-field__error">{error}</span>
             : hint && <span id={messageId} className="fc-field__hint">{hint}</span>}
    </div>
  );
}
