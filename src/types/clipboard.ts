export interface ClipboardEntry {
  id: string;
  content_hash: string;
  content_type: string;
  content_data: string | null;
  source_app: string | null;
  created_at: number;
  copy_count: number;
  file_path: string | null;
  is_favorite: boolean;
  content_subtype?: ContentSubType | string | null;
  metadata?: string | null;
  app_bundle_id?: string | null;
}

export type ContentType = 'text' | 'image' | 'file' | 'unknown';

export type ContentSubType =
  | 'plain_text'
  | 'url'
  | 'ip_address'
  | 'email'
  | 'color'
  | 'code'
  | 'command'
  | 'timestamp'
  | 'json'
  | 'markdown'
  | 'base64';

export interface ContentMetadata {
  detected_language?: string;
  url_parts?: UrlParts;
  color_formats?: ColorFormats;
  timestamp_formats?: TimestampFormats;
  image_metadata?: ImageMetadata;
  base64_metadata?: Base64Metadata;
  file_metadata?: FileMetadata;
  resolved_preview_summary?: ResolvedPreviewSummary;
}

export interface ImageMetadata {
  width: number;
  height: number;
  file_size: number;
  format?: string;
}

export interface UrlParts {
  protocol: string;
  host: string;
  path: string;
  query_params: Array<[string, string]>;
}

export interface ColorFormats {
  hex?: string;
  rgb?: string;
  rgba?: string;
  hsl?: string;
}

export interface TimestampFormats {
  unix_ms?: number;
  iso8601?: string;
  date_string?: string;
}

export interface Base64Metadata {
  estimated_original_size?: number;
  encoded_size?: number;
  content_hint?: string;
  encoding_efficiency?: number;
}

export interface FileMetadata {
  name?: string;
  extension?: string;
  mime?: string;
  size_bytes?: number;
  modified_at?: number;
  is_directory?: boolean;
}

export interface ResolvedPreviewSummary {
  kind?: PreviewKind;
  mime?: string;
  title?: string;
}

export type PreviewKind =
  | 'plain_text'
  | 'code'
  | 'markdown'
  | 'json'
  | 'image'
  | 'audio'
  | 'video'
  | 'url_card'
  | 'file_card'
  | 'email_card'
  | 'ip_card'
  | 'color_card'
  | 'timestamp_card'
  | 'base64_text'
  | 'base64_binary'
  | 'unsupported';

export type PreviewAction =
  | 'copy_raw'
  | 'copy_decoded'
  | 'open_url'
  | 'open_file'
  | 'download'
  | 'paste';

export interface PreviewBadge {
  label: string;
  tone?: 'default' | 'secondary' | 'warning';
}

export interface PreviewInspectorItem {
  label: string;
  value: string;
  mono?: boolean;
}

export interface PreviewInspectorSection {
  title: string;
  items: PreviewInspectorItem[];
}

export interface PreviewAlternateView {
  key: string;
  label: string;
  kind: PreviewKind | 'raw';
  payload: unknown;
}

export interface PreviewDescriptor {
  headline: string;
  typeLabel: string;
  badges: PreviewBadge[];
  primaryKind: PreviewKind;
  primaryPayload: unknown;
  inspectorSections: PreviewInspectorSection[];
  alternateViews: PreviewAlternateView[];
  actions: PreviewAction[];
}

export interface MediaPreviewInfo {
  duration?: string;
  bitrate?: string;
  codec?: string;
  width?: number;
  height?: number;
  fps?: string;
  sampleRate?: string;
  size?: string;
  format?: string;
}

export interface Base64PreviewData {
  decodedKind: 'text' | 'json' | 'image' | 'audio' | 'video' | 'binary' | 'unknown';
  mime?: string;
  textPreview?: string;
  jsonContent?: unknown;
  dataUrl?: string;
  filenameSuggestion?: string;
  sizeBytes?: number;
  error?: string;
}

export interface ResolvedPreviewData {
  sourceKind?: 'local' | 'remote' | 'decoded';
  mime?: string;
  fileName?: string;
  extension?: string;
  sizeBytes?: number;
  textContent?: string;
  jsonContent?: unknown;
  imageUrl?: string;
  audioUrl?: string;
  videoUrl?: string;
  media?: MediaPreviewInfo;
  base64?: Base64PreviewData;
  url?: {
    finalUrl?: string;
    status?: number;
    contentType?: string;
    contentLength?: number;
    title?: string;
    previewKind?: PreviewKind;
    error?: string;
  };
}

export interface UrlPreviewResolution {
  final_url?: string;
  status?: number;
  content_type?: string;
  content_length?: number;
  preview_kind?: PreviewKind;
  resolved?: {
    source_kind?: string;
    mime?: string;
    file_name?: string;
    extension?: string;
    size_bytes?: number;
    text_content?: string;
    json_content?: unknown;
    image_url?: string;
    video_url?: string;
    audio_url?: string;
    media?: {
      source?: string;
      source_kind?: string;
      kind?: string;
      mime?: string;
      format?: string;
      duration?: string;
      bitrate?: string;
      codec?: string;
      width?: number;
      height?: number;
      fps?: string;
      sample_rate?: string;
      size_bytes?: number;
      ffprobe_used?: boolean;
      error?: string;
    };
    base64?: {
      decoded_kind?: Base64PreviewData['decodedKind'];
      mime?: string;
      text_preview?: string;
      data_url?: string;
    };
  };
  error?: string;
}

export interface Base64PreviewResolution {
  preview_kind?: PreviewKind;
  decoded_kind?: 'text' | 'json' | 'image' | 'audio' | 'video' | 'binary' | 'unknown';
  filename_suggestion?: string;
  resolved?: {
    source_kind?: string;
    mime?: string;
    file_name?: string;
    extension?: string;
    size_bytes?: number;
    text_content?: string;
    json_content?: unknown;
    image_url?: string;
    video_url?: string;
    audio_url?: string;
    media?: {
      source?: string;
      source_kind?: string;
      kind?: string;
      mime?: string;
      format?: string;
      duration?: string;
      bitrate?: string;
      codec?: string;
      width?: number;
      height?: number;
      fps?: string;
      sample_rate?: string;
      size_bytes?: number;
      ffprobe_used?: boolean;
      error?: string;
    };
    base64?: {
      decoded_kind?: Base64PreviewData['decodedKind'];
      mime?: string;
      text_preview?: string;
      data_url?: string;
    };
  };
  error?: string;
}

export interface Statistics {
  total_entries: number;
  total_copies: number;
  most_copied: ClipboardEntry[];
  recent_apps: AppUsage[];
}

export interface AppUsage {
  app_name: string;
  count: number;
}
