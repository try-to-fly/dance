import * as monaco from 'monaco-editor';

export type CodeSnapshotTheme = 'light' | 'dark';

interface CodeSnapshotOptions {
  content: string;
  language: string;
  theme: CodeSnapshotTheme;
  title?: string;
  showLineNumbers?: boolean;
}

interface SnapshotPalette {
  backgroundStart: string;
  backgroundEnd: string;
  backgroundGlowA: string;
  backgroundGlowB: string;
  frameFill: string;
  frameBorder: string;
  frameShadow: string;
  headerFill: string;
  codeSurface: string;
  codeSurfaceBorder: string;
  title: string;
  lineNumber: string;
  gutterBorder: string;
  tokenDefault: string;
  tokenKeyword: string;
  tokenString: string;
  tokenNumber: string;
  tokenComment: string;
  tokenType: string;
  tokenOperator: string;
  tokenRegexp: string;
  tokenInvalid: string;
  pillFill: string;
  pillText: string;
}

interface SnapshotSegment {
  text: string;
  color: string;
  fontStyle?: 'normal' | 'italic';
  fontWeight?: number;
}

interface SnapshotVisualLine {
  numberLabel: string;
  segments: SnapshotSegment[];
}

const CODE_FONT_FAMILY = '"JetBrains Mono", "SF Mono", "IBM Plex Mono", ui-monospace, monospace';
const UI_FONT_FAMILY =
  '"IBM Plex Sans", "SF Pro Display", "Segoe UI Variable Text", "Segoe UI", system-ui, sans-serif';
const TAB_REPLACEMENT = '  ';
const MIN_FRAME_WIDTH = 760;
const MAX_FRAME_WIDTH = 1280;
const OUTER_PADDING = 0;
const HEADER_HEIGHT = 62;
const CODE_PADDING_X = 28;
const CODE_PADDING_Y = 26;
const CODE_FONT_SIZE = 18;
const LINE_HEIGHT = 32;
const FRAME_RADIUS = 28;
const CODE_RADIUS = 22;

const palettes: Record<CodeSnapshotTheme, SnapshotPalette> = {
  light: {
    backgroundStart: '#f8fffd',
    backgroundEnd: '#eef6ff',
    backgroundGlowA: 'rgba(20, 184, 166, 0.18)',
    backgroundGlowB: 'rgba(249, 115, 22, 0.16)',
    frameFill: 'rgba(255, 255, 255, 0.94)',
    frameBorder: 'rgba(148, 163, 184, 0.22)',
    frameShadow: 'rgba(15, 23, 42, 0.18)',
    headerFill: 'rgba(248, 250, 252, 0.92)',
    codeSurface: '#ffffff',
    codeSurfaceBorder: 'rgba(226, 232, 240, 0.9)',
    title: '#0f172a',
    lineNumber: '#94a3b8',
    gutterBorder: 'rgba(226, 232, 240, 0.9)',
    tokenDefault: '#0f172a',
    tokenKeyword: '#1d4ed8',
    tokenString: '#059669',
    tokenNumber: '#dc2626',
    tokenComment: '#6b7280',
    tokenType: '#7c3aed',
    tokenOperator: '#0f766e',
    tokenRegexp: '#b45309',
    tokenInvalid: '#b91c1c',
    pillFill: 'rgba(226, 232, 240, 0.72)',
    pillText: '#334155',
  },
  dark: {
    backgroundStart: '#091317',
    backgroundEnd: '#0f172a',
    backgroundGlowA: 'rgba(45, 212, 191, 0.18)',
    backgroundGlowB: 'rgba(251, 146, 60, 0.16)',
    frameFill: 'rgba(11, 23, 32, 0.92)',
    frameBorder: 'rgba(71, 85, 105, 0.38)',
    frameShadow: 'rgba(2, 8, 23, 0.46)',
    headerFill: 'rgba(15, 23, 42, 0.92)',
    codeSurface: '#0b1322',
    codeSurfaceBorder: 'rgba(51, 65, 85, 0.82)',
    title: '#e2e8f0',
    lineNumber: '#64748b',
    gutterBorder: 'rgba(51, 65, 85, 0.85)',
    tokenDefault: '#e5eef8',
    tokenKeyword: '#60a5fa',
    tokenString: '#34d399',
    tokenNumber: '#f87171',
    tokenComment: '#94a3b8',
    tokenType: '#c084fc',
    tokenOperator: '#2dd4bf',
    tokenRegexp: '#fbbf24',
    tokenInvalid: '#f87171',
    pillFill: 'rgba(30, 41, 59, 0.9)',
    pillText: '#cbd5e1',
  },
};

