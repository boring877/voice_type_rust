import { useState } from "react";

export function OnboardingOverlay() {
  const [visible, setVisible] = useState(
    () => !localStorage.getItem("voice-type-onboarding-done")
  );

  if (!visible) return null;

  function dismiss() {
    localStorage.setItem("voice-type-onboarding-done", "1");
    setVisible(false);
  }

  return (
    <div className="onboarding-overlay">
      <div className="onboarding-card">
        <h2>Welcome to Voice Type</h2>
        <p>Your voice, typed instantly.</p>
        <div className="onboarding-steps">
          <div className="onboarding-step">
            <span className="step-number">1</span>
            <span>Add API key</span>
          </div>
          <div className="onboarding-step">
            <span className="step-number">2</span>
            <span>Pick model</span>
          </div>
          <div className="onboarding-step">
            <span className="step-number">3</span>
            <span>Set hotkey</span>
          </div>
        </div>
        <button className="primary-button" onClick={dismiss} type="button">
          Get Started
        </button>
      </div>
    </div>
  );
}
