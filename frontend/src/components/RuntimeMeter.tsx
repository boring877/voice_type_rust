import { useMemo, type CSSProperties } from "react";

function clampUnit(value: number) {
  return Math.max(0, Math.min(1, value));
}

export function RuntimeMeter(props: {
  level: number;
  peak: number;
  active: boolean;
}) {
  const { active, level, peak } = props;

  const bars = useMemo(() => {
    const count = 20;
    const liveLevel = clampUnit(level);
    const livePeak = clampUnit(Math.max(level, peak));
    const intensity = active ? Math.max(liveLevel, livePeak * 0.82) : livePeak * 0.28;

    return Array.from({ length: count }, (_, index) => {
      const mid = (count - 1) / 2;
      const offset = Math.abs(index - mid) / mid;
      const ridge = 1 - Math.pow(offset, 1.2) * 0.72;
      const pattern = [1, 0.88, 0.96, 0.82, 0.91][index % 5];
      const floor = active ? 0.14 : 0.08;
      const height = clampUnit(
        floor + intensity * ridge * pattern + livePeak * 0.12 * (1 - offset * 0.55)
      );

      return {
        height: Math.max(active ? 0.15 : 0.08, height),
        opacity: active ? 0.42 + height * 0.5 : 0.2 + height * 0.24,
        delay: `${(index - count / 2) * 0.08}s`,
        bounce: String(1 + (0.08 + intensity * 0.16) * ridge)
      };
    });
  }, [active, level, peak]);

  return (
    <div
      className={`meter-shell meter-shell-bars ${active ? "meter-shell-active" : ""}`}
      aria-label="Audio level meter"
    >
      <div className="meter-bars">
        {bars.map((bar, index) => (
          <span
            key={index}
            className="meter-bar"
            style={
              {
                "--bar-height": `${Math.round(bar.height * 100)}%`,
                "--bar-opacity": String(bar.opacity),
                "--bar-delay": bar.delay,
                "--bar-bounce": bar.bounce
              } as CSSProperties
            }
          />
        ))}
      </div>
    </div>
  );
}
