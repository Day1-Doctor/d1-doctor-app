import { useState } from "react";
import { useTranslation } from "react-i18next";
import { useAuthStore } from "../../stores/authStore";

interface AuthWallProps {
  onAuthenticated: () => void;
}

export function AuthWall({ onAuthenticated }: AuthWallProps) {
  const { t } = useTranslation();
  const authenticateWithApiKey = useAuthStore((s) => s.authenticateWithApiKey);
  const isLoading = useAuthStore((s) => s.isLoading);
  const error = useAuthStore((s) => s.error);
  const [apiKey, setApiKey] = useState("");
  const [showKey, setShowKey] = useState(false);

  async function handleConnect() {
    const trimmed = apiKey.trim();
    if (!trimmed) return;
    const ok = await authenticateWithApiKey(trimmed);
    if (ok) onAuthenticated();
  }

  function handleKeyDown(e: React.KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") { e.preventDefault(); handleConnect(); }
  }

  function openExternal(url: string) {
    window.open(url, "_blank", "noopener,noreferrer");
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
      <div className="w-full max-w-md mx-4 rounded-2xl border border-border p-8 bg-card/95 shadow-2xl shadow-black/50" style={{ backdropFilter: "blur(24px)", WebkitBackdropFilter: "blur(24px)" }} role="dialog" aria-modal="true" aria-label={t("auth.signIn")}>
        <div className="text-center mb-6">
          <h2 className="text-lg font-semibold text-text-primary mb-1">{t("auth.signIn")}</h2>
          <p className="text-sm text-text-muted">{t("auth.apiKeyLabel")}</p>
        </div>
        <div className="mb-4">
          <div className="relative">
            <input type={showKey ? "text" : "password"} value={apiKey} onChange={(e) => setApiKey(e.target.value)} onKeyDown={handleKeyDown} placeholder={t("auth.apiKeyPlaceholder")} className="w-full bg-background border border-border rounded-lg px-4 py-3 pr-10 text-sm text-text-primary placeholder:text-text-disabled font-mono focus:outline-none focus:ring-2 focus:ring-accent/50 focus:border-transparent" autoFocus disabled={isLoading} autoComplete="off" spellCheck={false} />
            <button type="button" onClick={() => setShowKey((v) => !v)} className="absolute right-3 top-1/2 -translate-y-1/2 text-text-muted hover:text-text-primary transition-colors" aria-label={showKey ? "Hide API key" : "Show API key"} tabIndex={-1}>
              {showKey ? (
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94" /><path d="M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19" /><line x1="1" y1="1" x2="23" y2="23" /></svg>
              ) : (
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" /><circle cx="12" cy="12" r="3" /></svg>
              )}
            </button>
          </div>
        </div>
        {error && <p className="text-error text-sm mb-4 text-center">{t("auth.invalidKey")}</p>}
        <button onClick={handleConnect} disabled={isLoading || !apiKey.trim()} className="w-full py-3 rounded-lg font-medium text-sm transition-colors duration-100 bg-accent hover:bg-accent-hover text-background disabled:bg-muted disabled:text-text-disabled disabled:cursor-not-allowed focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50">{isLoading ? t("auth.connecting") : t("auth.connect")}</button>
        <div className="flex items-center gap-3 my-6"><div className="flex-1 h-px bg-border" /><span className="text-text-disabled text-sm">{t("auth.or")}</span><div className="flex-1 h-px bg-border" /></div>
        <button onClick={() => openExternal("https://day1.doctor/auth/login")} className="w-full py-3 rounded-lg font-medium text-sm transition-colors duration-100 border border-border bg-transparent text-text-primary hover:bg-muted/50 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50">{t("auth.signInWithAccount")}</button>
        <p className="text-center mt-4"><button onClick={() => openExternal("https://day1.doctor/auth/register")} className="text-accent text-sm hover:underline focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50 rounded">{t("auth.createAccount")}</button></p>
      </div>
    </div>
  );
}
