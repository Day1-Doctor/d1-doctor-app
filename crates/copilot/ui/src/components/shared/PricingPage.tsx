import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useBillingStore, SubscriptionTier } from "../../stores/billingStore";
import { useCostStore } from "../../stores/costStore";

interface TierCard {
  id: SubscriptionTier;
  nameKey: string;
  priceMonthly: number;
  priceAnnual: number;
  maxAgents: number;
  monthlyCredits: number;
  features: string[];
}

const TIERS: TierCard[] = [
  {
    id: "free_man",
    nameKey: "pricing.freeMan",
    priceMonthly: 0,
    priceAnnual: 0,
    maxAgents: 1,
    monthlyCredits: 100,
    features: [
      "1 office spot (Dr. Bob)",
      "100 DD credits / month",
      "Basic chat",
      "Community support",
    ],
  },
  {
    id: "mini_shop",
    nameKey: "pricing.miniShop",
    priceMonthly: 19,
    priceAnnual: 190,
    maxAgents: 3,
    monthlyCredits: 1_000,
    features: [
      "3 office spots",
      "1,000 DD credits / month",
      "Web search + fetch tools",
      "Document generation",
      "Priority support",
    ],
  },
  {
    id: "rocket_inc",
    nameKey: "pricing.rocketInc",
    priceMonthly: 49,
    priceAnnual: 490,
    maxAgents: 6,
    monthlyCredits: 5_000,
    features: [
      "6 office spots (full office)",
      "5,000 DD credits / month",
      "All MCP tools",
      "Browser automation",
      "Hot-update config",
      "Dedicated support",
    ],
  },
];

interface TopUpOption {
  nameKey: string;
  dd: number;
  price: number;
}

const TOP_UP_OPTIONS: TopUpOption[] = [
  { nameKey: "pricing.boost", dd: 1_000, price: 10 },
  { nameKey: "pricing.powerPack", dd: 6_000, price: 50 },
];

/**
 * PricingPage — 3-column tier layout with annual toggle, top-up section,
 * and current balance display.
 */
export function PricingPage() {
  const { t } = useTranslation();
  const [annual, setAnnual] = useState(false);
  const [customSlider, setCustomSlider] = useState(10);

  const currentTier = useBillingStore((s) => s.tier);
  const setTier = useBillingStore((s) => s.setTier);
  const balance = useCostStore((s) => s.balance);
  const limit = useCostStore((s) => s.limit);

  const customDd = Math.round(customSlider * 100);

  return (
    <div className="mx-auto max-w-4xl px-4 py-8">
      {/* Header */}
      <h1 className="mb-2 text-center text-2xl font-bold text-text-primary">
        {t("pricing.title")}
      </h1>
      <p className="mb-6 text-center text-sm text-text-secondary">
        {t("pricing.subtitle")}
      </p>

      {/* Annual toggle */}
      <div className="mb-8 flex items-center justify-center gap-3">
        <span
          className={`text-sm ${!annual ? "text-text-primary font-medium" : "text-text-muted"}`}
        >
          {t("pricing.monthly")}
        </span>
        <button
          onClick={() => setAnnual((v) => !v)}
          className={`relative h-6 w-11 rounded-full transition-colors ${
            annual ? "bg-accent" : "bg-muted"
          }`}
          role="switch"
          aria-checked={annual}
          aria-label={t("pricing.annual")}
        >
          <span
            className={`absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white shadow
              transition-transform ${annual ? "translate-x-5" : "translate-x-0"}`}
          />
        </button>
        <span
          className={`text-sm ${annual ? "text-text-primary font-medium" : "text-text-muted"}`}
        >
          {t("pricing.annual")}{" "}
          <span className="text-sm text-accent">{t("pricing.savePercent")}</span>
        </span>
      </div>

      {/* Tier cards */}
      <div className="mb-12 grid grid-cols-1 gap-4 md:grid-cols-3">
        {TIERS.map((tier) => {
          const isCurrent = tier.id === currentTier;
          const price = annual
            ? (tier.priceAnnual / 12).toFixed(2)
            : tier.priceMonthly.toFixed(2);

          return (
            <div
              key={tier.id}
              className={`rounded-xl border p-5 transition-shadow ${
                isCurrent
                  ? "border-accent shadow-lg shadow-accent/10"
                  : "border-border hover:shadow-md"
              }`}
            >
              <h3 className="mb-1 text-lg font-semibold text-text-primary">
                {t(tier.nameKey)}
              </h3>

              <div className="mb-4">
                <span className="text-2xl font-bold text-text-primary">
                  ${price}
                </span>
                <span className="text-sm text-text-muted"> {t("pricing.perMonth")}</span>
                {annual && tier.priceAnnual > 0 && (
                  <p className="text-sm text-text-muted">
                    {t("pricing.billedAnnually", { amount: tier.priceAnnual })}
                  </p>
                )}
              </div>

              <ul className="mb-5 space-y-2">
                {tier.features.map((f) => (
                  <li
                    key={f}
                    className="flex items-start gap-2 text-sm text-text-secondary"
                  >
                    <svg
                      width="16"
                      height="16"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      className="mt-0.5 shrink-0 text-accent"
                      aria-hidden="true"
                    >
                      <path d="M20 6 9 17l-5-5" />
                    </svg>
                    {f}
                  </li>
                ))}
              </ul>

              <button
                onClick={() => setTier(tier.id)}
                disabled={isCurrent}
                className={`w-full rounded-lg px-4 py-2 text-sm font-medium transition-colors
                  focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
                  ${
                    isCurrent
                      ? "cursor-default border border-accent/30 bg-accent/10 text-accent"
                      : "bg-accent text-white hover:bg-accent/90"
                  }`}
              >
                {isCurrent ? t("pricing.currentPlan") : t("pricing.selectPlan")}
              </button>
            </div>
          );
        })}
      </div>

      {/* Top-up section */}
      <div className="rounded-xl border border-border p-5">
        <h2 className="mb-1 text-lg font-semibold text-text-primary">
          {t("pricing.topUp")}
        </h2>
        <p className="mb-4 text-sm text-text-secondary">
          {t("pricing.topUpDesc")}
        </p>

        {/* Current balance */}
        <div className="mb-4 flex items-center gap-2 text-sm">
          <span className="text-text-muted">{t("pricing.currentBalance")}</span>
          <span className="font-medium text-text-primary">
            {balance} / {limit} DD
          </span>
        </div>

        <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
          {TOP_UP_OPTIONS.map((opt) => (
            <button
              key={opt.nameKey}
              className="rounded-lg border border-border p-3 text-left transition-colors
                hover:border-accent hover:bg-accent/5
                focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
            >
              <p className="font-medium text-text-primary">{t(opt.nameKey)}</p>
              <p className="text-sm text-text-secondary">
                {opt.dd.toLocaleString()} DD &mdash; ${opt.price}
              </p>
            </button>
          ))}

          {/* Custom slider */}
          <div className="rounded-lg border border-border p-3">
            <p className="mb-2 font-medium text-text-primary">{t("pricing.custom")}</p>
            <input
              type="range"
              min={1}
              max={100}
              value={customSlider}
              onChange={(e) => setCustomSlider(Number(e.target.value))}
              className="w-full accent-accent"
              aria-label={t("pricing.custom")}
            />
            <p className="mt-1 text-sm text-text-secondary">
              {customDd.toLocaleString()} DD &mdash; ${customSlider}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
