<script setup lang="ts">
import { PropType } from "vue";
import { ListItem } from "../data";
import NoticeItem from "./NoticeItem.vue";

const emit = defineEmits<{
  read: [id: number];
}>();

defineProps({
  list: {
    type: Array as PropType<Array<ListItem>>,
    default: () => []
  },
  showReadButton: {
    type: Boolean,
    default: true
  },
  emptyText: {
    type: String,
    default: ""
  }
});

function onRead(id: number) {
  emit("read", id);
}
</script>

<template>
  <div v-if="list.length">
    <NoticeItem
      v-for="item in list"
      :key="item.id"
      :noticeItem="item"
      :showReadButton="showReadButton"
      @read="onRead"
    />
  </div>
  <el-empty v-else :description="emptyText" />
</template>