const clamp = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value));

const normalizeCodeText = (content: string) =>
  content.replace(/\r\n?/g, '\n').replace(/\t/g, TAB_REPLACEMENT);

const buildFont = ({
  size,
  family,
  weight = 500,
  style = 'normal',
}: {
  size: number;
  family: string;
  weight?: number;
  style?: 'normal' | 'italic';
}) => `${style} ${weight} ${size}px ${family}`;

const waitForFonts = async () => {
  if (typeof document === 'undefined' || !('fonts' in document)) {
    return;
  }

  try {
    await (document as Document & { fonts?: FontFaceSet }).fonts?.ready;
  } catch {
    // Best-effort only.
  }
};

const createCanvasContext = () => {
  if (typeof document === 'undefined') {
    throw new Error('Document is unavailable');
  }

  const canvas = document.createElement('canvas');
  const context = canvas.getContext('2d');
  if (!context) {
    throw new Error('Canvas 2D context is unavailable');
  }

  return { canvas, context };
};

const drawRoundedRect = (
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number
) => {
  const safeRadius = Math.min(radius, width / 2, height / 2);
  context.beginPath();
  context.moveTo(x + safeRadius, y);
  context.lineTo(x + width - safeRadius, y);
  context.quadraticCurveTo(x + width, y, x + width, y + safeRadius);
  context.lineTo(x + width, y + height - safeRadius);
  context.quadraticCurveTo(x + width, y + height, x + width - safeRadius, y + height);
  context.lineTo(x + safeRadius, y + height);
  context.quadraticCurveTo(x, y + height, x, y + height - safeRadius);
  context.lineTo(x, y + safeRadius);
  context.quadraticCurveTo(x, y, x + safeRadius, y);
  context.closePath();
};

const measurePillWidth = (context: CanvasRenderingContext2D, text: string) => {
  context.save();
  context.font = buildFont({ size: 12, family: UI_FONT_FAMILY, weight: 600 });
  const width = Math.ceil(context.measureText(text).width + 24);
  context.restore();
  return width;
};

const drawPill = (
  context: CanvasRenderingContext2D,
  {
    text,
    x,
    y,
    palette,
  }: {
    text: string;
    x: number;
    y: number;
    palette: SnapshotPalette;
  }
) => {
  const pillHeight = 28;
  const pillWidth = measurePillWidth(context, text);

  context.save();
  drawRoundedRect(context, x, y, pillWidth, pillHeight, pillHeight / 2);
  context.fillStyle = palette.pillFill;
  context.fill();

  context.font = buildFont({ size: 12, family: UI_FONT_FAMILY, weight: 600 });
  context.fillStyle = palette.pillText;
  context.textBaseline = 'middle';
  context.fillText(text, x + 12, y + pillHeight / 2);
  context.restore();

  return pillWidth;
};

const getTokenColor = (tokenType: string, palette: SnapshotPalette) => {
  if (!tokenType) {
    return { color: palette.tokenDefault, fontStyle: 'normal' as const, fontWeight: 500 };
  }

  if (tokenType.includes('comment')) {
    return { color: palette.tokenComment, fontStyle: 'italic' as const, fontWeight: 400 };
  }
  if (tokenType.includes('string')) {
    return { color: palette.tokenString, fontStyle: 'normal' as const, fontWeight: 500 };
  }
  if (tokenType.includes('regexp')) {
    return { color: palette.tokenRegexp, fontStyle: 'normal' as const, fontWeight: 500 };
  }
  if (tokenType.includes('number')) {
    return { color: palette.tokenNumber, fontStyle: 'normal' as const, fontWeight: 500 };
  }
  if (
    tokenType.includes('keyword') ||
    tokenType.includes('storage') ||
    tokenType.includes('annotation')
  ) {
    return { color: palette.tokenKeyword, fontStyle: 'normal' as const, fontWeight: 600 };
  }
  if (
    tokenType.includes('type') ||
    tokenType.includes('namespace') ||
    tokenType.includes('class') ||
    tokenType.includes('interface')
  ) {
    return { color: palette.tokenType, fontStyle: 'normal' as const, fontWeight: 600 };
  }
  if (
    tokenType.includes('operator') ||
    tokenType.includes('delimiter') ||
    tokenType.includes('tag')
  ) {
    return { color: palette.tokenOperator, fontStyle: 'normal' as const, fontWeight: 500 };
  }
  if (tokenType.includes('invalid')) {
    return { color: palette.tokenInvalid, fontStyle: 'normal' as const, fontWeight: 600 };
  }

  return { color: palette.tokenDefault, fontStyle: 'normal' as const, fontWeight: 500 };
};

