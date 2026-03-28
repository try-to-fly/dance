import { useEffect, useState } from 'react';
import { AlertTriangle, AppWindow, Clock3, Layers3 } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { useClipboardStore } from '../../stores/clipboardStore';
import { ContentSubType, ResolvedPreviewData } from '../../types/clipboard';
import { buildPreviewDescriptor } from '../../lib/preview/previewDescriptor';
import {
  getEntryAnalysisDiagnostics,
  getEntryAnalysisStatus,
  getEntryAnalysisSubtype,
} from '../../lib/preview/entryPresentation';
import { DetailEmptyState, DetailScene } from './scene/DetailScene';

const normalizeMetaLabel = (value: string) => value.replace(/[：:]$/, '');

const normalizeEntryUrl = (value?: string | null) => {
  const trimmed = value?.trim();
  if (!trimmed) {
    return '';
  }

  try {
    const parsed = new URL(trimmed);
    if (parsed.protocol === 'http:' || parsed.protocol === 'https:') {
      return parsed.toString();
    }
  } catch {
    // Keep bare-domain fallback below.
  }

  if (!/^\S+\.\S+/.test(trimmed)) {
    return '';
  }

  try {
    return new URL(`https://${trimmed}`).toString();
  } catch {
    return '';
  }
};

const getSubTypeFallbackLabel = (subType: ContentSubType) => {
  const map: Record<ContentSubType, string> = {
    plain_text: 'Text',
    url: 'URL',
    ip_address: 'IP',
    email: 'Email',
    color: 'Color',
    code: 'Code',
    command: 'Command',
    timestamp: 'Timestamp',
    json: 'JSON',
    markdown: 'Markdown',
    base64: 'Base64',
  };

  return map[subType];
};

const stringifyPreviewValue = (value: unknown) => {
  if (typeof value === 'string') {
    return value;
  }

  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
};

