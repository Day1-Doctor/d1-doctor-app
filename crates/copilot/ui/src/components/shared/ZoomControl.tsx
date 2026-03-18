import { useTranslation } from "react-i18next";
import { useSettingsStore, type ZoomLevel } from "../../stores/settingsStore";

const ZOOM_LEVELS: ZoomLevel[] = [80, 90, 100, 110, 120, 130, 150];

export function ZoomControl() {
  const { t } = useTranslation();
  const zoomLevel = useSettingsStore((s) => s.zoomLevel);
  const setZoomLevel = useSettingsStore((s) => s.setZoomLevel);

  const currentIndex = ZOOM_LEVELS.indexOf(zoomLevel);

  const zoomIn = () => {
    if (currentIndex < ZOOM_LEVELS.length - 1) {
      setZoomLevel(ZOOM_LEVELS[currentIndex + 1]);
    }
  };

  const zoomOut = () => {
    if (currentIndex > 0) {
      setZoomLevel(ZOOM_LEVELS[currentIndex - 1]);
    }
  };

  const reset = () => {
    setZoomLevel(100);
  };

  return (
    <div className="flex items-center gap-1 text-[12px]">
      <button
        onClick={zoomOut}
        disabled={currentIndex <= 0}
        className="w-5 h-5 flex items-center justify-center rounded hover:bg-[#1F1F1F] disabled:opacity-30 text-[#A0A0A0]"
        title={t("settings.zoomOut", "Zoom out")}
      >
        -
      </button>
      <button
        onClick={reset}
        className="px-1 h-5 flex items-center justify-center rounded hover:bg-[#1F1F1F] text-[#A0A0A0] min-w-[32px]"
        title={t("settings.zoomReset", "Reset zoom")}
      >
        {zoomLevel}%
      </button>
      <button
        onClick={zoomIn}
        disabled={currentIndex >= ZOOM_LEVELS.length - 1}
        className="w-5 h-5 flex items-center justify-center rounded hover:bg-[#1F1F1F] disabled:opacity-30 text-[#A0A0A0]"
        title={t("settings.zoomIn", "Zoom in")}
      >
        +
      </button>
    </div>
  );
}
