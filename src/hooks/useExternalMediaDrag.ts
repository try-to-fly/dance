import { DragEvent, useCallback, useEffect, useMemo, useState } from 'react';
import { invoke, isTauri } from '@tauri-apps/api/core';

type DragKind = 'image' | 'video';

interface UseExternalMediaDragOptions {
  enabled: boolean;
  kind: DragKind;
  sourceUrl?: string | null;
  filePath?: string | null;
  fileName?: string | null;
  mimeType?: string | null;
}

interface PreparedDragMediaFile {
  filePath: string;
  fileName: string;
  mimeType?: string;
  isTemp: boolean;
}

const extensionFromMime = (mime?: string | null): string | null => {
  if (!mime) {
    return null;
  }

  const normalized = mime.toLowerCase().split(';')[0]?.trim();
  switch (normalized) {
    case 'image/png':
      return 'png';
    case 'image/jpeg':
      return 'jpg';
    case 'image/webp':
      return 'webp';
    case 'image/gif':
      return 'gif';
    case 'video/mp4':
      return 'mp4';
    case 'video/webm':
      return 'webm';
    case 'video/quicktime':
      return 'mov';
    default:
      return null;
  }
};

const extensionFromUrl = (value?: string | null): string | null => {
  if (!value) {
    return null;
  }

  try {
    const url = new URL(value);
    const segment = url.pathname.split('/').pop() || '';
    const extension = segment.split('.').pop();
    return extension && extension !== segment ? extension.toLowerCase() : null;
  } catch {
    const segment = value.split('/').pop() || '';
    const extension = segment.split('.').pop();
    return extension && extension !== segment ? extension.toLowerCase() : null;
  }
};

const guessFileName = ({
  kind,
  sourceUrl,
  fileName,
  mimeType,
}: {
  kind: DragKind;
  sourceUrl?: string | null;
  fileName?: string | null;
  mimeType?: string | null;
}): string => {
  if (fileName?.trim()) {
    return fileName.trim();
  }

  const extension =
    extensionFromMime(mimeType) ||
    extensionFromUrl(sourceUrl) ||
    (kind === 'image' ? 'png' : 'mp4');
  const prefix = kind === 'image' ? 'clipboard-image' : 'clipboard-video';
  return `${prefix}.${extension}`;
};

const toFileUrl = (absolutePath: string): string => {
  const unixPath = absolutePath.replace(/\\/g, '/');
  const normalized = unixPath.startsWith('/') ? unixPath : `/${unixPath}`;
  return `file://${encodeURI(normalized)}`;
};

const dataUrlToBlob = async (source: string): Promise<Blob> => {
  const response = await fetch(source);
  if (!response.ok) {
    throw new Error(`Failed to read data URL: ${response.status}`);
  }
  return response.blob();
};

const buildDragFileFromSource = async (
  sourceUrl: string,
  fileName: string,
  mimeType?: string | null
): Promise<File> => {
  const blob = sourceUrl.startsWith('data:')
    ? await dataUrlToBlob(sourceUrl)
    : await fetch(sourceUrl).then((response) => {
        if (!response.ok) {
          throw new Error(`Failed to fetch source: ${response.status}`);
        }
        return response.blob();
      });
  const resolvedType = mimeType || blob.type || undefined;
  return new File([blob], fileName, resolvedType ? { type: resolvedType } : undefined);
};

const buildDragFileFromPath = async (
  absolutePath: string,
  fileName: string,
  mimeType?: string | null
): Promise<File> => {
  const dataUrl = await invoke<string>('read_media_file_as_data_url', {
    filePath: absolutePath,
    mimeType: mimeType || undefined,
  });
  const blob = await dataUrlToBlob(dataUrl);
  const resolvedType = mimeType || blob.type || undefined;
  return new File([blob], fileName, resolvedType ? { type: resolvedType } : undefined);
};

