import { Settings } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { Button, type ButtonProps } from './ui/button';
import { useConfigStore } from '../stores/configStore';

interface SettingsButtonProps {
  buttonClassName?: string;
  variant?: ButtonProps['variant'];
  size?: ButtonProps['size'];
}

export function SettingsButton({
  buttonClassName,
  variant = 'outline',
  size = 'icon',
}: SettingsButtonProps) {
  const { t } = useTranslation(['common']);
  const { setShowPreferences } = useConfigStore();

  const handleOpenSettings = () => {
    setShowPreferences(true);
  };

  return (
    <Button
      variant={variant}
      size={size}
      onClick={handleOpenSettings}
      aria-label={t('settings.open')}
      className={buttonClassName}
    >
      <Settings className="h-[1.2rem] w-[1.2rem]" />
      <span className="sr-only">{t('settings.open')}</span>
    </Button>
  );
}
