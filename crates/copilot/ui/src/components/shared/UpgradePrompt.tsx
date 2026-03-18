import { useTranslation } from "react-i18next";
import { useBillingStore } from "../../stores/billingStore";

/**
 * UpgradePrompt — Modal shown when a user's task requires more agents
 * than their current subscription tier allows.
 */
export function UpgradePrompt() {
  const { t } = useTranslation();
  const showUpgradePrompt = useBillingStore((s) => s.showUpgradePrompt);
  const upgradeMessage = useBillingStore((s) => s.upgradeMessage);
  const closeUpgradePrompt = useBillingStore((s) => s.closeUpgradePrompt);
  const setTier = useBillingStore((s) => s.setTier);

  if (!showUpgradePrompt) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center"
      role="dialog"
      aria-modal="true"
      aria-label={t("upgrade.title")}
    >
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/60"
        onClick={closeUpgradePrompt}
        aria-hidden="true"
      />

      {/* Modal */}
      <div
        className="relative z-10 w-full max-w-md rounded-xl border border-border
          bg-card p-6 shadow-2xl shadow-black/40"
        style={{
          backdropFilter: "blur(24px)",
          WebkitBackdropFilter: "blur(24px)",
        }}
      >
        {/* Icon */}
        <div className="mb-4 flex justify-center">
          <div className="flex h-12 w-12 items-center justify-center rounded-full bg-warning/10">
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="text-warning"
              aria-hidden="true"
            >
              <path d="M12 9v4" />
              <path d="M12 17h.01" />
              <path d="M3.6 15.4 10.8 3.2a1.4 1.4 0 0 1 2.4 0l7.2 12.2A1.4 1.4 0 0 1 19.2 18H4.8a1.4 1.4 0 0 1-1.2-2.6z" />
            </svg>
          </div>
        </div>

        <h2 className="mb-2 text-center text-lg font-semibold text-text-primary">
          {t("upgrade.title")}
        </h2>

        <p className="mb-6 text-center text-sm text-text-secondary">
          {upgradeMessage || t("upgrade.defaultMessage")}
        </p>

        {/* Actions */}
        <div className="flex flex-col gap-3">
          <button
            onClick={() => {
              setTier("mini_shop");
              closeUpgradePrompt();
            }}
            className="w-full rounded-lg bg-accent px-4 py-2.5 text-sm font-medium
              text-white transition-colors hover:bg-accent/90
              focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
          >
            {t("upgrade.upgradeToMiniShop")}
          </button>

          <button
            onClick={closeUpgradePrompt}
            className="w-full rounded-lg border border-border bg-transparent px-4 py-2.5
              text-sm font-medium text-text-secondary transition-colors
              hover:bg-muted hover:text-text-primary
              focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
          >
            {t("upgrade.runWithDrBob")}
          </button>
        </div>
      </div>
    </div>
  );
}
