<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import BellIcon from "~icons/ep/bell";
import { message } from "@/utils/message";
import {
  getReadNotices,
  getUnreadNotices,
  markNoticeAsRead
} from "@/api/notice";
import { NOTICE_TABS, type TabItem } from "./data";
import NoticeList from "./components/NoticeList.vue";

const notices = ref<TabItem[]>(
  NOTICE_TABS.map(item => ({
    ...item,
    list: []
  }))
);
const activeKey = ref(NOTICE_TABS[0]?.key ?? "1");

const unreadTabKeys = new Set(["1", "2", "3"]);

const noticesNum = computed(() =>
  notices.value.reduce((total, tab) => {
    if (!unreadTabKeys.has(tab.key)) {
      return total;
    }
    return total + tab.list.length;
  }, 0)
);

const getLabel = computed(
  () => (item: TabItem) =>
    item.name + (item.list.length > 0 ? `(${item.list.length})` : "")
);

async function loadNotices() {
  try {
    const [unreadResponse, readResponse] = await Promise.all([
      getUnreadNotices(),
      getReadNotices()
    ]);

    const unreadItems = unreadResponse.data ?? [];
    const readItems = readResponse.data ?? [];

    notices.value = NOTICE_TABS.map(tab => {
      if (tab.key === "read") {
        return {
          ...tab,
          list: readItems
        };
      }

      return {
        ...tab,
        list: unreadItems.filter(item => item.type === tab.key)
      };
    });
  } catch (error: any) {
    message(error?.message ?? "加载通知失败", { type: "error" });
  }
}

async function onRead(id: number) {
  try {
    const result = await markNoticeAsRead(id);
    if (!result.data) {
      return;
    }
    await loadNotices();
  } catch (error: any) {
    message(error?.message ?? "设置已读失败", { type: "error" });
  }
}

onMounted(() => {
  loadNotices();
});
</script>

<template>
  <el-dropdown trigger="click" placement="bottom-end">
    <span
      :class="[
        'dropdown-badge',
        'navbar-bg-hover',
        'select-none',
        Number(noticesNum) !== 0 && 'mr-[10px]'
      ]"
    >
      <el-badge :value="Number(noticesNum) === 0 ? '' : noticesNum" :max="99">
        <span class="header-notice-icon">
          <IconifyIconOffline :icon="BellIcon" />
        </span>
      </el-badge>
    </span>
    <template #dropdown>
      <el-dropdown-menu>
        <el-tabs
          v-model="activeKey"
          :stretch="true"
          class="dropdown-tabs"
          :style="{ width: notices.length === 0 ? '200px' : '330px' }"
        >
          <el-empty
            v-if="notices.length === 0"
            description="暂无消息"
            :image-size="60"
          />
          <span v-else>
            <template v-for="item in notices" :key="item.key">
              <el-tab-pane :label="getLabel(item)" :name="`${item.key}`">
                <el-scrollbar max-height="330px">
                  <div class="noticeList-container">
                    <NoticeList
                      :list="item.list"
                      :showReadButton="item.key !== 'read'"
                      :emptyText="item.emptyText"
                      @read="onRead"
                    />
                  </div>
                </el-scrollbar>
              </el-tab-pane>
            </template>
          </span>
        </el-tabs>
      </el-dropdown-menu>
    </template>
  </el-dropdown>
</template>

<style lang="scss" scoped>
.dropdown-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 48px;
  cursor: pointer;

  .header-notice-icon {
    font-size: 18px;
  }
}

.dropdown-tabs {
  .noticeList-container {
    padding: 15px 24px 0;
  }

  :deep(.el-tabs__header) {
    margin: 0;
  }

  :deep(.el-tabs__nav-wrap)::after {
    height: 1px;
  }

  :deep(.el-tabs__nav-wrap) {
    padding: 0 36px;
  }
}
</style>
