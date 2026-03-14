import { ChipToggle, ColorField, FilePathField, SectionHeader, SegmentedButton } from "./controls";
import { HudCard } from "./HudCard";
import type { Config, RuntimeSnapshot } from "../types";

type UpdateConfig = <K extends keyof Config>(key: K, value: Config[K]) => void;

export function HudStageSection(props: {
  config: Config;
  runtime: RuntimeSnapshot;
  backgroundUrl: string | null;
  onPickHudBackgroundImage: () => void;
  onUpdate: UpdateConfig;
}) {
  const { backgroundUrl, config, onPickHudBackgroundImage, onUpdate, runtime } = props;

  const previewState = !config.hud_enabled
    ? "ready"
    : runtime.appState === "processing"
      ? "processing"
      : "recording";
  const previewSnapshot: RuntimeSnapshot = {
    ...runtime,
    appState: previewState,
    audioLevel: previewState === "processing" ? 0.22 : 0.72,
    statusText:
      previewState === "processing"
        ? "Turning your speech into text."
        : runtime.statusText || "Release your hotkey when you are done speaking."
  };
  const cornerLabel = config.hud_side === "left" ? "bottom-left" : "bottom-right";

  return (
    <section className="hud-stage card">
      <div className="hud-stage-toprow">
        <SectionHeader title="Floating HUD" />
        <div className="segmented-row hud-stage-power">
          <SegmentedButton
            active={config.hud_enabled}
            label="On"
            onClick={() => onUpdate("hud_enabled", true)}
          />
          <SegmentedButton
            active={!config.hud_enabled}
            label="Off"
            onClick={() => onUpdate("hud_enabled", false)}
          />
        </div>
      </div>

      <div className="hud-stage-body">
        <div
          className={`hud-preview-frame hud-side-${config.hud_side}${config.hud_enabled ? "" : " preview-disabled"}`}
        >
          <HudCard
            active={config.hud_enabled}
            backgroundUrl={backgroundUrl}
            config={config}
            level={previewState === "processing" ? 0.22 : 0.72}
            peak={previewState === "processing" ? 0.5 : 0.9}
            preview
            snapshot={previewSnapshot}
          />
        </div>

        <div className="hud-stage-panel">
          <div className="hud-stage-inline-grid">
            <div className="hud-stage-control">
              <span className="hud-stage-label">Side</span>
              <div className="segmented-row">
                <SegmentedButton
                  active={config.hud_side === "left"}
                  label="Left"
                  onClick={() => onUpdate("hud_side", "left")}
                />
                <SegmentedButton
                  active={config.hud_side === "right"}
                  label="Right"
                  onClick={() => onUpdate("hud_side", "right")}
                />
              </div>
            </div>

            <div className="hud-stage-control">
              <span className="hud-stage-label">Background</span>
              <div className="segmented-row">
                <SegmentedButton
                  active={config.hud_background_mode === "glass"}
                  label="Glass"
                  onClick={() => onUpdate("hud_background_mode", "glass")}
                />
                <SegmentedButton
                  active={config.hud_background_mode === "image"}
                  label="Image"
                  onClick={() => onUpdate("hud_background_mode", "image")}
                />
              </div>
            </div>
          </div>

          <div className="hud-stage-control">
            <span className="hud-stage-label">Show</span>
            <div className="hud-stage-chip-grid">
              <ChipToggle
                label="State"
                checked={config.hud_show_state}
                onChange={(value) => onUpdate("hud_show_state", value)}
              />
              <ChipToggle
                label="Label"
                checked={config.hud_show_app_name}
                onChange={(value) => onUpdate("hud_show_app_name", value)}
              />
              <ChipToggle
                label="Helper"
                checked={config.hud_show_description}
                onChange={(value) => onUpdate("hud_show_description", value)}
              />
              <ChipToggle
                label="Meter"
                checked={config.hud_show_meter}
                onChange={(value) => onUpdate("hud_show_meter", value)}
              />
            </div>
          </div>

          {config.hud_background_mode === "image" ? (
            <FilePathField
              label="HUD image or GIF"
              value={config.hud_background_path}
              onBrowse={onPickHudBackgroundImage}
              onClear={() => onUpdate("hud_background_path", "")}
              placeholder="Built-in sakura HUD background"
            />
          ) : null}

          <ColorField
            label="HUD tint"
            value={config.hud_background_color}
            onChange={(value) => onUpdate("hud_background_color", value)}
            fallback="#101419"
          />

          <p className="hud-stage-note">
            {config.hud_enabled
              ? `Shows in the ${cornerLabel} corner while recording or transcribing.`
              : "Hidden until you turn it back on."}
          </p>
        </div>
      </div>
    </section>
  );
}
