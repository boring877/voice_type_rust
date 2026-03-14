import { buildHudCardStyle, hudDescription, hudHeadline, normalizeHudBackgroundMode } from "../lib/hud";
import { runtimeBadge } from "../lib/appearance";
import type { Config, RuntimeSnapshot } from "../types";
import { RuntimeMeter } from "./RuntimeMeter";

export function HudCard(props: {
  config: Config;
  snapshot: RuntimeSnapshot;
  backgroundUrl: string | null;
  active: boolean;
  level: number;
  peak: number;
  preview?: boolean;
}) {
  const { active, backgroundUrl, config, level, peak, preview = false, snapshot } = props;

  const headline = hudHeadline(snapshot.appState);
  const description = hudDescription(snapshot);
  const stateMeta = runtimeBadge(snapshot.appState);
  const backgroundMode = normalizeHudBackgroundMode(config.hud_background_mode);
  const showTopline = config.hud_show_state || config.hud_show_app_name;
  const showDescription = config.hud_show_description && Boolean(description);
  const showMeter = config.hud_show_meter;
  const cardStyle = buildHudCardStyle(config, backgroundUrl);
  const mediaClass = backgroundMode === "image" && backgroundUrl ? " hud-card-media" : "";

  return (
    <div
      className={`hud-card${preview ? " hud-card-preview" : ""}${mediaClass}`}
      data-state={snapshot.appState}
      style={cardStyle}
    >
      {showTopline ? (
        <div
          className={`hud-topline${config.hud_show_state && config.hud_show_app_name ? "" : " hud-topline-single"}`}
        >
          {config.hud_show_state ? (
            <span className={stateMeta.tone}>
              <span className="hud-dot" />
              {stateMeta.label}
            </span>
          ) : null}

          {config.hud_show_app_name ? <span className="hud-chip">Voice Type</span> : null}
        </div>
      ) : null}

      <div className="hud-copy">
        <strong className="hud-title">{headline}</strong>
        {showDescription ? <p className="hud-description">{description}</p> : null}
      </div>

      {showMeter ? <RuntimeMeter active={active} level={level} peak={peak} /> : null}
    </div>
  );
}
