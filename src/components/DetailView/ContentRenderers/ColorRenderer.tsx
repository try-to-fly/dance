import { Copy, Palette } from 'lucide-react';
import colorConvert from 'color-convert';
import { useClipboardStore } from '../../../stores/clipboardStore';
import { ColorFormats } from '../../../types/clipboard';
import { Button } from '../../ui/button';
import { Badge } from '../../ui/badge';
import { Card, CardContent, CardHeader } from '../../ui/card';

interface ColorRendererProps {
  content: string;
  metadata?: string | null;
}

interface ParsedColorValue {
  rgb: [number, number, number];
  alpha: number;
}

const HEX_SHORT_LENGTH = 3;
const HEX_FULL_LENGTH = 6;

const parseMetadataColorFormats = (metadata?: string | null): ColorFormats | null => {
  if (!metadata) {
    return null;
  }

  try {
    const parsed = JSON.parse(metadata) as unknown;
    if (!parsed || typeof parsed !== 'object') {
      return null;
    }

    const normalized = parsed as {
      color_formats?: ColorFormats;
      kind?: string;
      data?: ColorFormats;
      hex?: string;
      rgb?: string;
      rgba?: string;
      hsl?: string;
    };

    if (normalized.color_formats) {
      return normalized.color_formats;
    }

    if (normalized.kind === 'color' && normalized.data) {
      return normalized.data;
    }

    if (normalized.hex || normalized.rgb || normalized.rgba || normalized.hsl) {
      return {
        hex: normalized.hex,
        rgb: normalized.rgb,
        rgba: normalized.rgba,
        hsl: normalized.hsl,
      };
    }

    return null;
  } catch {
    return null;
  }
};