export const useExternalMediaDrag = ({
  enabled,
  kind,
  sourceUrl,
  filePath,
  fileName,
  mimeType,
}: UseExternalMediaDragOptions) => {
  const [preparedFile, setPreparedFile] = useState<PreparedDragMediaFile | null>(null);
  const [dragFile, setDragFile] = useState<File | null>(null);

  const normalizedSourceUrl = sourceUrl?.trim() || '';
  const normalizedFilePath = filePath?.trim() || '';

  useEffect(() => {
    if (!enabled) {
      setPreparedFile(null);
      setDragFile(null);
      return;
    }

    const fallbackName = guessFileName({
      kind,
      sourceUrl: normalizedSourceUrl,
      fileName,
      mimeType,
    });

    let cancelled = false;
    const prepare = async () => {
      if (isTauri()) {
        try {
          const prepared = await invoke<PreparedDragMediaFile>('prepare_drag_media_file', {
            sourceUrl: normalizedSourceUrl || undefined,
            filePath: normalizedFilePath || undefined,
            fileName: fileName || undefined,
            mimeType: mimeType || undefined,
          });
          if (!cancelled) {
            setPreparedFile(prepared);
            try {
              const file = await buildDragFileFromPath(
                prepared.filePath,
                prepared.fileName || fallbackName,
                prepared.mimeType || mimeType
              );
              if (!cancelled) {
                setDragFile(file);
              }
            } catch (error) {
              console.debug('[media-drag] local file object prepare failed:', error);
              if (normalizedSourceUrl) {
                try {
                  const fallbackFile = await buildDragFileFromSource(
                    normalizedSourceUrl,
                    prepared.fileName || fallbackName,
                    prepared.mimeType || mimeType
                  );
                  if (!cancelled) {
                    setDragFile(fallbackFile);
                  }
                } catch (fallbackError) {
                  console.debug(
                    '[media-drag] source fallback after local-file failure failed:',
                    fallbackError
                  );
                  if (!cancelled) {
                    setDragFile(null);
                  }
                }
              } else if (!cancelled) {
                setDragFile(null);
              }
            }
          }
          return;
        } catch (error) {
          console.debug('[media-drag] prepare command failed, fallback to source:', error);
        }
      }

      if (!normalizedSourceUrl) {
        if (!cancelled) {
          setPreparedFile(null);
          setDragFile(null);
        }
        return;
      }

      try {
        const fallbackFile = await buildDragFileFromSource(
          normalizedSourceUrl,
          fallbackName,
          mimeType
        );
        if (!cancelled) {
          setPreparedFile(null);
          setDragFile(fallbackFile);
        }
      } catch (error) {
        console.debug('[media-drag] fallback file prepare failed:', error);
        if (!cancelled) {
          setPreparedFile(null);
          setDragFile(null);
        }
      }
    };

    void prepare();

    return () => {
      cancelled = true;
    };
  }, [enabled, fileName, kind, mimeType, normalizedFilePath, normalizedSourceUrl]);

  const onDragStart = useCallback(
    (event: DragEvent<HTMLElement>) => {
      const transfer = event.dataTransfer;
      if (!transfer) {
        return;
      }

      transfer.effectAllowed = 'copy';
      let attachedFile = false;

      if (dragFile) {
        try {
          transfer.items.clear();
          transfer.items.add(dragFile);
          attachedFile = true;
        } catch (error) {
          console.debug('[media-drag] attach file object failed:', error);
        }
      }

      if (attachedFile) {
        return;
      }

      const fileUrl = preparedFile?.filePath ? toFileUrl(preparedFile.filePath) : '';
      const fallbackText = fileUrl || normalizedSourceUrl || dragFile?.name || '';
      if (fileUrl) {
        transfer.setData('text/uri-list', fileUrl);
        transfer.setData('text/plain', fileUrl);
      } else if (fallbackText) {
        transfer.setData('text/plain', fallbackText);
        if (
          normalizedSourceUrl.startsWith('http://') ||
          normalizedSourceUrl.startsWith('https://')
        ) {
          transfer.setData('text/uri-list', normalizedSourceUrl);
        }
      }
    },
    [dragFile, normalizedSourceUrl, preparedFile?.filePath]
  );

  return useMemo(
    () => ({
      draggable: enabled && Boolean(dragFile),
      onDragStart,
    }),
    [dragFile, enabled, onDragStart]
  );
};
