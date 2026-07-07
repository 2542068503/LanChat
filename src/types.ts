export interface Peer {
  id: string;
  username: string;
  ip: string;
  lastSeen: string;
  isOnline: boolean;
  remark?: string;
  isPinned?: boolean;
  avatarId?: number;
  avatarBase64?: string;
  os?: string;
}

export interface Profile {
  username: string;
  avatarId: number;
  avatarBase64?: string;
  remark?: string;
  os?: string;
  isPinned?: boolean;
}

export interface FileInfo {
  fileId: string;
  name: string;
  size: number;
  sha256: string;
}

export interface Message {
  messageId: string;
  senderId: string;
  contentType: string; // "text" | "image" | "file" | "system"
  content: string;
  timestamp: number;
  fileInfo?: FileInfo | null;
  renderLatex?: boolean;
  quoteMsgId?: string;
  quoteSender?: string;
  quoteContent?: string;
  isLobby?: boolean;
  localPath?: string;
  isDownloading?: boolean;
  downloadBytesProcessed?: number;
  downloadTotalBytes?: number;
  downloadSpeed?: string;
  downloadEta?: string;
  downloadStartTime?: number;
}

export interface ProgressState {
  show: boolean;
  filePath: string;
  fileName: string;
  bytesProcessed: number;
  totalBytes: number;
  speed: string;
  startTime: number;
  isHashing: boolean;
  eta: string;
}

export interface FileSendConfirmState {
  show: boolean;
  filePath: string;
  fileInfo: FileInfo | null;
  targetPeerId: string;
  imagePreviewUrl: string | null;
}
