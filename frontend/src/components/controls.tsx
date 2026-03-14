import { normalizedColor } from "../lib/appearance";

export function SectionHeader({ title }: { title: string }) {
  return (
    <header className="section-header">
      <h2>{title}</h2>
    </header>
  );
}

export function SegmentedButton(props: {
  active: boolean;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      className={props.active ? "theme-pill active" : "theme-pill"}
      onClick={props.onClick}
      type="button"
    >
      {props.label}
    </button>
  );
}

export function ColorField(props: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  fallback: string;
}) {
  const swatchValue = normalizedColor(props.value, props.fallback);

  return (
    <label className="field">
      <span>{props.label}</span>
      <div className="color-input">
        <input
          aria-label={`${props.label} picker`}
          className="color-swatch"
          type="color"
          value={swatchValue}
          onChange={(event) => props.onChange(event.target.value)}
        />
        <input
          className="color-text"
          type="text"
          value={props.value}
          onChange={(event) => props.onChange(event.target.value)}
          placeholder={props.fallback}
        />
      </div>
    </label>
  );
}

export function FilePathField(props: {
  label: string;
  value: string;
  onBrowse: () => void;
  onClear: () => void;
  placeholder?: string;
}) {
  return (
    <label className="field field-full">
      <span>{props.label}</span>
      <div className="path-input">
        <input
          type="text"
          value={props.value}
          placeholder={props.placeholder}
          readOnly
        />
        <button className="secondary-button" onClick={props.onBrowse} type="button">
          Browse
        </button>
        <button className="ghost-button" onClick={props.onClear} type="button">
          Clear
        </button>
      </div>
    </label>
  );
}

export function TextField(props: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  type?: "text" | "password";
}) {
  return (
    <label className="field">
      <span>{props.label}</span>
      <input
        type={props.type ?? "text"}
        value={props.value}
        onChange={(event) => props.onChange(event.target.value)}
        placeholder={props.placeholder}
      />
    </label>
  );
}

export function TextAreaField(props: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}) {
  return (
    <label className="field field-full">
      <span>{props.label}</span>
      <textarea
        value={props.value}
        onChange={(event) => props.onChange(event.target.value)}
        placeholder={props.placeholder}
        rows={4}
      />
    </label>
  );
}

export function SelectField(props: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: Array<{ label: string; value: string }>;
}) {
  return (
    <label className="field">
      <span>{props.label}</span>
      <select value={props.value} onChange={(event) => props.onChange(event.target.value)}>
        {props.options.map((option) => (
          <option key={`${option.label}-${option.value}`} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </label>
  );
}

export function DisplayField(props: {
  label: string;
  value: string;
}) {
  return (
    <div className="field">
      <span>{props.label}</span>
      <div className="value-display">{props.value}</div>
    </div>
  );
}

export function NumberField(props: {
  label: string;
  value: number;
  onChange: (value: number) => void;
  step: number;
}) {
  const precision = String(props.step).includes(".")
    ? String(props.step).split(".")[1].length
    : 0;

  function nudge(direction: 1 | -1) {
    const nextValue = Number((props.value + props.step * direction).toFixed(precision));
    props.onChange(nextValue);
  }

  return (
    <label className="field">
      <span>{props.label}</span>
      <div className="number-input">
        <input
          type="number"
          value={props.value}
          step={props.step}
          onChange={(event) => {
            const nextValue = Number(event.target.value);
            if (Number.isFinite(nextValue)) {
              props.onChange(nextValue);
            }
          }}
        />
        <div className="number-stepper">
          <button
            aria-label={`Increase ${props.label}`}
            className="stepper-button"
            onClick={() => nudge(1)}
            type="button"
          >
            +
          </button>
          <button
            aria-label={`Decrease ${props.label}`}
            className="stepper-button"
            onClick={() => nudge(-1)}
            type="button"
          >
            -
          </button>
        </div>
      </div>
    </label>
  );
}

export function Toggle(props: {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <button
      className={props.checked ? "toggle-card active" : "toggle-card"}
      onClick={() => props.onChange(!props.checked)}
      type="button"
    >
      <span className="toggle-copy">
        <span className="toggle-title">{props.label}</span>
        {props.description ? <span className="toggle-description">{props.description}</span> : null}
      </span>
      <strong className="toggle-state">{props.checked ? "On" : "Off"}</strong>
    </button>
  );
}

export function ChipToggle(props: {
  label: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}) {
  return (
    <button
      className={props.checked ? "chip-toggle active" : "chip-toggle"}
      onClick={() => props.onChange(!props.checked)}
      type="button"
    >
      {props.label}
    </button>
  );
}