const tokenizeLineSegments = ({
  line,
  tokens,
  palette,
}: {
  line: string;
  tokens: monaco.Token[] | undefined;
  palette: SnapshotPalette;
}): SnapshotSegment[] => {
  if (!tokens || tokens.length === 0 || line.length === 0) {
    return [{ text: line, color: palette.tokenDefault, fontStyle: 'normal', fontWeight: 500 }];
  }

  const segments: SnapshotSegment[] = [];

  for (let index = 0; index < tokens.length; index += 1) {
    const currentToken = tokens[index];
    const nextToken = tokens[index + 1];
    const start = currentToken.offset;
    const end = nextToken?.offset ?? line.length;
    const text = line.slice(start, end);

    if (!text) {
      continue;
    }

    const tokenStyle = getTokenColor(currentToken.type, palette);
    segments.push({
      text,
      color: tokenStyle.color,
      fontStyle: tokenStyle.fontStyle,
      fontWeight: tokenStyle.fontWeight,
    });
  }

  return segments.length > 0
    ? segments
    : [{ text: line, color: palette.tokenDefault, fontStyle: 'normal', fontWeight: 500 }];
};

const measureSegmentText = (
  context: CanvasRenderingContext2D,
  segment: SnapshotSegment,
  text: string
) => {
  context.font = buildFont({
    size: CODE_FONT_SIZE,
    family: CODE_FONT_FAMILY,
    weight: segment.fontWeight ?? 500,
    style: segment.fontStyle ?? 'normal',
  });
  return context.measureText(text).width;
};

const findWrapBreakpoint = (text: string, maxCharacters: number) => {
  const softBreakpoint = Math.max(
    text.lastIndexOf(' ', maxCharacters - 1),
    text.lastIndexOf('.', maxCharacters - 1),
    text.lastIndexOf('/', maxCharacters - 1),
    text.lastIndexOf('_', maxCharacters - 1),
    text.lastIndexOf('-', maxCharacters - 1)
  );

  if (softBreakpoint > Math.floor(maxCharacters * 0.55)) {
    return softBreakpoint + 1;
  }

  return maxCharacters;
};

const measureFittingCharacters = ({
  context,
  segment,
  text,
  maxWidth,
}: {
  context: CanvasRenderingContext2D;
  segment: SnapshotSegment;
  text: string;
  maxWidth: number;
}) => {
  if (maxWidth <= 0) {
    return 0;
  }

  let low = 1;
  let high = text.length;
  let best = 0;

  while (low <= high) {
    const middle = Math.floor((low + high) / 2);
    const slice = text.slice(0, middle);
    const width = measureSegmentText(context, segment, slice);

    if (width <= maxWidth) {
      best = middle;
      low = middle + 1;
    } else {
      high = middle - 1;
    }
  }

  return best;
};

const createDefaultSegment = (palette: SnapshotPalette): SnapshotSegment => ({
  text: '',
  color: palette.tokenDefault,
  fontStyle: 'normal',
  fontWeight: 500,
});

const wrapSegments = ({
  context,
  segments,
  maxWidth,
  palette,
}: {
  context: CanvasRenderingContext2D;
  segments: SnapshotSegment[];
  maxWidth: number;
  palette: SnapshotPalette;
}) => {
  if (segments.length === 0) {
    return [[createDefaultSegment(palette)]];
  }

  const visualLines: SnapshotSegment[][] = [[]];
  let remainingWidth = maxWidth;

  const pushLine = () => {
    visualLines.push([]);
    remainingWidth = maxWidth;
  };

  for (const segment of segments) {
    let remainder = segment.text;

    while (remainder.length > 0) {
      const fittingCharacters = measureFittingCharacters({
        context,
        segment,
        text: remainder,
        maxWidth: remainingWidth,
      });

      if (fittingCharacters === 0) {
        pushLine();
        continue;
      }

      let sliceLength = fittingCharacters;
      if (sliceLength < remainder.length) {
        sliceLength = findWrapBreakpoint(remainder, fittingCharacters);
      }

      const chunk = remainder.slice(0, sliceLength);
      visualLines[visualLines.length - 1].push({ ...segment, text: chunk });
      remainingWidth -= measureSegmentText(context, segment, chunk);
      remainder = remainder.slice(sliceLength);

      if (remainder.length > 0) {
        pushLine();
      }
    }
  }

  return visualLines;
};

