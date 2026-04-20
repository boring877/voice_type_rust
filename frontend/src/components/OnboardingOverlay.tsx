import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Config } from "../types";

type UpdateConfig = <K extends keyof Config>(key: K, value: Config[K]) => void;

interface Props {
  config: Config;
  hotkeyLabel: string;
  capturingHotkey: boolean;
  captureMessage: string;
  onUpdate: UpdateConfig;
  onStartHotkeyCapture: () => void;
  onStopHotkeyCapture: () => void;
  onOpenApiKeyPage: () => void;
}

const TOTAL_STEPS = 3;
const STORAGE_KEY = "voice-type-onboarding-done";

const styleChoices = [
  { value: "none", label: "None", description: "Plain text as-is." },
  { value: "japanese_emojis", label: "Japanese Emojis", description: "Adds mood-matched kaomoji like (^_^)." },
  { value: "niko_style", label: "Niko Style", description: "Adds cute cat expressions like nya!" },
];

export function OnboardingOverlay(props: Props) {
  const [visible, setVisible] = useState(() => !localStorage.getItem(STORAGE_KEY));
  const [step, setStep] = useState(0);

  if (!visible) return null;

  function dismiss() {
    localStorage.setItem(STORAGE_KEY, "1");
    setVisible(false);
  }

  function next() {
    if (step < TOTAL_STEPS - 1) {
      setStep(step + 1);
    }
  }

  function back() {
    if (step > 0) {
      setStep(step - 1);
    }
  }

  return (
    <div className="onboarding-overlay">
      <div className="onboarding-card onboarding-wizard">
        <div className="onboarding-dots">
          {Array.from({ length: TOTAL_STEPS }, (_, i) => (
            <span key={i} className={i === step ? "onboarding-dot active" : "onboarding-dot"} />
          ))}
        </div>

        {step === 0 && <ApiKeyStep {...props} />}
        {step === 1 && <HotkeyStep {...props} />}
        {step === 2 && <StyleStep {...props} />}

        <div className="onboarding-actions">
          {step > 0 && (
            <button className="ghost-button" onClick={back} type="button">
              Back
            </button>
          )}
          <span className="onboarding-spacer" />
          <button className="link-button" onClick={dismiss} type="button">
            Skip
          </button>
          {step < TOTAL_STEPS - 1 ? (
            <button className="primary-button" onClick={next} type="button">
              Next
            </button>
          ) : (
            <button className="primary-button" onClick={dismiss} type="button">
              Get Started
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function ApiKeyStep(props: Props) {
  const { config, onOpenApiKeyPage, onUpdate } = props;
  const [keyStatus, setKeyStatus] = useState<"idle" | "testing" | "valid" | "invalid">("idle");
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => {
    if (debounceRef.current) {
      clearTimeout(debounceRef.current);
    }

    const key = config.api_key.trim();
    if (!key) {
      setKeyStatus("idle");
      return;
    }

    setKeyStatus("testing");
    debounceRef.current = setTimeout(async () => {
      try {
        await invoke("test_api_key", { apiKey: key });
        setKeyStatus("valid");
      } catch {
        setKeyStatus("invalid");
      }
    }, 500);

    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, [config.api_key]);

  const keyStatusIndicator =
    keyStatus === "testing" ? (
      <span className="key-status testing">Testing...</span>
    ) : keyStatus === "valid" ? (
      <span className="key-status valid">Valid</span>
    ) : keyStatus === "invalid" ? (
      <span className="key-status invalid">Invalid</span>
    ) : null;

  return (
    <div className="onboarding-step-content">
      <h2>Connect Your Account</h2>
      <p className="onboarding-desc">
        Voice Type uses the Groq Whisper API for transcription. It&apos;s free and fast.
      </p>

      <label className="field">
        <span>
          Groq API Key {keyStatusIndicator}
        </span>
        <input
          type="password"
          value={config.api_key}
          onChange={(e) => onUpdate("api_key", e.target.value)}
          placeholder="gsk_..."
        />
      </label>

      <button className="link-button onboarding-inline-link" onClick={onOpenApiKeyPage} type="button">
        Get a free API key from Groq
      </button>
    </div>
  );
}

function HotkeyStep(props: Props) {
  const {
    captureMessage,
    capturingHotkey,
    hotkeyLabel,
    onStartHotkeyCapture,
    onStopHotkeyCapture
  } = props;

  return (
    <div className="onboarding-step-content">
      <h2>Pick Your Button</h2>
      <p className="onboarding-desc">
        Hold this key while speaking. Release it and your words are typed instantly.
      </p>

      <div className="onboarding-hotkey-preview">
        <span className="onboarding-hotkey-label">Current key</span>
        <span className="onboarding-hotkey-value">{hotkeyLabel}</span>
      </div>

      <div className="onboarding-capture-row">
        <button
          className={capturingHotkey ? "primary-button" : "secondary-button"}
          onClick={capturingHotkey ? onStopHotkeyCapture : onStartHotkeyCapture}
          type="button"
        >
          {capturingHotkey ? "Cancel" : "Change Key"}
        </button>
        <p className="onboarding-capture-hint">
          {capturingHotkey ? "Press any key or mouse button now..." : captureMessage}
        </p>
      </div>
    </div>
  );
}

function StyleStep(props: Props) {
  const { config, onUpdate } = props;

  return (
    <div className="onboarding-step-content">
      <h2>Add Some Style</h2>
      <p className="onboarding-desc">
        Optionally add a personal touch to your transcribed text. You can change this anytime in settings.
      </p>

      <div className="onboarding-style-list">
        {styleChoices.map((choice) => (
          <button
            key={choice.value}
            className={config.style === choice.value ? "onboarding-style-card active" : "onboarding-style-card"}
            onClick={() => onUpdate("style", choice.value as Config["style"])}
            type="button"
          >
            <span className="onboarding-style-label">{choice.label}</span>
            <span className="onboarding-style-desc">{choice.description}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
