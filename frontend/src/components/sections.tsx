import {
  ColorField,
  DisplayField,
  FilePathField,
  SectionHeader,
  SegmentedButton,
  SelectField,
  TextAreaField,
  TextField,
  Toggle
} from "./controls";
import {
  grammarModels,
  grammarProfiles,
  languageOptions,
  modelOptions,
  styleOptions
} from "../lib/options";
import type { BackgroundMode, Config, ThemeMode } from "../types";

type UpdateConfig = <K extends keyof Config>(key: K, value: Config[K]) => void;

export function SetupSection(props: {
  config: Config;
  micNames: string[];
  hotkeyLabel: string;
  capturingHotkey: boolean;
  captureMessage: string;
  onUpdate: UpdateConfig;
  onStartHotkeyCapture: () => void;
  onStopHotkeyCapture: () => void;
  onOpenApiKeyPage: () => void;
}) {
  const {
    captureMessage,
    capturingHotkey,
    config,
    hotkeyLabel,
    micNames,
    onOpenApiKeyPage,
    onStartHotkeyCapture,
    onStopHotkeyCapture,
    onUpdate
  } = props;

  return (
    <section className="dock-section">
      <SectionHeader title="Setup" />

      <div className="field-grid">
        <TextField
          label="Groq API key"
          type="password"
          value={config.api_key}
          onChange={(value) => onUpdate("api_key", value)}
          placeholder="gsk_..."
        />
        <SelectField
          label="Language"
          value={config.language}
          onChange={(value) => onUpdate("language", value)}
          options={languageOptions.map((value) => ({
            label: value === "auto" ? "Auto" : value.toUpperCase(),
            value
          }))}
        />
        <SelectField
          label="Whisper model"
          value={config.transcription_model}
          onChange={(value) => onUpdate("transcription_model", value)}
          options={modelOptions}
        />
        <SelectField
          label="Grammar profile"
          value={config.grammar_profile}
          onChange={(value) => onUpdate("grammar_profile", value)}
          options={grammarProfiles}
        />
        <SelectField
          label="Grammar model"
          value={config.grammar_model}
          onChange={(value) => onUpdate("grammar_model", value)}
          options={grammarModels}
        />
      </div>

      <div className="field-grid">
        <DisplayField label="Push-to-talk key" value={hotkeyLabel} />
        <SelectField
          label="Input device"
          value={config.mic_index === null ? "" : String(config.mic_index)}
          onChange={(value) => onUpdate("mic_index", value === "" ? null : Number(value))}
          options={[
            { label: "Default microphone", value: "" },
            ...micNames.map((name, index) => ({
              label: `${name} (#${index})`,
              value: String(index)
            }))
          ]}
        />
      </div>

      <div className="capture-row">
        <button className="secondary-button" onClick={onStartHotkeyCapture} type="button">
          {capturingHotkey ? "Listening..." : "Change hotkey"}
        </button>
        {capturingHotkey ? (
          <button className="ghost-button" onClick={onStopHotkeyCapture} type="button">
            Cancel
          </button>
        ) : null}
        <p className="hint-text">{captureMessage}</p>
      </div>

      <div className="voice-actions">
        <Toggle
          label="Grammar correction"
          description="Fixes grammar and punctuation before pasting."
          checked={config.grammar_correction}
          onChange={(value) => onUpdate("grammar_correction", value)}
        />
        <button className="link-button" onClick={onOpenApiKeyPage} type="button">
          Get API key
        </button>
      </div>
    </section>
  );
}

export function SettingsSection(props: {
  config: Config;
  filterWordsText: string;
  onUpdate: UpdateConfig;
  onUpdateFilterWords: (value: string) => void;
}) {
  const { config, filterWordsText, onUpdate, onUpdateFilterWords } = props;

  return (
    <section className="dock-section">
      <SectionHeader title="Settings" />

      <div className="feature-grid">
        <Toggle
          label="Casual mode"
          description="Lowercase text with lighter punctuation."
          checked={config.casual_mode}
          onChange={(value) => onUpdate("casual_mode", value)}
        />
        <Toggle
          label="Shorthand mode"
          description="Common phrases become shortcuts (e.g. &quot;to be honest&quot; to &quot;tbh&quot;)."
          checked={config.shorthand_mode}
          onChange={(value) => onUpdate("shorthand_mode", value)}
        />
        <Toggle
          label="Number formatting"
          description="Convert spoken numbers (e.g. &quot;two million&quot; to &quot;2,000,000&quot;)."
          checked={config.accounting_mode}
          onChange={(value) => onUpdate("accounting_mode", value)}
        />
        <Toggle
          label="Auto-copy"
          description="Copy transcription to clipboard automatically."
          checked={config.auto_copy}
          onChange={(value) => onUpdate("auto_copy", value)}
        />
        <Toggle
          label="Always on top"
          description="Keep this window above others."
          checked={config.always_on_top}
          onChange={(value) => onUpdate("always_on_top", value)}
        />
      </div>

      <TextAreaField
        label="Phrases to remove"
        value={filterWordsText}
        onChange={onUpdateFilterWords}
        placeholder="Add phrases to remove, separated by commas"
      />
    </section>
  );
}