const normalizeHex = (value: string): string | null => {
  const normalized = value.trim().replace(/^#/, '');
  if (normalized.length === HEX_SHORT_LENGTH) {
    return normalized
      .split('')
      .map((char) => `${char}${char}`)
      .join('');
  }

  if (normalized.length === HEX_FULL_LENGTH) {
    return normalized;
  }

  return null;
};

const parseRgbLikeValue = (value: string): ParsedColorValue | null => {
  const matches = value.match(/[\d.]+/g);
  if (!matches || matches.length < 3) {
    return null;
  }

  const rgb: [number, number, number] = [
    Number.parseInt(matches[0], 10),
    Number.parseInt(matches[1], 10),
    Number.parseInt(matches[2], 10),
  ];

  return {
    rgb,
    alpha: matches[3] ? Number.parseFloat(matches[3]) : 1,
  };
};

const parseHslValue = (value: string): ParsedColorValue | null => {
  const matches = value.match(/[\d.]+/g);
  if (!matches || matches.length < 3) {
    return null;
  }

  const hsl: [number, number, number] = [
    Number.parseInt(matches[0], 10),
    Number.parseInt(matches[1], 10),
    Number.parseInt(matches[2], 10),
  ];

  return {
    rgb: colorConvert.hsl.rgb(hsl) as [number, number, number],
    alpha: matches[3] ? Number.parseFloat(matches[3]) : 1,
  };
};

const parseColorValue = (value: string): ParsedColorValue | null => {
  const trimmed = value.trim();
  if (!trimmed) {
    return null;
  }

  if (trimmed.startsWith('#')) {
    const normalizedHex = normalizeHex(trimmed);
    if (!normalizedHex) {
      return null;
    }

    return {
      rgb: colorConvert.hex.rgb(normalizedHex) as [number, number, number],
      alpha: 1,
    };
  }

  if (trimmed.startsWith('rgba(') || trimmed.startsWith('rgb(')) {
    return parseRgbLikeValue(trimmed);
  }

  if (trimmed.startsWith('hsl(') || trimmed.startsWith('hsla(')) {
    return parseHslValue(trimmed);
  }

  return null;
};

const formatAlpha = (value: number): string => {
  if (Number.isNaN(value)) {
    return '1';
  }

  return Number.isInteger(value)
    ? String(value)
    : value.toFixed(2).replace(/0+$/, '').replace(/\.$/, '');
};

const buildColorPresentation = (content: string, metadata?: string | null) => {
  const metadataFormats = parseMetadataColorFormats(metadata);
  const parseSource =
    metadataFormats?.rgba ||
    metadataFormats?.rgb ||
    metadataFormats?.hex ||
    metadataFormats?.hsl ||
    content;
  const parsed = parseColorValue(parseSource) ?? {
    rgb: [0, 0, 0] as [number, number, number],
    alpha: 1,
  };
  const [red, green, blue] = parsed.rgb;
  const hsl = colorConvert.rgb.hsl(parsed.rgb);

  return {
    rgbValues: parsed.rgb,
    swatchColor:
      metadataFormats?.hex ||
      metadataFormats?.rgba ||
      metadataFormats?.rgb ||
      metadataFormats?.hsl ||
      content,
    formats: {
      hex: metadataFormats?.hex ?? `#${colorConvert.rgb.hex(parsed.rgb)}`,
      rgb: metadataFormats?.rgb ?? `rgb(${red}, ${green}, ${blue})`,
      rgba:
        metadataFormats?.rgba ?? `rgba(${red}, ${green}, ${blue}, ${formatAlpha(parsed.alpha)})`,
      hsl: metadataFormats?.hsl ?? `hsl(${hsl[0]}, ${hsl[1]}%, ${hsl[2]}%)`,
    },
  };
};

const getContrastColor = ([red, green, blue]: [number, number, number]) => {
  const brightness = (red * 299 + green * 587 + blue * 114) / 1000;
  return brightness > 128 ? '#000000' : '#ffffff';
};

export function ColorRenderer({ content, metadata }: ColorRendererProps) {
  const { copyToClipboard } = useClipboardStore();
  const { formats, rgbValues, swatchColor } = buildColorPresentation(content, metadata);

  const handleCopy = async (text: string) => {
    await copyToClipboard(text);
  };

  const orderedFormats = [
    { copyLabel: 'HEX', displayLabel: 'HEX:', value: formats.hex },
    { copyLabel: 'RGB', displayLabel: 'RGB:', value: formats.rgb },
    { copyLabel: 'RGBA', displayLabel: 'RGBA:', value: formats.rgba },
    { copyLabel: 'HSL', displayLabel: 'HSL:', value: formats.hsl },
  ];

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Palette className="w-4 h-4" />
              <Badge variant="secondary">颜色值</Badge>
            </div>
            <Button onClick={() => handleCopy(content)} size="sm" variant="outline">
              <Copy className="w-4 h-4 mr-2" />
              复制原始值
            </Button>
          </div>
        </CardHeader>

        <CardContent className="space-y-4">
          <div className="flex items-center gap-4">
            <div
              data-testid="color-swatch"
              className="w-24 h-24 rounded-lg border-2 border-muted flex items-center justify-center shadow-sm"
              style={{
                backgroundColor: swatchColor,
                color: getContrastColor(rgbValues),
              }}
            >
              <Palette className="w-8 h-8" />
            </div>
            <div className="flex-1">
              <div className="text-sm font-medium text-muted-foreground mb-2">RGB 值:</div>
              <div className="text-2xl font-mono">
                {rgbValues[0]}, {rgbValues[1]}, {rgbValues[2]}
              </div>
            </div>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            {orderedFormats.map((item) => (
              <div
                key={item.copyLabel}
                data-format-label={item.copyLabel}
                data-testid="color-format-row"
                className="space-y-2"
              >
                <span className="text-sm font-medium text-muted-foreground">
                  {item.displayLabel}
                </span>
                <div className="flex items-center gap-2">
                  <code
                    data-testid={`color-format-${item.copyLabel.toLowerCase()}`}
                    className="flex-1 p-2 bg-muted rounded font-mono text-sm"
                  >
                    {item.value}
                  </code>
                  <Button
                    aria-label={`复制 ${item.copyLabel}`}
                    title={`复制 ${item.copyLabel}`}
                    onClick={() => handleCopy(item.value)}
                    size="sm"
                    variant="ghost"
                    className="h-8 w-8 p-0"
                  >
                    <Copy className="w-3 h-3" />
                  </Button>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
