export interface ListItem {
  id: number;
  type: string;
  title: string;
  datetime: string;
  description: string;
  status?: "primary" | "success" | "warning" | "info" | "danger";
  extra?: string;
  isRead: boolean;
}

export interface TabItem {
  key: string;
  name: string;
  list: ListItem[];
  emptyText: string;
}

export const NOTICE_TABS: Omit<TabItem, "list">[] = [
  {
    key: "1",
    name: "通知",
    emptyText: "暂无通知"
  },
  {
    key: "2",
    name: "消息",
    emptyText: "暂无消息"
  },
  {
    key: "3",
    name: "待办",
    emptyText: "暂无待办"
  },
  {
    key: "read",
    name: "已读",
    emptyText: "暂无已读"
  }
];
