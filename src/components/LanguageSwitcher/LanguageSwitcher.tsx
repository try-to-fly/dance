import React from 'react';
import { useTranslation } from 'react-i18next';
import { Globe } from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '../ui/dropdown-menu';
import { Button, type ButtonProps } from '../ui/button';
import { useConfigStore } from '../../stores/configStore';

const languages = [
  { code: 'zh', name: '中文', nativeName: '简体中文' },
  { code: 'en', name: 'English', nativeName: 'English' },
  { code: 'ja', name: '日本語', nativeName: '日本語' },
  { code: 'es', name: 'Español', nativeName: 'Español' },
  { code: 'fr', name: 'Français', nativeName: 'Français' },
  { code: 'de', name: 'Deutsch', nativeName: 'Deutsch' },
  { code: 'ko', name: '한국어', nativeName: '한국어' },
  { code: 'pt', name: 'Português', nativeName: 'Português' },
  { code: 'ru', name: 'Русский', nativeName: 'Русский' },
  { code: 'it', name: 'Italiano', nativeName: 'Italiano' },
];

interface LanguageSwitcherProps {
  buttonClassName?: string;
  variant?: ButtonProps['variant'];
  size?: ButtonProps['size'];
}

export const LanguageSwitcher: React.FC<LanguageSwitcherProps> = ({
  buttonClassName,
  variant = 'ghost',
  size = 'icon',
}) => {
  const { i18n } = useTranslation();
  const { config, updateConfig } = useConfigStore();

  const handleLanguageChange = async (languageCode: string) => {
    // Change i18n language
    await i18n.changeLanguage(languageCode);

    // Save to config if config is loaded
    if (config) {
      await updateConfig({
        ...config,
        language: languageCode,
      });
    }
  };

  const currentLanguage = languages.find((lang) => lang.code === i18n.language) || languages[0];

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant={variant}
          size={size}
          aria-label="Change language"
          className={buttonClassName}
        >
          <Globe className="h-4 w-4" />
          <span className="sr-only">Change language</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        {languages.map((lang) => (
          <DropdownMenuItem
            key={lang.code}
            onClick={() => handleLanguageChange(lang.code)}
            className={lang.code === currentLanguage.code ? 'bg-accent' : ''}
          >
            <span className="text-sm">{lang.nativeName}</span>
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
