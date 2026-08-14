import { useId } from 'react';

export function Input({ label, hint, error, id, ...props }) {
  const autoId = useId();
  const fieldId = id ?? autoId;
  const message = error ?? hint;
  const messageId = message ? `${fieldId}-message` : undefined;
  return (
    <div className="fc-field">
      {label && <label className="fc-label" htmlFor={fieldId}>{label}</label>}
      <input
        id={fieldId}
        className="fc-input"
        aria-invalid={!!error}
        aria-describedby={messageId}
        {...props}
      />
      {error ? <span id={messageId} className="fc-field__error">{error}</span>
             : hint && <span id={messageId} className="fc-field__hint">{hint}</span>}
    </div>
  );
}
