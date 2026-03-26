import type { LucideIcon } from 'lucide-react';
import {
  AppWindow,
  Braces,
  Clock3,
  Code2,
  FileIcon,
  FileText,
  FolderClosed,
  Globe2,
  ImageIcon,
  Link2,
  Mail,
  Palette,
  ScanText,
  TerminalSquare,
} from 'lucide-react';

export interface ClipboardFilterOption {
  value: string;
  labelKey: string;
  Icon: LucideIcon;
}

export const clipboardFilterOptions: ClipboardFilterOption[] = [
  { value: 'all', labelKey: 'clipboard:contentTypes.allTypes', Icon: AppWindow },
  { value: 'text', labelKey: 'clipboard:contentTypes.allText', Icon: ScanText },
  { value: 'text:plain_text', labelKey: 'clipboard:contentTypes.plainText', Icon: FileText },
  { value: 'text:url', labelKey: 'clipboard:contentTypes.url', Icon: Link2 },
  { value: 'text:ip_address', labelKey: 'clipboard:contentTypes.ipAddress', Icon: Globe2 },
  { value: 'text:email', labelKey: 'clipboard:contentTypes.email', Icon: Mail },
  { value: 'text:color', labelKey: 'clipboard:contentTypes.color', Icon: Palette },
  { value: 'text:code', labelKey: 'clipboard:contentTypes.codeSnippet', Icon: Code2 },
  { value: 'text:command', labelKey: 'clipboard:contentTypes.command', Icon: TerminalSquare },
  { value: 'text:timestamp', labelKey: 'clipboard:contentTypes.timestamp', Icon: Clock3 },
  { value: 'text:json', labelKey: 'clipboard:contentTypes.json', Icon: Braces },
  { value: 'text:markdown', labelKey: 'clipboard:contentTypes.markdown', Icon: FileIcon },
  { value: 'image', labelKey: 'clipboard:contentTypes.image', Icon: ImageIcon },
  { value: 'file', labelKey: 'clipboard:contentTypes.file', Icon: FolderClosed },
];

export function findClipboardFilterOption(value: string) {
  return (
    clipboardFilterOptions.find((option) => option.value === value) ?? clipboardFilterOptions[0]
  );
}
