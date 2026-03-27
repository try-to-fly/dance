import { useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useConfigStore } from '../../stores/configStore';

export const MenuEventHandler: React.FC = () => {
  const { setShowPreferences } = useConfigStore();

  useEffect(() => {
    const setupListeners = async () => {
      // Listen for monitoring toggle updates
      const unlistenMonitoring = await listen('monitoring_toggled', (event) => {
        const isMonitoringNow = event.payload as boolean;
        console.log('Monitoring toggled:', isMonitoringNow);
      });

      // Listen for history cleared event
      const unlistenHistory = await listen('history_cleared', () => {
        console.log('History cleared from menu');
        // The clipboard store will automatically refresh
      });

      // Listen for preferences event
      const unlistenPreferences = await listen('show_preferences', () => {
        setShowPreferences(true);
      });

      // Listen for global shortcut events
      const unlistenGlobalShortcut = await listen('global-shortcut', (_event) => {
        // Show/focus the main window when global shortcut is pressed
        window.focus();
        window.scrollTo(0, 0);
      });

      return () => {
        unlistenMonitoring();
        unlistenHistory();
        unlistenPreferences();
        unlistenGlobalShortcut();
      };
    };

    setupListeners();
  }, []);

  return null;
};