const buildVisualLines = ({
  context,
  rawLines,
  tokenLines,
  palette,
  maxWidth,
  showLineNumbers,
}: {
  context: CanvasRenderingContext2D;
  rawLines: string[];
  tokenLines: monaco.Token[][];
  palette: SnapshotPalette;
  maxWidth: number;
  showLineNumbers: boolean;
}): SnapshotVisualLine[] =>
  rawLines.flatMap((line, index) => {
    const tokenSegments = tokenizeLineSegments({
      line,
      tokens: tokenLines[index],
      palette,
    });
    const wrappedLines = wrapSegments({
      context,
      segments: tokenSegments,
      maxWidth,
      palette,
    });

    return wrappedLines.map((segments, wrappedIndex) => ({
      numberLabel: showLineNumbers && wrappedIndex === 0 ? String(index + 1) : '',
      segments,
    }));
  });

const drawBackground = (
  context: CanvasRenderingContext2D,
  width: number,
  height: number,
  palette: SnapshotPalette
) => {
  const linearGradient = context.createLinearGradient(0, 0, width, height);
  linearGradient.addColorStop(0, palette.backgroundStart);
  linearGradient.addColorStop(1, palette.backgroundEnd);
  context.fillStyle = linearGradient;
  context.fillRect(0, 0, width, height);

  const topGlow = context.createRadialGradient(
    width * 0.18,
    height * 0.14,
    40,
    width * 0.18,
    height * 0.14,
    width * 0.42
  );
  topGlow.addColorStop(0, palette.backgroundGlowA);
  topGlow.addColorStop(1, 'transparent');
  context.fillStyle = topGlow;
  context.fillRect(0, 0, width, height);

  const bottomGlow = context.createRadialGradient(
    width * 0.84,
    height * 0.9,
    30,
    width * 0.84,
    height * 0.9,
    width * 0.36
  );
  bottomGlow.addColorStop(0, palette.backgroundGlowB);
  bottomGlow.addColorStop(1, 'transparent');
  context.fillStyle = bottomGlow;
  context.fillRect(0, 0, width, height);
};

