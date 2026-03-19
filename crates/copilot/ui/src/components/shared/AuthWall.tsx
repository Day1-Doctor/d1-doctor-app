import { useTranslation } from "react-i18next";
import { useAuthStore } from "../../stores/authStore";

interface AuthWallProps {
  onAuthenticated: () => void;
}

export function AuthWall({ onAuthenticated }: AuthWallProps) {
  const { t } = useTranslation();
  const isLoading = useAuthStore((s) => s.isLoading);

  function openExternal(url: string) {
    // Append the deep link redirect so the web auth page sends the token
    // back to the desktop app via day1copilot:// URL scheme
    const separator = url.includes("?") ? "&" : "?";
    const redirectUrl = `${url}${separator}redirect_uri=${encodeURIComponent("day1copilot://auth/callback")}`;
    window.open(redirectUrl, "_blank", "noopener,noreferrer");
    void onAuthenticated; // acknowledged — dismiss is handled by deep link callback
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
      <div
        className="w-full max-w-sm mx-4 rounded-2xl border border-border p-8 bg-card/95 shadow-2xl shadow-black/50"
        style={{ backdropFilter: "blur(24px)", WebkitBackdropFilter: "blur(24px)" }}
        role="dialog"
        aria-modal="true"
        aria-label={t("auth.signIn")}
      >
        {/* Logo / Title */}
        <div className="text-center mb-8">
          <img src="/logo-64.png" alt="Day1" width={48} height={48} className="mx-auto mb-4" />
          <h2 className="text-lg font-semibold text-text-primary mb-1">
            {t("auth.signIn")}
          </h2>
          <p className="text-sm text-text-muted">
            {t("auth.signInDesc", { defaultValue: "Sign in to start using Day1 Copilot" })}
          </p>
        </div>

        {/* Gmail / Google OAuth */}
        <button
          onClick={() => openExternal("https://day1.doctor/auth/login?provider=google")}
          disabled={isLoading}
          className="w-full py-3 rounded-lg font-medium text-sm transition-colors duration-100
            border border-border bg-transparent text-text-primary hover:bg-muted/50
            disabled:opacity-50 disabled:cursor-not-allowed
            focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
            flex items-center justify-center gap-3"
        >
          {/* Google icon */}
          <svg width="18" height="18" viewBox="0 0 24 24" aria-hidden="true">
            <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4" />
            <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853" />
            <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18A11.96 11.96 0 0 0 0 12c0 1.94.46 3.77 1.28 5.4l3.56-2.77.01-.54z" fill="#FBBC05" />
            <path d="M12 4.75c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 1.19 14.97 0 12 0 7.7 0 3.99 2.47 2.18 6.07l3.66 2.84c.87-2.6 3.3-4.16 6.16-4.16z" fill="#EA4335" />
          </svg>
          {t("auth.continueWithGoogle", { defaultValue: "Continue with Google" })}
        </button>

        <div className="flex items-center gap-3 my-5">
          <div className="flex-1 h-px bg-border" />
          <span className="text-text-disabled text-sm">{t("auth.or", { defaultValue: "or" })}</span>
          <div className="flex-1 h-px bg-border" />
        </div>

        {/* Email OTP */}
        <button
          onClick={() => openExternal("https://day1.doctor/auth/login")}
          disabled={isLoading}
          className="w-full py-3 rounded-lg font-medium text-sm transition-colors duration-100
            border border-border bg-transparent text-text-primary hover:bg-muted/50
            disabled:opacity-50 disabled:cursor-not-allowed
            focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50
            flex items-center justify-center gap-3"
        >
          {/* Mail icon */}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor"
            strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true"
            className="text-text-secondary">
            <rect x="2" y="4" width="20" height="16" rx="2" />
            <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7" />
          </svg>
          {t("auth.continueWithEmail", { defaultValue: "Continue with Email" })}
        </button>

        {/* Create account */}
        <p className="text-center mt-6">
          <span className="text-text-muted text-sm">
            {t("auth.noAccount", { defaultValue: "No account?" })}{" "}
          </span>
          <button
            onClick={() => openExternal("https://day1.doctor/auth/register")}
            className="text-accent text-sm hover:underline focus:outline-none
              focus-visible:ring-2 focus-visible:ring-accent/50 rounded"
          >
            {t("auth.createFree", { defaultValue: "Create one for free" })}
          </button>
        </p>
      </div>
    </div>
  );
}
