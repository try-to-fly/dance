import * as Select from '@radix-ui/react-select';
import { ChevronDown, Check } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useClipboardStore } from '../../stores/clipboardStore';
import { clipboardFilterOptions, findClipboardFilterOption } from '../../lib/clipboardFilters';
import { cn } from '../../lib/utils';

interface TypeFilterProps {
  compact?: boolean;
  className?: string;
}

export function TypeFilter({ compact = false, className }: TypeFilterProps) {
  const { t } = useTranslation('clipboard');
  const { selectedType, setSelectedType } = useClipboardStore();

  const currentOption = findClipboardFilterOption(selectedType);

  return (
    <div className={cn(compact ? 'min-w-0' : 'space-y-1.5', className)}>
      {!compact && (
        <h3 className="text-[11px] font-medium uppercase tracking-[0.22em] text-muted-foreground min-[1200px]:text-xs min-[1200px]:tracking-[0.24em]">
          {t('contentTypes.allTypes')}
        </h3>
      )}

      <Select.Root value={selectedType} onValueChange={setSelectedType}>
        <Select.Trigger
          className={cn(
            'flex w-full items-center justify-between border border-border/70 bg-background/80 px-3 text-sm text-foreground shadow-sm transition-colors hover:bg-accent/60 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2',
            compact
              ? 'h-7 rounded-[10px] px-2.5 pr-2 text-[12px] min-[1200px]:h-[30px]'
              : 'h-10 rounded-xl min-[1200px]:h-11 min-[1200px]:rounded-2xl'
          )}
        >
          <Select.Value>
            <span className="flex min-w-0 items-center gap-2">
              <currentOption.Icon className="h-4 w-4 text-muted-foreground" />
              <span className="truncate">
                {t(currentOption.labelKey.replace('clipboard:', ''))}
              </span>
            </span>
          </Select.Value>
          <Select.Icon className="text-muted-foreground">
            <ChevronDown size={16} />
          </Select.Icon>
        </Select.Trigger>

        <Select.Portal>
          <Select.Content
            className="z-50 overflow-hidden rounded-xl border border-border/70 bg-popover/95 shadow-[0_16px_40px_rgba(15,23,42,0.16)] backdrop-blur-xl min-[1200px]:rounded-2xl"
            position="popper"
            sideOffset={8}
          >
            <Select.Viewport className="p-2">
              {clipboardFilterOptions.map((option) => (
                <Select.Item
                  key={option.value}
                  value={option.value}
                  className="relative flex cursor-pointer select-none items-center rounded-lg px-3 py-1.5 text-[13px] text-popover-foreground outline-none transition-colors hover:bg-accent focus:bg-accent min-[1200px]:rounded-xl"
                >
                  <Select.ItemText>
                    <span className="flex items-center gap-2">
                      <option.Icon className="h-4 w-4 text-muted-foreground" />
                      {t(option.labelKey.replace('clipboard:', ''))}
                    </span>
                  </Select.ItemText>
                  <Select.ItemIndicator className="absolute right-3">
                    <Check size={14} />
                  </Select.ItemIndicator>
                </Select.Item>
              ))}
            </Select.Viewport>
          </Select.Content>
        </Select.Portal>
      </Select.Root>
    </div>
  );
}