export async function generateCodeSnapshotDataUrl({
  content,
  language,
  theme,
  title,
  showLineNumbers = true,
}: CodeSnapshotOptions) {
  const normalizedContent = normalizeCodeText(content);
  const rawLines = normalizedContent.split('\n');
  const palette = palettes[theme];

  await waitForFonts();

  const { canvas, context } = createCanvasContext();
  context.font = buildFont({ size: CODE_FONT_SIZE, family: CODE_FONT_FAMILY, weight: 500 });

  const longestLineWidth = rawLines.reduce((maxWidth, line) => {
    const lineWidth = context.measureText(line || ' ').width;
    return Math.max(maxWidth, lineWidth);
  }, 0);

  const lineNumberGutterWidth = showLineNumbers
    ? Math.max(context.measureText(String(rawLines.length)).width + 24, 52)
    : 0;

  const frameWidth = clamp(
    Math.ceil(longestLineWidth + lineNumberGutterWidth + CODE_PADDING_X * 2),
    MIN_FRAME_WIDTH,
    MAX_FRAME_WIDTH
  );
  const codeWidth = frameWidth - lineNumberGutterWidth - CODE_PADDING_X * 2;

  let tokenLines: monaco.Token[][];
  try {
    tokenLines = monaco.editor.tokenize(normalizedContent, language);
  } catch {
    tokenLines = rawLines.map(() => []);
  }

  const visualLines = buildVisualLines({
    context,
    rawLines,
    tokenLines,
    palette,
    maxWidth: codeWidth,
    showLineNumbers,
  });

  const frameHeight = HEADER_HEIGHT + CODE_PADDING_Y * 2 + visualLines.length * LINE_HEIGHT;
  const canvasWidth = frameWidth + OUTER_PADDING * 2;
  const canvasHeight = frameHeight + OUTER_PADDING * 2;
  const pixelRatio =
    typeof window === 'undefined' ? 2 : Math.max(2, Math.min(window.devicePixelRatio || 2, 3));

  canvas.width = Math.round(canvasWidth * pixelRatio);
  canvas.height = Math.round(canvasHeight * pixelRatio);
  canvas.style.width = `${canvasWidth}px`;
  canvas.style.height = `${canvasHeight}px`;
  context.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);

  drawBackground(context, canvasWidth, canvasHeight, palette);

  const frameX = OUTER_PADDING;
  const frameY = OUTER_PADDING;

  drawRoundedRect(context, frameX, frameY, frameWidth, frameHeight, FRAME_RADIUS);
  context.fillStyle = palette.frameFill;
  context.fill();

  drawRoundedRect(context, frameX, frameY, frameWidth, frameHeight, FRAME_RADIUS);
  context.strokeStyle = palette.frameBorder;
  context.lineWidth = 1;
  context.stroke();

  drawRoundedRect(context, frameX, frameY, frameWidth, HEADER_HEIGHT, FRAME_RADIUS);
  context.save();
  context.clip();
  context.fillStyle = palette.headerFill;
  context.fillRect(frameX, frameY, frameWidth, HEADER_HEIGHT);
  context.restore();

  context.strokeStyle = palette.frameBorder;
  context.beginPath();
  context.moveTo(frameX, frameY + HEADER_HEIGHT);
  context.lineTo(frameX + frameWidth, frameY + HEADER_HEIGHT);
  context.stroke();

  const trafficLights = ['#ff5f57', '#ffbd2e', '#28c840'];
  trafficLights.forEach((color, index) => {
    context.beginPath();
    context.fillStyle = color;
    context.arc(frameX + 24 + index * 16, frameY + HEADER_HEIGHT / 2, 5.5, 0, Math.PI * 2);
    context.fill();
  });

  context.fillStyle = palette.title;
  context.font = buildFont({ size: 14, family: UI_FONT_FAMILY, weight: 600 });
  context.textBaseline = 'middle';
  context.fillText(title || 'Code Snapshot', frameX + 84, frameY + HEADER_HEIGHT / 2);

  const headerPills = [language === 'plaintext' ? 'code' : language, `${rawLines.length} lines`];
  let pillCursorX = frameX + frameWidth - 18;

  [...headerPills].reverse().forEach((pill) => {
    const pillWidth = measurePillWidth(context, pill);
    pillCursorX -= pillWidth;
    drawPill(context, {
      text: pill,
      x: pillCursorX,
      y: frameY + (HEADER_HEIGHT - 28) / 2,
      palette,
    });
    pillCursorX -= 10;
  });

  const codeSurfaceX = frameX + 10;
  const codeSurfaceY = frameY + HEADER_HEIGHT + 10;
  const codeSurfaceWidth = frameWidth - 20;
  const codeSurfaceHeight = frameHeight - HEADER_HEIGHT - 20;

  drawRoundedRect(
    context,
    codeSurfaceX,
    codeSurfaceY,
    codeSurfaceWidth,
    codeSurfaceHeight,
    CODE_RADIUS
  );
  context.fillStyle = palette.codeSurface;
  context.fill();
  context.strokeStyle = palette.codeSurfaceBorder;
  context.stroke();

  if (showLineNumbers) {
    context.beginPath();
    context.moveTo(
      frameX + CODE_PADDING_X + lineNumberGutterWidth,
      frameY + HEADER_HEIGHT + CODE_PADDING_Y - 8
    );
    context.lineTo(
      frameX + CODE_PADDING_X + lineNumberGutterWidth,
      frameY + frameHeight - CODE_PADDING_Y + 8
    );
    context.strokeStyle = palette.gutterBorder;
    context.stroke();
  }

  const codeOriginX = frameX + CODE_PADDING_X + lineNumberGutterWidth;
  const lineNumberX = frameX + CODE_PADDING_X + lineNumberGutterWidth - 14;
  const firstLineBaseline = frameY + HEADER_HEIGHT + CODE_PADDING_Y + CODE_FONT_SIZE;

  visualLines.forEach((visualLine, lineIndex) => {
    const baselineY = firstLineBaseline + lineIndex * LINE_HEIGHT;

    if (showLineNumbers && visualLine.numberLabel) {
      context.save();
      context.fillStyle = palette.lineNumber;
      context.font = buildFont({ size: 14, family: CODE_FONT_FAMILY, weight: 500 });
      context.textAlign = 'right';
      context.textBaseline = 'alphabetic';
      context.fillText(visualLine.numberLabel, lineNumberX, baselineY);
      context.restore();
    }

    let cursorX = codeOriginX;
    for (const segment of visualLine.segments) {
      if (!segment.text) {
        continue;
      }

      context.save();
      context.fillStyle = segment.color;
      context.font = buildFont({
        size: CODE_FONT_SIZE,
        family: CODE_FONT_FAMILY,
        weight: segment.fontWeight ?? 500,
        style: segment.fontStyle ?? 'normal',
      });
      context.textBaseline = 'alphabetic';
      context.fillText(segment.text, cursorX, baselineY);
      cursorX += measureSegmentText(context, segment, segment.text);
      context.restore();
    }
  });

  return canvas.toDataURL('image/png');
}
