import { invoke, isTauri } from "@tauri-apps/api/core";

export type NoticeItem = {
  id: number;
  type: string;
  title: string;
  datetime: string;
  description: string;
  status?: "primary" | "success" | "warning" | "info" | "danger";
  extra?: string;
  isRead: boolean;
};

type NoticeListResult = {
  success: boolean;
  data: NoticeItem[];
};

type MarkReadResult = {
  success: boolean;
  data: boolean;
};

export const getUnreadNotices = () => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`getUnreadNotices` only supports Tauri desktop runtime.")
    );
  }
  return invoke<NoticeListResult>("notice_get_unread_items");
};

export const getReadNotices = () => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`getReadNotices` only supports Tauri desktop runtime.")
    );
  }
  return invoke<NoticeListResult>("notice_get_read_items");
};

export const markNoticeAsRead = (id: number) => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`markNoticeAsRead` only supports Tauri desktop runtime.")
    );
  }
  return invoke<MarkReadResult>("notice_mark_read", { payload: { id } });
};
