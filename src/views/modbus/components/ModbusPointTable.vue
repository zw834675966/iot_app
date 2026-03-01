<script setup lang="ts">
import type { ModbusPointRow } from "../composables/useModbusPage";

defineOptions({
  name: "ModbusPointTable"
});

const props = defineProps<{
  points: ModbusPointRow[];
  loadingRowId: string;
  pollingRowId: string;
  pollRunning: boolean;
  autoReadRowId: string;
  autoReadRunning: boolean;
  selectedPointIds: string[];
}>();

const emit = defineEmits<{
  (event: "read", row: ModbusPointRow): void;
  (event: "write", row: ModbusPointRow): void;
  (event: "poll-start", row: ModbusPointRow): void;
  (event: "remove", id: string): void;
  (event: "selection-change", pointIds: string[]): void;
}>();

const READ_CODES = [1, 2, 3, 4];
const WRITE_CODES = [5, 6, 15, 16];

const functionCodeOptions = [
  { value: 1, label: "01 读线圈" },
  { value: 2, label: "02 读离散输入" },
  { value: 3, label: "03 读保持寄存器" },
  { value: 4, label: "04 读输入寄存器" },
  { value: 5, label: "05 写单线圈" },
  { value: 6, label: "06 写单寄存器" },
  { value: 15, label: "15 写多线圈" },
  { value: 16, label: "16 写多寄存器" }
];

function isReadCode(functionCode: number) {
  return READ_CODES.includes(functionCode);
}

function isWriteCode(functionCode: number) {
  return WRITE_CODES.includes(functionCode);
}

function formatUpdatedAt(timestamp?: number): string {
  if (!timestamp) return "--";
  return new Date(timestamp).toLocaleTimeString();
}

function handleSelectionChange(rows: ModbusPointRow[]) {
  emit(
    "selection-change",
    rows.map(row => row.id)
  );
}
</script>

<template>
  <el-table
    :data="props.points"
    stripe
    border
    size="small"
    table-layout="fixed"
    class="modbus-table flex-1 w-full h-full"
    height="100%"
    row-key="id"
    @selection-change="handleSelectionChange"
  >
    <el-table-column type="selection" width="48" reserve-selection />
    <el-table-column type="index" label="序号" width="56" />

    <el-table-column label="名称" min-width="140" show-overflow-tooltip>
      <template #default="{ row }">
        <el-input v-model="row.name" placeholder="点位名称" />
      </template>
    </el-table-column>

    <el-table-column label="数值" min-width="132" show-overflow-tooltip>
      <template #default="{ row }">
        <div class="value-cell">
          <span>{{ row.valueText || "--" }}</span>
          <small>{{ formatUpdatedAt(row.updatedAt) }}</small>
        </div>
      </template>
    </el-table-column>

    <el-table-column label="功能码" min-width="152" show-overflow-tooltip>
      <template #default="{ row }">
        <el-select v-model="row.functionCode" class="w-full">
          <el-option
            v-for="option in functionCodeOptions"
            :key="option.value"
            :label="option.label"
            :value="option.value"
          />
        </el-select>
      </template>
    </el-table-column>

    <el-table-column label="地址" width="88">
      <template #default="{ row }">
        <el-input-number
          v-model="row.address"
          :min="0"
          :max="65535"
          controls-position="right"
          class="w-full"
        />
      </template>
    </el-table-column>

    <el-table-column label="数量" width="88">
      <template #default="{ row }">
        <el-input-number
          v-model="row.quantity"
          :min="1"
          :max="125"
          controls-position="right"
          class="w-full"
        />
      </template>
    </el-table-column>

    <el-table-column label="指令" min-width="128" show-overflow-tooltip>
      <template #default="{ row }">
        <el-input
          v-model="row.instruction"
          placeholder="写入值，例: 1 或 1,0"
          :disabled="!isWriteCode(row.functionCode)"
        />
      </template>
    </el-table-column>

    <el-table-column label="状态" min-width="120" show-overflow-tooltip>
      <template #default="{ row }">
        <div class="status-cell">
          <el-tag
            v-if="props.pollRunning && props.pollingRowId === row.id"
            type="success"
            effect="dark"
          >
            轮询中
          </el-tag>
          <el-tag
            v-else-if="props.autoReadRunning && props.autoReadRowId === row.id"
            type="warning"
            effect="plain"
          >
            自动刷新中
          </el-tag>
          <el-tag v-else type="info" effect="plain"> 空闲 </el-tag>
          <el-tag v-if="row.collectEnabled" type="success" effect="plain">
            写库
          </el-tag>
          <span v-if="row.lastError" class="error-text" :title="row.lastError">
            {{ row.lastError }}
          </span>
        </div>
      </template>
    </el-table-column>

    <el-table-column label="操作" min-width="216">
      <template #default="{ row }">
        <div class="actions">
          <el-button
            size="small"
            type="primary"
            :loading="props.loadingRowId === row.id"
            :disabled="!isReadCode(row.functionCode)"
            @click="emit('read', row)"
          >
            读
          </el-button>
          <el-button
            size="small"
            type="success"
            plain
            :loading="props.loadingRowId === row.id"
            :disabled="!isWriteCode(row.functionCode)"
            @click="emit('write', row)"
          >
            写
          </el-button>
          <el-button
            size="small"
            type="warning"
            plain
            :disabled="!isReadCode(row.functionCode)"
            @click="emit('poll-start', row)"
          >
            轮询
          </el-button>
          <el-button
            size="small"
            type="danger"
            plain
            @click="emit('remove', row.id)"
          >
            删除
          </el-button>
        </div>
      </template>
    </el-table-column>
  </el-table>
</template>

<style scoped>
@reference "../../../style/tailwind.css";

.modbus-table :deep(.el-input-number) {
  width: 100%;
}

.modbus-table :deep(.el-table__cell) {
  padding: 6px 4px;
}

.value-cell {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.value-cell small {
  font-size: 11px;
  color: rgb(148 163 184);
}

.status-cell {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.error-text {
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 12px;
  color: rgb(220 38 38);
  white-space: nowrap;
}

.actions {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
}

@media (width <= 768px) {
  .actions {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
}
</style>
