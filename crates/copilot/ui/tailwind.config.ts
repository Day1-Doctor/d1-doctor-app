import type { Config } from "tailwindcss";

export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        background: "#050505",
        card: "#0D0D0D",
        muted: "#1F1F1F",
        border: "#242424",
        "text-primary": "#E5E5E5",
        "text-secondary": "#A0A0A0",
        "text-muted": "#878787",
        "text-disabled": "#555555",
        accent: "#F97316",
        "accent-hover": "#EA580C",
        success: "#22C55E",
        error: "#EF4444",
        warning: "#F59E0B",
      },
      fontFamily: {
        mono: [
          "Geist Mono",
          "SF Mono",
          "Fira Code",
          "ui-monospace",
          "monospace",
        ],
      },
    },
  },
  plugins: [],
} satisfies Config;
