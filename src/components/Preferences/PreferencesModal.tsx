import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '../ui/dialog';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../ui/tabs';
import { Switch } from '../ui/switch';
import { Slider } from '../ui/slider';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '../ui/select';
import { Button } from '../ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '../ui/card';
import { Label } from '../ui/label';
import { RadioGroup, RadioGroupItem } from '../ui/radio-group';
import {
  Settings,
  Keyboard,
  Shield,
  Power,
  Sun,
  Moon,
  Monitor,
  Plus,
  X,
  CheckCircle,
  Globe,
  BarChart3,
  FileText,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from '../../stores/configStore';
import { ShortcutRecorder } from './ShortcutRecorder';
import * as Toast from '@radix-ui/react-toast';
import { analytics } from '../../services/analytics';
import { getSystemLanguage } from '../../i18n/config';
import { LogViewer } from '../LogViewer/LogViewer';
import { useTheme } from '../theme-provider';
import { cn } from '../../lib/utils';

type AppTheme = 'dark' | 'light' | 'system';

export function PreferencesModal() {
  const { t, i18n } = useTranslation(['preferences', 'common']);
  const { theme, setTheme } = useTheme();
  const {
    config,
    cacheStats,
    loading,
    showPreferences,
    cacheStatsLoading,
    cacheStatsError,
    loadConfig,
    updateConfig,
    loadCacheStatistics,
    registerGlobalShortcut,
    setAutoStartup,
    getAutoStartupStatus,
    setShowPreferences,
    formatBytes,
    getExpiryDisplayValue,
    createExpiryOption,
  } = useConfigStore();

  const [localConfig, setLocalConfig] = useState(config);
  const [autoStartupEnabled, setAutoStartupEnabled] = useState(false);
  const [autoUpdateEnabled, setAutoUpdateEnabled] = useState(config?.auto_update ?? true);
  const [shortcutError, setShortcutError] = useState<string | null>(null);
  const [availableApps, setAvailableApps] = useState<{ name: string; bundle_id: string }[]>([]);
  const [updateCheckLoading, setUpdateCheckLoading] = useState(false);
  const [showUpdateToast, setShowUpdateToast] = useState(false);
  const [updateToastMessage, setUpdateToastMessage] = useState('');
  const [updateToastType, setUpdateToastType] = useState<'success' | 'error'>('success');
  const [analyticsEnabled, setAnalyticsEnabled] = useState(analytics.isEnabled());
  const [selectedLanguage, setSelectedLanguage] = useState(() => {
    return config?.language || i18n.language || getSystemLanguage();
  });
  const [selectedTheme, setSelectedTheme] = useState<AppTheme>(theme);
  const [showLogViewer, setShowLogViewer] = useState(false);

  useEffect(() => {
    if (showPreferences) {
      if (!config) {
        loadConfig();
      }
      // Always refresh cache statistics when preferences open
      loadCacheStatistics();
    }
  }, [showPreferences, config, loadConfig, loadCacheStatistics]);

  useEffect(() => {
    if (config) {
      setLocalConfig(config);
      setAutoUpdateEnabled(config.auto_update ?? true);
      // Set language selection based on config, defaulting to system if not set
      setSelectedLanguage(config.language || 'system');
    }
  }, [config]);

  useEffect(() => {
    if (showPreferences) {
      getAutoStartupStatus().then(setAutoStartupEnabled);
      // Load available applications
      loadAvailableApps();
    }
  }, [showPreferences, getAutoStartupStatus]);

  useEffect(() => {
    setSelectedTheme(theme);
  }, [theme, showPreferences]);

  const loadAvailableApps = async () => {
    try {
      const apps = await invoke<{ name: string; bundle_id: string }[]>(
        'get_installed_applications'
      );
      console.log('Loaded apps:', apps.length);
      setAvailableApps(apps);
    } catch (error) {
      console.error('Failed to load applications:', error);
    }
  };

  const validateShortcut = async (shortcut: string): Promise<boolean> => {
    try {
      return await invoke<boolean>('validate_shortcut', { shortcut });
    } catch (error) {
      console.error('Failed to validate shortcut:', error);
      return false;
    }
  };

  const handleSave = async () => {
    if (!localConfig) return;

    try {
      // Update config with auto_update and language settings
      const updatedConfig = {
        ...localConfig,
        auto_update: autoUpdateEnabled,
        language: selectedLanguage,
      };

      await updateConfig(updatedConfig);

      // Update global shortcut if changed
      if (config && localConfig.global_shortcut !== config.global_shortcut) {
        try {
          await registerGlobalShortcut(localConfig.global_shortcut);
          setShortcutError(null);
        } catch (error) {
          setShortcutError(String(error));
          return;
        }
      }

      // Update auto startup if changed
      if (autoStartupEnabled !== (config?.auto_startup || false)) {
        await setAutoStartup(autoStartupEnabled);
      }

      // Update language if changed
      const targetLanguage = selectedLanguage === 'system' ? getSystemLanguage() : selectedLanguage;
      if (targetLanguage !== i18n.language) {
        await i18n.changeLanguage(targetLanguage);
      }

      if (selectedTheme !== theme) {
        setTheme(selectedTheme);
      }

      setShowPreferences(false);
    } catch (error) {
      console.error('Failed to save preferences:', error);
    }
  };

  const handleCancel = () => {
    setLocalConfig(config);
    setSelectedLanguage(config?.language || 'system');
    setSelectedTheme(theme);
    setShortcutError(null);
    setShowPreferences(false);
  };

  if (!localConfig) {
    return null;
  }

  return (
    <>
      <Toast.Provider swipeDirection="right">
        <Dialog open={showPreferences} onOpenChange={setShowPreferences}>
          <DialogContent className="max-w-3xl h-[70vh] flex flex-col p-0">
            <DialogHeader className="px-6 pt-6 pb-4 flex-shrink-0">
              <DialogTitle className="flex items-center gap-2">
                <Settings className="w-5 h-5" />
                {t('title')}
              </DialogTitle>
            </DialogHeader>

            <div className="flex-1 flex flex-col min-h-0 px-6">
              <Tabs defaultValue="text" className="w-full flex-1 flex flex-col min-h-0">
                <TabsList className="grid w-full grid-cols-8 mb-4 flex-shrink-0">
                  <TabsTrigger value="text">{t('tabs.text')}</TabsTrigger>
                  <TabsTrigger value="image">{t('tabs.image')}</TabsTrigger>
                  <TabsTrigger value="security" className="flex items-center gap-1">
                    <Shield className="w-4 h-4" />
                    {t('tabs.security')}
                  </TabsTrigger>
                  <TabsTrigger value="shortcuts" className="flex items-center gap-1">
                    <Keyboard className="w-4 h-4" />
                    {t('tabs.shortcuts')}
                  </TabsTrigger>
                  <TabsTrigger value="system" className="flex items-center gap-1">
                    <Power className="w-4 h-4" />
                    {t('tabs.system')}
                  </TabsTrigger>
                  <TabsTrigger value="language" className="flex items-center gap-1">
                    <Globe className="w-4 h-4" />
                    {t('tabs.language')}
                  </TabsTrigger>
                  <TabsTrigger value="analytics" className="flex items-center gap-1">
                    <BarChart3 className="w-4 h-4" />
                    {t('tabs.analytics')}
                  </TabsTrigger>
                  <TabsTrigger value="logs" className="flex items-center gap-1">
                    <FileText className="w-4 h-4" />
                    {t('tabs.logs')}
                  </TabsTrigger>
                </TabsList>

                <TabsContent
                  value="text"
                  className="flex-1 overflow-y-auto pr-2 data-[state=active]:flex data-[state=active]:flex-col"
                >
                  <div className="space-y-6 pb-4">
                    <div>
                      <h3 className="text-lg font-semibold mb-4">{t('text.title')}</h3>
                      <div className="space-y-4">
                        <div className="space-y-2">
                          <Label className="text-sm font-medium">
                            {t('text.maxSizeValue', { value: localConfig.text.max_size_mb })}
                          </Label>
                          <Slider
                            value={[localConfig.text.max_size_mb]}
                            onValueChange={([value]: number[]) =>
                              setLocalConfig((prev) =>
                                prev
                                  ? {
                                      ...prev,
                                      text: { ...prev.text, max_size_mb: value },
                                    }
                                  : prev
                              )
                            }
                            min={0.1}
                            max={10}
                            step={0.1}
                            className="w-full"
                          />
                          <p className="text-sm text-muted-foreground">
                            {t('text.maxSizeDescription')}
                          </p>
                        </div>

                        <div className="space-y-3">
                          <Label className="text-sm font-medium">{t('text.expiration')}</Label>
                          <RadioGroup
                            value={getExpiryDisplayValue(localConfig.text.expiry)}
                            onValueChange={(value) => {
                              setLocalConfig((prev) =>
                                prev
                                  ? {
                                      ...prev,
                                      text: {
                                        ...prev.text,
                                        expiry: createExpiryOption(value),
                                      },
                                    }
                                  : prev
                              );
                            }}
                            className="grid grid-cols-2 gap-4"
                          >
                            {[
                              { value: '7', label: t('text.expirationOptions.7days') },
                              { value: '14', label: t('text.expirationOptions.14days') },
                              { value: '30', label: t('text.expirationOptions.30days') },
                              { value: 'never', label: t('text.expirationOptions.never') },
                            ].map((option) => (
                              <div key={option.value} className="flex items-center space-x-2">
                                <RadioGroupItem
                                  value={option.value}
                                  id={`text-expiry-${option.value}`}
                                />
                                <Label
                                  htmlFor={`text-expiry-${option.value}`}
                                  className="text-sm font-normal"
                                >
                                  {option.label}
                                </Label>
                              </div>
                            ))}
                          </RadioGroup>
                        </div>
                      </div>
                    </div>
                  </div>
                </TabsContent>

                <TabsContent
                  value="image"
                  className="flex-1 overflow-y-auto pr-2 data-[state=active]:flex data-[state=active]:flex-col"
                >
                  <div className="space-y-6 pb-4">
                    <div>
                      <h3 className="text-lg font-semibold mb-4">{t('image.title')}</h3>
                      <div className="space-y-3">
                        <Label className="text-sm font-medium">{t('image.expiration')}</Label>
                        <RadioGroup
                          value={getExpiryDisplayValue(localConfig.image.expiry)}
                          onValueChange={(value) => {
                            setLocalConfig((prev) =>
                              prev
                                ? {
                                    ...prev,
                                    image: {
                                      ...prev.image,
                                      expiry: createExpiryOption(value),
                                    },
                                  }
                                : prev
                            );
                          }}
                          className="grid grid-cols-2 gap-4"
                        >
                          {[
                            { value: '7', label: t('image.expirationOptions.7days') },
                            { value: '14', label: t('image.expirationOptions.14days') },
                            { value: '30', label: t('image.expirationOptions.30days') },
                            { value: 'never', label: t('image.expirationOptions.never') },
                          ].map((option) => (
                            <div key={option.value} className="flex items-center space-x-2">
                              <RadioGroupItem
                                value={option.value}
                                id={`image-expiry-${option.value}`}
                              />
                              <Label
                                htmlFor={`image-expiry-${option.value}`}
                                className="text-sm font-normal"
                              >
                                {option.label}
                              </Label>
                            </div>
                          ))}
                        </RadioGroup>
                      </div>
                    </div>
                  </div>
                </TabsContent>

                <TabsContent
                  value="security"
                  className="flex-1 overflow-y-auto pr-2 data-[state=active]:flex data-[state=active]:flex-col"
                >
                  <div className="space-y-6 pb-4">
                    <div>
                      <h3 className="text-lg font-semibold mb-4">{t('security.excludedApps')}</h3>
                      <p className="text-sm text-muted-foreground mb-4">
                        {t('security.excludedAppsDescription')}
                      </p>

                      <div className="space-y-3">
                        {localConfig.excluded_apps_v2?.map((excludedApp, index) => (
                          <div
                            key={index}
                            className="flex items-center justify-between p-3 bg-secondary rounded-lg border"
                          >
                            <div className="flex flex-col">
                              <span className="font-medium text-sm">{excludedApp.name}</span>
                              <span className="text-xs text-muted-foreground">
                                {excludedApp.bundle_id}
                              </span>
                            </div>
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() =>
                                setLocalConfig((prev) =>
                                  prev
                                    ? {
                                        ...prev,
                                        excluded_apps_v2:
                                          prev.excluded_apps_v2?.filter((_, i) => i !== index) ||
                                          [],
                                      }
                                    : prev
                                )
                              }
                              className="h-8 w-8 p-0 text-destructive hover:text-destructive"
                            >
                              <X className="h-4 w-4" />
                            </Button>
                          </div>
                        ))}

                        <div className="space-y-2">
                          <Label className="text-sm font-medium flex items-center gap-2">
                            <Plus className="h-4 w-4" />
                            {t('security.addApp')}
                          </Label>
                          <Select
                            value=""
                            onValueChange={(value) => {
                              if (value) {
                                const selectedApp = availableApps.find(
                                  (app) => app.bundle_id === value
                                );
                                if (selectedApp) {
                                  setLocalConfig((prev) =>
                                    prev
                                      ? {
                                          ...prev,
                                          excluded_apps_v2: [
                                            ...(prev.excluded_apps_v2 || []),
                                            {
                                              name: selectedApp.name,
                                              bundle_id: selectedApp.bundle_id,
                                            },
                                          ],
                                        }
                                      : prev
                                  );
                                }
                              }
                            }}
                          >
                            <SelectTrigger>
                              <SelectValue placeholder={t('security.selectApp')} />
                            </SelectTrigger>
                            <SelectContent>
                              {availableApps
                                .filter(
                                  (app) =>
                                    !localConfig.excluded_apps_v2?.some(
                                      (excluded) => excluded.bundle_id === app.bundle_id
                                    )
                                )
                                .sort((a, b) => a.name.localeCompare(b.name))
                                .map((app) => (
                                  <SelectItem key={app.bundle_id} value={app.bundle_id}>
                                    {app.name}
                                  </SelectItem>
                                ))}
                            </SelectContent>
                          </Select>
                        </div>
                      </div>
                    </div>
                  </div>
                </TabsContent>

                <TabsContent
                  value="shortcuts"
                  className="flex-1 overflow-y-auto pr-2 data-[state=active]:flex data-[state=active]:flex-col"
                >
                  <div className="space-y-6 pb-4">
                    <div>
                      <h3 className="text-lg font-semibold mb-4">{t('shortcuts.title')}</h3>
                      <div className="space-y-3">
                        <Label className="text-sm font-medium">
                          {t('shortcuts.activationShortcut')}
                        </Label>
                        <ShortcutRecorder
                          value={localConfig.global_shortcut}
                          onChange={(shortcut) =>
                            setLocalConfig((prev) =>
                              prev
                                ? {
                                    ...prev,
                                    global_shortcut: shortcut,
                                  }
                                : prev
                            )
                          }
                          onValidate={validateShortcut}
                        />
                        {shortcutError && (
                          <p className="text-sm text-destructive">{shortcutError}</p>
                        )}
                      </div>
                    </div>
                  </div>
                </TabsContent>

                <TabsContent
                  value="system"
                  className="flex-1 overflow-y-auto pr-2 data-[state=active]:flex data-[state=active]:flex-col min-h-0"
                >
                  <div className="space-y-6 pb-4">
                    <div className="space-y-4">
                      <h3 className="text-lg font-semibold">{t('common:theme.toggle')}</h3>
                      <RadioGroup
                        value={selectedTheme}
                        onValueChange={(value) => setSelectedTheme(value as AppTheme)}
                        className="grid grid-cols-1 gap-3 md:grid-cols-3"
                      >
                        {[
                          {
                            value: 'light',
                            label: t('common:theme.light'),
                            Icon: Sun,
                          },
                          {
                            value: 'dark',
                            label: t('common:theme.dark'),
                            Icon: Moon,
                          },
                          {
                            value: 'system',
                            label: t('common:theme.system'),
                            Icon: Monitor,
                          },
                        ].map((option) => (
                          <Label
                            key={option.value}
                            htmlFor={`theme-${option.value}`}
                            className={cn(
                              'flex cursor-pointer items-center gap-3 rounded-xl border px-4 py-3 transition-colors',
                              selectedTheme === option.value
                                ? 'border-primary/30 bg-primary/10 text-foreground'
                                : 'border-border bg-background/60 text-muted-foreground hover:bg-accent/40'
                            )}
                          >
                            <RadioGroupItem value={option.value} id={`theme-${option.value}`} />
                            <option.Icon className="h-4 w-4" />
                            <span className="text-sm font-medium">{option.label}</span>
                          </Label>
                        ))}
                      </RadioGroup>
                    </div>

                    <div className="space-y-4">
                      <h3 className="text-lg font-semibold">{t('system.startup.title')}</h3>
                      <div className="flex items-center space-x-2">
                        <Switch
                          checked={autoStartupEnabled}
                          onCheckedChange={setAutoStartupEnabled}
                        />
                        <Label className="text-sm font-medium">
                          {t('system.startup.runAtStartup')}
                        </Label>
                      </div>
                    </div>

                    <div className="space-y-4">
                      <h3 className="text-lg font-semibold">{t('system.update.title')}</h3>
                      <div className="space-y-3">
                        <div className="flex items-center space-x-2">
                          <Switch
                            checked={autoUpdateEnabled}
                            onCheckedChange={setAutoUpdateEnabled}
                          />
                          <Label className="text-sm font-medium">
                            {t('system.update.autoCheck')}
                          </Label>
                        </div>
                        <p className="text-sm text-muted-foreground">
                          {t('system.update.autoCheckDescription')}
                        </p>
                        <Button
                          variant="secondary"
                          onClick={async () => {
                            const { invoke } = await import('@tauri-apps/api/core');
                            const { ask, message } = await import('@tauri-apps/plugin-dialog');

                            setUpdateCheckLoading(true);
                            try {
                              console.log('Starting manual update check...');
                              const { getVersion } = await import('@tauri-apps/api/app');
                              const currentVersion = await getVersion();
                              console.log('Current version:', currentVersion);
                              const updateInfo = await invoke<any>('check_for_update');
                              console.log('Update check result:', updateInfo);

                              if (updateInfo.available === true) {
                                console.log('Update available, showing dialog');
                                const yes = await ask(
                                  `${t('system.update.newVersionAvailable', { version: updateInfo.version })}!\n\n${updateInfo.notes || t('system.update.updateNotes')}\n\n是否立即更新？`,
                                  {
                                    title: t('system.update.updateTitle'),
                                    okLabel: t('system.update.updateNow'),
                                    cancelLabel: t('system.update.later'),
                                  }
                                );
                                if (yes) {
                                  try {
                                    await invoke('install_update');
                                    // App will restart automatically after update
                                  } catch (installError) {
                                    console.error('Failed to install update:', installError);
                                    await message(t('system.update.installFailed'), {
                                      title: t('system.update.updateError'),
                                    });
                                  }
                                }
                              } else {
                                console.log('No updates available, showing toast');
                                setUpdateToastMessage(
                                  t('system.update.upToDate', { version: currentVersion })
                                );
                                setUpdateToastType('success');
                                setShowUpdateToast(true);
                              }
                            } catch (error) {
                              console.error('Failed to check for updates:', error);
                              const errorMessage =
                                typeof error === 'string' ? error : t('system.update.networkError');
                              setUpdateToastMessage(
                                t('system.update.checkError', { error: errorMessage })
                              );
                              setUpdateToastType('error');
                              setShowUpdateToast(true);
                            } finally {
                              setUpdateCheckLoading(false);
                            }
                          }}
                          disabled={updateCheckLoading}
                        >
                          {updateCheckLoading
                            ? t('system.update.checking')
                            : t('system.update.checkNow')}
                        </Button>
                      </div>
                    </div>

                    <div className="space-y-4">
                      <h3 className="text-lg font-semibold">{t('system.cache.title')}</h3>

                      {cacheStatsLoading && (
                        <div className="flex items-center justify-center py-8">
                          <div className="text-sm text-muted-foreground">
                            {t('common:loading', 'Loading...')}
                          </div>
                        </div>
                      )}

                      {cacheStatsError && (
                        <Card className="border-destructive/50 bg-destructive/10">
                          <CardContent className="pt-6">
                            <div className="text-sm text-destructive">
                              <p className="font-medium mb-2">
                                {t('system.cache.loadError', 'Failed to load cache statistics')}
                              </p>
                              <p className="text-xs opacity-80">{cacheStatsError}</p>
                              <Button
                                variant="outline"
                                size="sm"
                                className="mt-3"
                                onClick={() => loadCacheStatistics()}
                                disabled={cacheStatsLoading}
                              >
                                {t('common:retry', 'Retry')}
                              </Button>
                            </div>
                          </CardContent>
                        </Card>
                      )}

                      {cacheStats && !cacheStatsLoading && !cacheStatsError && (
                        <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                          <Card>
                            <CardHeader className="pb-2">
                              <CardTitle className="text-sm font-medium">
                                {t('system.cache.databaseSize')}
                              </CardTitle>
                            </CardHeader>
                            <CardContent className="pt-0">
                              <div className="text-2xl font-bold text-primary">
                                {formatBytes(cacheStats.db_size_bytes)}
                              </div>
                            </CardContent>
                          </Card>
                          <Card>
                            <CardHeader className="pb-2">
                              <CardTitle className="text-sm font-medium">
                                {t('system.cache.imageCache')}
                              </CardTitle>
                            </CardHeader>
                            <CardContent className="pt-0">
                              <div className="text-2xl font-bold text-primary">
                                {formatBytes(cacheStats.images_size_bytes)}
                              </div>
                            </CardContent>
                          </Card>
                          <Card>
                            <CardHeader className="pb-2">
                              <CardTitle className="text-sm font-medium">
                                {t('system.cache.totalEntries')}
                              </CardTitle>
                            </CardHeader>
                            <CardContent className="pt-0">
                              <div className="text-2xl font-bold text-primary">
                                {cacheStats.total_entries}
                              </div>
                            </CardContent>
                          </Card>
                          <Card>
                            <CardHeader className="pb-2">
                              <CardTitle className="text-sm font-medium">
                                {t('system.cache.textEntries')}
                              </CardTitle>
                            </CardHeader>
                            <CardContent className="pt-0">
                              <div className="text-2xl font-bold text-primary">
                                {cacheStats.text_entries}
                              </div>
                            </CardContent>
                          </Card>
                          <Card>
                            <CardHeader className="pb-2">
                              <CardTitle className="text-sm font-medium">
                                {t('system.cache.imageEntries')}
                              </CardTitle>
                            </CardHeader>
                            <CardContent className="pt-0">
                              <div className="text-2xl font-bold text-primary">
                                {cacheStats.image_entries}
                              </div>
                            </CardContent>
                          </Card>
                        </div>
                      )}

                      {!cacheStats && !cacheStatsLoading && !cacheStatsError && (
                        <Card>
                          <CardContent className="pt-6">
                            <div className="text-center text-sm text-muted-foreground">
                              <p>{t('system.cache.noData', 'No cache data available')}</p>
                            </div>
                          </CardContent>
                        </Card>
                      )}
                    </div>
                  </div>
                </TabsContent>

                <TabsContent
                  value="language"
                  className="flex-1 overflow-y-auto pr-2 data-[state=active]:flex data-[state=active]:flex-col"
                >
                  <div className="space-y-6 pb-4">
                    <div>
                      <h3 className="text-lg font-semibold mb-4">{t('language.title')}</h3>
                      <p className="text-sm text-muted-foreground mb-6">
                        {t('language.description')}
                      </p>

                      <div className="space-y-3">
                        <Label className="text-sm font-medium">{t('language.title')}</Label>
                        <RadioGroup
                          value={selectedLanguage}
                          onValueChange={setSelectedLanguage}
                          className="grid grid-cols-1 gap-4"
                        >
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="system" id="lang-system" />
                            <Label htmlFor="lang-system" className="text-sm font-normal">
                              {t('language.systemDefault')} (
                              {getSystemLanguage() === 'zh'
                                ? '中文'
                                : getSystemLanguage() === 'ja'
                                  ? '日本語'
                                  : getSystemLanguage() === 'es'
                                    ? 'Español'
                                    : getSystemLanguage() === 'fr'
                                      ? 'Français'
                                      : getSystemLanguage() === 'de'
                                        ? 'Deutsch'
                                        : getSystemLanguage() === 'ko'
                                          ? '한국어'
                                          : getSystemLanguage() === 'pt'
                                            ? 'Português'
                                            : getSystemLanguage() === 'ru'
                                              ? 'Русский'
                                              : getSystemLanguage() === 'it'
                                                ? 'Italiano'
                                                : 'English'}
                              )
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="zh" id="lang-zh" />
                            <Label htmlFor="lang-zh" className="text-sm font-normal">
                              {t('language.chinese')}
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="en" id="lang-en" />
                            <Label htmlFor="lang-en" className="text-sm font-normal">
                              {t('language.english')}
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="ja" id="lang-ja" />
                            <Label htmlFor="lang-ja" className="text-sm font-normal">
                              {t('language.japanese')}
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="es" id="lang-es" />
                            <Label htmlFor="lang-es" className="text-sm font-normal">
                              {t('language.spanish')}
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="fr" id="lang-fr" />
                            <Label htmlFor="lang-fr" className="text-sm font-normal">
                              {t('language.french')}
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="de" id="lang-de" />
                            <Label htmlFor="lang-de" className="text-sm font-normal">
                              {t('language.german')}
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="ko" id="lang-ko" />
                            <Label htmlFor="lang-ko" className="text-sm font-normal">
                              {t('language.korean')}
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="pt" id="lang-pt" />
                            <Label htmlFor="lang-pt" className="text-sm font-normal">
                              {t('language.portuguese')}
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="ru" id="lang-ru" />
                            <Label htmlFor="lang-ru" className="text-sm font-normal">
                              {t('language.russian')}
                            </Label>
                          </div>
                          <div className="flex items-center space-x-2">
                            <RadioGroupItem value="it" id="lang-it" />
                            <Label htmlFor="lang-it" className="text-sm font-normal">
                              {t('language.italian')}
                            </Label>
                          </div>
                        </RadioGroup>
                      </div>
                    </div>
                  </div>
                </TabsContent>

                <TabsContent
                  value="analytics"
                  className="flex-1 overflow-y-auto pr-2 data-[state=active]:flex data-[state=active]:flex-col"
                >
                  <div className="space-y-6 pb-4">
                    <div>
                      <h3 className="text-lg font-semibold mb-4">{t('analytics.title')}</h3>
                      <p className="text-sm text-muted-foreground mb-6">
                        {t('analytics.description')}
                      </p>

                      <div className="space-y-4">
                        <div className="flex items-center space-x-2">
                          <Switch
                            checked={analyticsEnabled}
                            onCheckedChange={(checked) => {
                              setAnalyticsEnabled(checked);
                              analytics.setEnabled(checked);
                            }}
                          />
                          <Label className="text-sm font-medium">{t('analytics.enable')}</Label>
                        </div>

                        <Card className="bg-secondary/50">
                          <CardHeader className="pb-3">
                            <CardTitle className="text-sm font-medium">
                              {t('analytics.dataCollected.title')}
                            </CardTitle>
                          </CardHeader>
                          <CardContent>
                            <ul className="text-sm text-muted-foreground space-y-1">
                              {(
                                t('analytics.dataCollected.items', {
                                  returnObjects: true,
                                }) as string[]
                              ).map((item: string, index: number) => (
                                <li key={index}>• {item}</li>
                              ))}
                            </ul>
                          </CardContent>
                        </Card>

                        <Card className="bg-red-500/10 border-red-500/20">
                          <CardHeader className="pb-3">
                            <CardTitle className="text-sm font-medium text-red-600 dark:text-red-400">
                              {t('analytics.dataNotCollected.title')}
                            </CardTitle>
                          </CardHeader>
                          <CardContent>
                            <ul className="text-sm text-red-600/80 dark:text-red-400/80 space-y-1">
                              {(
                                t('analytics.dataNotCollected.items', {
                                  returnObjects: true,
                                }) as string[]
                              ).map((item: string, index: number) => (
                                <li key={index}>• {item}</li>
                              ))}
                            </ul>
                          </CardContent>
                        </Card>

                        <div className="pt-2">
                          <p className="text-xs text-muted-foreground">
                            {t('analytics.provider').split('<link>')[0]}
                            <a
                              href="https://aptabase.com"
                              target="_blank"
                              rel="noopener noreferrer"
                              className="underline"
                            >
                              Aptabase
                            </a>{' '}
                            {t('analytics.provider').split('</link>')[1]}
                          </p>
                        </div>
                      </div>
                    </div>
                  </div>
                </TabsContent>

                <TabsContent
                  value="logs"
                  className="flex-1 overflow-y-auto pr-2 data-[state=active]:flex data-[state=active]:flex-col"
                >
                  <div className="space-y-6">
                    <div>
                      <h3 className="text-lg font-semibold mb-4">{t('tabs.logs')}</h3>

                      <Card>
                        <CardHeader>
                          <CardTitle className="flex items-center gap-2">
                            <FileText className="w-5 h-5" />
                            {t('logs.title')}
                          </CardTitle>
                        </CardHeader>
                        <CardContent className="space-y-4">
                          <div className="flex items-center justify-between">
                            <div>
                              <h4 className="font-medium">{t('logs.viewLogs')}</h4>
                              <p className="text-sm text-muted-foreground">
                                {t('logs.description')}
                              </p>
                            </div>
                            <Button
                              onClick={() => setShowLogViewer(true)}
                              className="flex items-center gap-2"
                            >
                              <FileText className="w-4 h-4" />
                              {t('logs.openViewer', 'Open Log Viewer')}
                            </Button>
                          </div>

                          <div className="text-sm text-muted-foreground">
                            <p>
                              {t(
                                'logs.storageInfo',
                                'Log files are stored in the system log directory and contain information about:'
                              )}
                            </p>
                            <ul className="list-disc list-inside mt-2 space-y-1">
                              {(
                                (t('logs.features', { returnObjects: true }) as string[]) || [
                                  'Clipboard monitoring activities',
                                  'Error messages and warnings',
                                  'Performance information',
                                  'System events and user actions',
                                ]
                              ).map((feature: string, index: number) => (
                                <li key={index}>{feature}</li>
                              ))}
                            </ul>
                          </div>
                        </CardContent>
                      </Card>
                    </div>
                  </div>
                </TabsContent>
              </Tabs>

              <div className="flex justify-end gap-3 mt-4 pt-4 border-t flex-shrink-0 pb-6">
                <Button variant="secondary" onClick={handleCancel}>
                  {t('common:cancel')}
                </Button>
                <Button onClick={handleSave} disabled={loading}>
                  {loading ? t('common:saving') : t('common:save')}
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>

        <Toast.Root
          className="bg-card border border-border rounded-md shadow-lg p-4 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2"
          open={showUpdateToast}
          onOpenChange={setShowUpdateToast}
          duration={4000}
        >
          <Toast.Title className="flex items-center gap-2 text-sm font-semibold text-card-foreground">
            {updateToastType === 'success' ? (
              <CheckCircle className="h-4 w-4 text-green-600 dark:text-green-400" />
            ) : (
              <X className="h-4 w-4 text-red-600 dark:text-red-400" />
            )}
            {updateToastType === 'success'
              ? t('system.update.checkComplete')
              : t('system.update.checkFailed')}
          </Toast.Title>
          <Toast.Description className="text-sm text-muted-foreground mt-1">
            {updateToastMessage}
          </Toast.Description>
          <Toast.Close className="absolute right-2 top-2 rounded-md p-1 text-muted-foreground hover:text-card-foreground transition-colors">
            <X className="h-3 w-3" />
          </Toast.Close>
        </Toast.Root>

        <Toast.Viewport className="fixed top-4 right-4 flex flex-col p-6 gap-2 w-96 max-w-[100vw] m-0 list-none z-[2147483647] outline-none" />
      </Toast.Provider>

      <LogViewer isOpen={showLogViewer} onClose={() => setShowLogViewer(false)} />
    </>
  );
}