export function AdvancedSection(props: {
  theme: ThemeMode;
  backgroundMode: BackgroundMode;
  config: Config;
  onUpdate: UpdateConfig;
  onPickBackgroundImage: () => void;
  onResetAppearance: () => void;
  expanded: boolean;
  onToggle: () => void;
}) {
  const { backgroundMode, config, onPickBackgroundImage, onResetAppearance, onUpdate, theme } =
    props;

  if (!props.expanded) {
    return null;
  }

  return (
    <>
      <section className="dock-section">
        <SectionHeader title="Appearance" />

        <div className="segmented-row">
          <SegmentedButton
            active={theme === "dark"}
            label="Dark"
            onClick={() => onUpdate("theme", "dark")}
          />
          <SegmentedButton
            active={theme === "light"}
            label="Light"
            onClick={() => onUpdate("theme", "light")}
          />
        </div>

        <div className="segmented-row">
          <SegmentedButton
            active={backgroundMode === "solid"}
            label="Solid"
            onClick={() => onUpdate("background_mode", "solid")}
          />
          <SegmentedButton
            active={backgroundMode === "gradient"}
            label="Gradient"
            onClick={() => onUpdate("background_mode", "gradient")}
          />
          <SegmentedButton
            active={backgroundMode === "image"}
            label="Image"
            onClick={() => onUpdate("background_mode", "image")}
          />
        </div>

        <div className="field-grid appearance-color-grid">
          <ColorField
            label="Container color"
            value={config.panel_color}
            onChange={(value) => onUpdate("panel_color", value)}
            fallback={theme === "light" ? "#ffffff" : "#111418"}
          />
          <ColorField
            label="Section color"
            value={config.section_color}
            onChange={(value) => onUpdate("section_color", value)}
            fallback={theme === "light" ? "#f7f8fa" : "#161a1f"}
          />
          <ColorField
            label="Meter color"
            value={config.meter_color_start}
            onChange={(value) => onUpdate("meter_color_start", value)}
            fallback={theme === "light" ? "#e88bad" : "#ff99c4"}
          />
          <ColorField
            label="Meter accent"
            value={config.meter_color_end}
            onChange={(value) => onUpdate("meter_color_end", value)}
            fallback={theme === "light" ? "#6f96da" : "#7faef6"}
          />
        </div>

        {backgroundMode === "solid" ? (
          <div className="field-grid">
            <ColorField
              label="Background color"
              value={config.background_color}
              onChange={(value) => onUpdate("background_color", value)}
              fallback="#111315"
            />
          </div>
        ) : null}

        {backgroundMode === "gradient" ? (
          <div className="field-grid">
            <ColorField
              label="Gradient start"
              value={config.background_gradient_start}
              onChange={(value) => onUpdate("background_gradient_start", value)}
              fallback="#202938"
            />
            <ColorField
              label="Gradient end"
              value={config.background_gradient_end}
              onChange={(value) => onUpdate("background_gradient_end", value)}
              fallback="#0f1117"
            />
          </div>
        ) : null}

        {backgroundMode === "image" ? (
          <div className="field-grid">
            <FilePathField
              label="Image path"
              value={config.background_image_path}
              onBrowse={onPickBackgroundImage}
              onClear={() => onUpdate("background_image_path", "")}
              placeholder="Built-in sakura background"
            />
            <ColorField
              label="Fallback color"
              value={config.background_color}
              onChange={(value) => onUpdate("background_color", value)}
              fallback="#111315"
            />
          </div>
        ) : null}

        <div className="appearance-footer">
          <button className="ghost-button" onClick={onResetAppearance} type="button">
            Defaults
          </button>
        </div>
      </section>

      <section className="dock-section">
        <SectionHeader title="Style" />
        <div className="style-panel-body">
          <p className="style-panel-note">
            Adds a small character touch to the final text after transcription finishes.
          </p>
          <div className="style-panel-fields">
            <SelectField
              label="Output style"
              value={config.style}
              onChange={(value) => onUpdate("style", value)}
              options={styleOptions}
            />
          </div>
        </div>
      </section>
    </>
  );
}
