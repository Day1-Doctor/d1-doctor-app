import { useTranslation } from "react-i18next";

export function LanguageToggle() {
  const { i18n } = useTranslation();
  const isZh = i18n.language.startsWith("zh");

  const toggle = () => {
    const newLang = isZh ? "en" : "zh";
    i18n.changeLanguage(newLang);
  };

  return (
    <button
      onClick={toggle}
      className="px-2 py-1 rounded text-sm text-text-secondary hover:text-text-primary
        hover:bg-muted transition-colors duration-100
        focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50"
      title={isZh ? "Switch to English" : "切换到中文"}
      aria-label={isZh ? "Switch to English" : "Switch to Chinese"}
    >
      {isZh ? "EN" : "中"}
    </button>
  );
}