export function DetailView() {
  const { t } = useTranslation(['common', 'clipboard']);
  const {
    selectedEntry,
    getImageUrl,
    openFileWithSystem,
    copyToClipboard,
    pasteSelectedEntry,
    toggleFavorite,
    deleteEntry,
    resolveEntryPreview,
  } = useClipboardStore();
  const [resolvedPreview, setResolvedPreview] = useState<{
    sourceKey: string;
    data: ResolvedPreviewData | null;
  } | null>(null);
  const [workbenchBuffer, setWorkbenchBuffer] = useState<string | null>(null);
  const selectedEntryKey = selectedEntry
    ? `${selectedEntry.id}:${selectedEntry.content_hash}`
    : null;
  const selectedEntrySubType = getEntryAnalysisSubtype(selectedEntry);
  const usesWorkbench =
    Boolean(selectedEntry) &&
    (selectedEntrySubType === 'code' || selectedEntrySubType === 'command');
  const activeResolvedPreview =
    selectedEntryKey && resolvedPreview?.sourceKey === selectedEntryKey
      ? resolvedPreview.data
      : null;
  const resolveLabel = (key: string, fallback: string) => {
    const translated = t(key);
    return translated === key ? fallback : translated;
  };

  useEffect(() => {
    let isActive = true;

    const resolve = async () => {
      if (!selectedEntry) {
        setResolvedPreview(null);
        return;
      }

      const sourceKey = `${selectedEntry.id}:${selectedEntry.content_hash}`;
      setResolvedPreview((current) =>
        current?.sourceKey === sourceKey ? current : { sourceKey, data: null }
      );

      let nextResolved: ResolvedPreviewData = {};

      if (resolveEntryPreview) {
        try {
          nextResolved = (await resolveEntryPreview(selectedEntry)) || {};
        } catch (error) {
          console.error('[DetailView] 解析预览失败:', error);
        }
      }

      if (
        selectedEntry.file_path &&
        selectedEntry.content_type.toLowerCase().includes('image') &&
        !nextResolved.imageUrl
      ) {
        try {
          const imageUrl = await getImageUrl(selectedEntry.file_path);
          nextResolved = {
            ...nextResolved,
            sourceKind: nextResolved.sourceKind ?? 'local',
            imageUrl,
          };
        } catch (error) {
          console.error('[DetailView] 图片加载失败:', error);
        }
      }

      if (isActive) {
        setResolvedPreview({ sourceKey, data: nextResolved });
      }
    };

    void resolve();

    return () => {
      isActive = false;
    };
  }, [getImageUrl, resolveEntryPreview, selectedEntry, selectedEntryKey]);

  useEffect(() => {
    if (!selectedEntry || !usesWorkbench) {
      setWorkbenchBuffer(null);
      return;
    }

    setWorkbenchBuffer(selectedEntry.content_data ?? '');
  }, [selectedEntry, selectedEntryKey, usesWorkbench]);

  if (!selectedEntry) {
    return <DetailEmptyState selectItemLabel={t('detail.selectItem')} />;
  }

  const subType = selectedEntrySubType;
  const analysisStatus = getEntryAnalysisStatus(selectedEntry);
  const analysisDiagnostics = getEntryAnalysisDiagnostics(selectedEntry);
  const translatedSubTypeLabel = t(`detail.contentTypes.${subType}`);
  const subTypeLabel =
    translatedSubTypeLabel === `detail.contentTypes.${subType}`
      ? getSubTypeFallbackLabel(subType)
      : translatedSubTypeLabel;

  const metadataPills = [
    {
      key: 'source',
      icon: AppWindow,
      label: normalizeMetaLabel(t('detail.source')),
      value: selectedEntry.source_app || t('detail.unknown'),
      fullValue: selectedEntry.source_app || t('detail.unknown'),
    },
    {
      key: 'time',
      value: new Date(selectedEntry.created_at).toLocaleString(undefined, {
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
      }),
      fullValue: new Date(selectedEntry.created_at).toLocaleString(),
      icon: Clock3,
      label: normalizeMetaLabel(t('detail.time')),
    },
    {
      key: 'copy-count',
      icon: Layers3,
      label: normalizeMetaLabel(t('detail.copyCount')),
      value: String(selectedEntry.copy_count),
      fullValue: String(selectedEntry.copy_count),
    },
    ...(analysisStatus === 'fallback'
      ? [
          {
            key: 'analysis-status',
            icon: AlertTriangle,
            label: normalizeMetaLabel(resolveLabel('detail.analysisStatus', '分析')),
            value: analysisDiagnostics[0]?.code || 'Fallback',
            fullValue: analysisDiagnostics[0]?.message || 'Fallback',
          },
        ]
      : []),
  ];

  const descriptor = buildPreviewDescriptor({
    entry: selectedEntry,
    resolvedData: activeResolvedPreview || undefined,
    labels: {
      unknown: t('detail.unknown'),
      image: t('detail.contentTypes.image'),
      file: t('detail.contentTypes.file'),
      text: subTypeLabel,
      base64:
        t('detail.contentTypes.base64') === 'detail.contentTypes.base64'
          ? 'Base64'
          : t('detail.contentTypes.base64'),
      subtypeLabels: {
        plain_text: t('detail.contentTypes.plain_text') || subTypeLabel,
        url: t('detail.contentTypes.url') || 'URL',
        ip_address: t('detail.contentTypes.ip_address') || 'IP',
        email: t('detail.contentTypes.email') || 'Email',
        color: t('detail.contentTypes.color') || 'Color',
        code: t('detail.contentTypes.code') || 'Code',
        command: t('detail.contentTypes.command') || 'Command',
        timestamp: t('detail.contentTypes.timestamp') || 'Timestamp',
        json: t('detail.contentTypes.json') || 'JSON',
        markdown: t('detail.contentTypes.markdown') || 'Markdown',
        base64: t('detail.contentTypes.base64') || 'Base64',
      },
    },
  });
  const detailDescriptor = usesWorkbench
    ? {
        ...descriptor,
        primaryPayload: {
          ...(descriptor.primaryPayload as Record<string, unknown>),
          sessionKey: selectedEntryKey,
          onContentChange: setWorkbenchBuffer,
        },
      }
    : descriptor;

  const handleCopy = async () => {
    if (usesWorkbench && workbenchBuffer !== null) {
      await copyToClipboard(workbenchBuffer);
      return;
    }

    if (selectedEntry.content_data) {
      await copyToClipboard(selectedEntry.content_data);
    }
  };

  const decodedContent =
    activeResolvedPreview?.base64?.textPreview ??
    (activeResolvedPreview?.jsonContent !== undefined
      ? stringifyPreviewValue(activeResolvedPreview.jsonContent)
      : (activeResolvedPreview?.textContent ?? null));

  const handleCopyDecoded = async () => {
    if (decodedContent) {
      await copyToClipboard(decodedContent);
    }
  };

  const handlePaste = async () => {
    await pasteSelectedEntry(selectedEntry);
  };

  const handleDelete = async () => {
    await deleteEntry(selectedEntry.id);
  };

  const handleOpenFile = async () => {
    if (!selectedEntry.file_path) {
      return;
    }

    try {
      await openFileWithSystem(selectedEntry.file_path);
    } catch (error) {
      console.error('[DetailView] 打开文件失败:', error);
    }
  };

  const handleOpenUrl = () => {
    const previewUrl =
      activeResolvedPreview?.url?.finalUrl || normalizeEntryUrl(selectedEntry.content_data);
    if (!previewUrl) {
      return;
    }

    try {
      const url = new URL(previewUrl);
      if (url.protocol === 'http:' || url.protocol === 'https:') {
        window.open(url.toString(), '_blank');
      }
    } catch {
      // Ignore invalid URL values
    }
  };

  return (
    <DetailScene
      entry={selectedEntry}
      descriptor={detailDescriptor}
      metadataPills={metadataPills}
      labels={{
        copy: t('copy'),
        copyDecoded: resolveLabel('detail.actions.copyDecoded', '复制解码内容'),
        paste: t('paste'),
        delete: t('delete'),
        favorite: t('clipboard:actions.favorite'),
        unfavorite: t('clipboard:actions.unfavorite'),
        openFile: resolveLabel('detail.actions.openFile', '打开文件'),
        openUrl: resolveLabel('renderers.url.open', '打开链接'),
        title: t('detail.title'),
      }}
      onCopy={handleCopy}
      onCopyDecoded={handleCopyDecoded}
      onPaste={handlePaste}
      onDelete={handleDelete}
      onToggleFavorite={() => toggleFavorite(selectedEntry.id)}
      onOpenUrl={handleOpenUrl}
      onOpenFile={handleOpenFile}
      canCopyDecoded={Boolean(decodedContent)}
    />
  );
}
