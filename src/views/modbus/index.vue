<script setup lang="ts">
import { ElMessageBox } from "element-plus";
import { computed, reactive, ref } from "vue";
import ModbusPointTable from "./components/ModbusPointTable.vue";
import {
  useModbusPage,
  type ModbusBatchPointPayload,
  type ModbusPointRow
} from "./composables/useModbusPage";

defineOptions({
  name: "ModbusPage"
});

const {
  channelsByGateway,
  points,
  activeChannel,
  activeGateway,
  activeChannelKey,
  loading,
  syncMode,
  selectedPointIds,
  collectingPointIds,
  isAutoReadRunning,
  autoReadPointId,
  getPollAlarmMessage,
  connectChannel,
  disconnectChannel,
  updatePollInterval,
  applyPollTimeoutConfig,
  setAutoReadEnabled,
  updateAutoReadInterval,
  stopAutoRead,
  setConfigMode,
  loadConfigFromDatabase,
  saveConfigToDatabase,
  onPointSelectionChange,
  applyCollectPointsToDatabase,
  readPoint,
  writePoint,
  startPolling,
  stopPolling,
  syncSnapshot,
  addGatewayWithRange,
  setGatewayNameDraft,
  updateGatewayName,
  removeChannel,
  appendPointByBatch,
  removePoint
} = useModbusPage();

const addGatewayDialogVisible = ref(false);
const addPointDialogVisible = ref(false);
const addGatewayForm = reactive({
  name: "",
  startSlaveId: 1,
  endSlaveId: 1
});
const addPointForm = reactive<ModbusBatchPointPayload>({
  count: 1,
  functionCode: 3,
  startAddress: 0
});

const pointFunctionCodeOptions = [
  { value: 1, label: "01 读线圈" },
  { value: 2, label: "02 读离散输入" },
  { value: 3, label: "03 读保持寄存器" },
  { value: 4, label: "04 读输入寄存器" },
  { value: 5, label: "05 写单线圈" },
  { value: 6, label: "06 写单寄存器" },
  { value: 15, label: "15 写多线圈" },
  { value: 16, label: "16 写多寄存器" }
];

function nowText(timestamp?: number): string {
  if (!timestamp) return "--";
  return new Date(timestamp).toLocaleString();
}

const connectionStatusText = computed(() => {
  if (!activeChannel.value) return "未选择通道";
  return activeChannel.value.connected ? "已连接" : "未连接";
});

const connectionStatusType = computed(() => {
  if (!activeChannel.value) return "info";
  return activeChannel.value.connected ? "success" : "info";
});

const pollAlarmMessage = computed(() => {
  if (!activeChannel.value) return "";
  return getPollAlarmMessage(activeChannel.value);
});

const configModeSwitchLabel = computed(() => {
  return syncMode.value === "online" ? "离线配置" : "在线配置";
});

const isOnlineConfig = computed(() => syncMode.value === "online");

function selectChannel(channelKey: string) {
  if (activeChannelKey.value === channelKey) return;
  stopAutoRead();
  activeChannelKey.value = channelKey;
  if (activeChannel.value?.connected) {
    void syncSnapshot();
  }
}

async function toggleConfigMode() {
  if (syncMode.value === "online") {
    await setConfigMode("offline");
    return;
  }
  await setConfigMode("online");
}

async function handleSaveConfig() {
  try {
    const saved = await saveConfigToDatabase(false);
    if (saved) return;

    await ElMessageBox.confirm("与数据库重名，是否名字后+1", "网关名称重名", {
      type: "warning",
      confirmButtonText: "确定",
      cancelButtonText: "取消"
    });
    await saveConfigToDatabase(true);
  } catch {
    // 已由 composable 处理错误提示
  }
}

function openAddGatewayDialog() {
  addGatewayForm.name = "";
  addGatewayForm.startSlaveId = 1;
  addGatewayForm.endSlaveId = 1;
  addGatewayDialogVisible.value = true;
}

function submitAddGateway() {
  const ok = addGatewayWithRange({
    name: addGatewayForm.name,
    startSlaveId: addGatewayForm.startSlaveId,
    endSlaveId: addGatewayForm.endSlaveId
  });
  if (!ok) return;
  addGatewayDialogVisible.value = false;
}

function handleGatewayNameChange(gatewayId: string, nextName: string) {
  updateGatewayName(gatewayId, nextName);
}

function handleGatewayNameInput(gatewayId: string, nextName: string) {
  setGatewayNameDraft(gatewayId, nextName);
}

function openAddPointDialog() {
  addPointForm.count = 1;
  addPointForm.functionCode = 3;
  addPointForm.startAddress = 0;
  addPointDialogVisible.value = true;
}

function submitAddPoint() {
  const ok = appendPointByBatch({
    count: addPointForm.count,
    functionCode: addPointForm.functionCode,
    startAddress: addPointForm.startAddress
  });
  if (!ok) return;
  addPointDialogVisible.value = false;
}

function handleRead(row: ModbusPointRow) {
  void readPoint(row);
}

function handleWrite(row: ModbusPointRow) {
  void writePoint(row);
}

function handlePollStart(row: ModbusPointRow) {
  void startPolling(row);
}

function handlePollStop() {
  void stopPolling();
}

function handleRefreshSnapshot() {
  void syncSnapshot();
}

function handleAutoReadEnabledChange(enabled: boolean) {
  setAutoReadEnabled(enabled);
}

function handleAutoReadIntervalApply() {
  updateAutoReadInterval();
}

function handlePollTimeoutApply() {
  if (!activeChannel.value) return;
  applyPollTimeoutConfig(activeChannel.value);
}

function handleAutoReadStop() {
  stopAutoRead(true);
}

function handlePointSelectionChange(pointIds: string[]) {
  onPointSelectionChange(pointIds);
}

function handleCollectSelectedToDb() {
  void applyCollectPointsToDatabase(false);
}

function handleCollectAllToDb() {
  void applyCollectPointsToDatabase(true);
}

async function handleRemoveChannel(channelKey: string) {
  if (isOnlineConfig.value) {
    try {
      await ElMessageBox.confirm("确认删除该通道配置？", "删除确认", {
        type: "warning",
        confirmButtonText: "删除",
        cancelButtonText: "取消"
      });
    } catch {
      return;
    }
  }
  await removeChannel(channelKey);
}

function handleRemovePoint(pointId: string) {
  if (!isOnlineConfig.value) {
    removePoint(pointId);
    return;
  }
  ElMessageBox.confirm("确认删除该点位配置？", "删除确认", {
    type: "warning",
    confirmButtonText: "删除",
    cancelButtonText: "取消"
  })
    .then(() => {
      removePoint(pointId);
    })
    .catch(() => {
      // ignore cancel
    });
}
</script>

<template>
  <div class="modbus-page p-3 sm:p-4 h-full">
    <div class="modbus-shell h-full w-full">
      <header class="modbus-header card-base shrink-0">
        <div>
          <h1 class="title">Modbus TCP 调试台</h1>
          <p class="subtitle">
            当前配置模式：{{ isOnlineConfig ? "在线配置" : "离线配置" }}
          </p>
        </div>
        <div v-if="activeChannel" class="status-wrap">
          <el-button size="small" plain @click="toggleConfigMode">
            {{ configModeSwitchLabel }}
          </el-button>
          <el-button
            size="small"
            type="primary"
            :loading="loading.sync"
            @click="handleSaveConfig"
          >
            存储
          </el-button>
          <el-button
            v-if="isOnlineConfig"
            size="small"
            type="info"
            plain
            :loading="loading.sync"
            @click="loadConfigFromDatabase"
          >
            读取在线配置
          </el-button>
          <el-tag :type="connectionStatusType" effect="dark">{{
            connectionStatusText
          }}</el-tag>
          <el-tag
            :type="activeChannel.pollRunning ? 'warning' : 'info'"
            effect="plain"
          >
            {{ activeChannel.pollRunning ? "轮询运行中" : "轮询已停止" }}
          </el-tag>
        </div>
      </header>

      <section class="modbus-layout flex-1 min-h-0">
        <aside class="sidebar card-base flex flex-col h-full">
          <div class="sidebar-head shrink-0">
            <h2>通道</h2>
            <el-button type="primary" link @click="openAddGatewayDialog"
              >新增</el-button
            >
          </div>
          <el-scrollbar class="flex-1 min-h-0">
            <div class="gateway-list">
              <section
                v-for="gateway in channelsByGateway"
                :key="gateway.id"
                class="gateway-item"
              >
                <div class="gateway-main">
                  <el-input
                    :model-value="gateway.name"
                    size="small"
                    @update:model-value="
                      value =>
                        handleGatewayNameInput(gateway.id, String(value ?? ''))
                    "
                    @change="
                      value =>
                        handleGatewayNameChange(gateway.id, String(value ?? ''))
                    "
                  />
                </div>
                <div class="gateway-children">
                  <button
                    v-for="channel in gateway.channels"
                    :key="channel.key"
                    class="channel-item"
                    :class="{ active: channel.key === activeChannelKey }"
                    type="button"
                    @click="selectChannel(channel.key)"
                  >
                    <div class="channel-main">
                      <span>{{ channel.label }}</span>
                      <el-tag
                        size="small"
                        :type="channel.connected ? 'success' : 'info'"
                        effect="light"
                      >
                        {{ channel.connected ? "在线" : "离线" }}
                      </el-tag>
                    </div>
                    <div class="channel-sub">
                      <span>站号 {{ channel.slaveId }}</span>
                      <el-button
                        type="danger"
                        link
                        @click.stop="handleRemoveChannel(channel.key)"
                        >删除</el-button
                      >
                    </div>
                  </button>
                </div>
              </section>
            </div>
          </el-scrollbar>
        </aside>

        <main v-if="activeChannel" class="content flex flex-col h-full min-h-0">
          <section class="card-base connection-panel shrink-0">
            <div class="panel-head">
              <h2>连接配置</h2>
              <div class="panel-actions">
                <el-button
                  size="small"
                  type="primary"
                  :loading="loading.connect"
                  @click="connectChannel"
                >
                  连接
                </el-button>
                <el-button
                  size="small"
                  type="danger"
                  plain
                  :loading="loading.disconnect"
                  @click="disconnectChannel"
                >
                  断开
                </el-button>
                <el-button
                  size="small"
                  type="info"
                  plain
                  @click="handleRefreshSnapshot"
                >
                  刷新快照
                </el-button>
              </div>
            </div>

            <el-form
              label-width="96px"
              class="connection-form"
              label-position="left"
            >
              <div class="form-grid">
                <el-form-item label="通道Key">
                  <el-input v-model="activeChannel.key" disabled />
                </el-form-item>
                <el-form-item label="网关名称">
                  <el-input
                    :model-value="activeGateway?.name ?? '--'"
                    disabled
                  />
                </el-form-item>
                <el-form-item label="主机地址">
                  <el-input
                    v-model="activeChannel.host"
                    placeholder="127.0.0.1"
                  />
                </el-form-item>
                <el-form-item label="端口">
                  <el-input-number
                    v-model="activeChannel.port"
                    :min="1"
                    :max="65535"
                    controls-position="right"
                    class="w-full"
                  />
                </el-form-item>
                <el-form-item label="站号">
                  <el-input-number
                    v-model="activeChannel.slaveId"
                    :min="1"
                    :max="247"
                    controls-position="right"
                    class="w-full"
                  />
                </el-form-item>
                <el-form-item label="轮询周期(ms)">
                  <el-input-number
                    v-model="activeChannel.pollIntervalMs"
                    :min="100"
                    :max="60000"
                    controls-position="right"
                    class="w-full"
                  />
                </el-form-item>
                <el-form-item label="轮询控制">
                  <div class="poll-actions">
                    <el-button
                      size="small"
                      plain
                      :disabled="!activeChannel.connected"
                      @click="updatePollInterval"
                    >
                      应用周期
                    </el-button>
                    <el-button
                      size="small"
                      type="warning"
                      plain
                      :disabled="!activeChannel.pollRunning"
                      @click="handlePollStop"
                    >
                      停止轮询
                    </el-button>
                  </div>
                </el-form-item>
                <el-form-item label="超时时间(ms)">
                  <el-input-number
                    v-model="activeChannel.pollReadTimeoutMs"
                    :min="100"
                    :max="60000"
                    controls-position="right"
                    class="w-full"
                  />
                </el-form-item>
                <el-form-item label="重发次数">
                  <el-input-number
                    v-model="activeChannel.pollReadRetryCount"
                    :min="1"
                    :max="20"
                    controls-position="right"
                    class="w-full"
                  />
                </el-form-item>
                <el-form-item label="超时策略">
                  <div class="poll-actions">
                    <el-button
                      size="small"
                      plain
                      @click="handlePollTimeoutApply"
                    >
                      应用超时策略
                    </el-button>
                    <span class="policy-tip">
                      报警阈值 = {{ activeChannel.pollReadTimeoutMs }}ms ×
                      {{ activeChannel.pollReadRetryCount }} 次 =
                      {{
                        activeChannel.pollReadTimeoutMs *
                        activeChannel.pollReadRetryCount
                      }}ms
                    </span>
                  </div>
                </el-form-item>
                <el-form-item label="自动刷新(非轮询)">
                  <el-switch
                    v-model="activeChannel.autoReadEnabled"
                    :disabled="!activeChannel.connected"
                    @change="handleAutoReadEnabledChange"
                  />
                </el-form-item>
                <el-form-item label="自动刷新周期(ms)">
                  <div class="poll-actions">
                    <el-input-number
                      v-model="activeChannel.autoReadIntervalMs"
                      :min="100"
                      :max="60000"
                      controls-position="right"
                      class="w-full"
                    />
                    <el-button
                      size="small"
                      plain
                      :disabled="!activeChannel.connected"
                      @click="handleAutoReadIntervalApply"
                    >
                      应用自动刷新周期
                    </el-button>
                    <el-button
                      size="small"
                      type="warning"
                      plain
                      :disabled="!isAutoReadRunning"
                      @click="handleAutoReadStop"
                    >
                      停止自动刷新
                    </el-button>
                  </div>
                </el-form-item>
              </div>
            </el-form>

            <div class="connection-meta">
              <span>端点: {{ activeChannel.endpoint || "--" }}</span>
              <span>快照时间: {{ nowText(activeChannel.updatedAt) }}</span>
              <span>
                轮询超时策略:
                {{ activeChannel.pollReadTimeoutMs }}ms ×
                {{ activeChannel.pollReadRetryCount }} 次
              </span>
              <span v-if="isAutoReadRunning">
                自动刷新点位: {{ autoReadPointId || "--" }}
              </span>
              <span v-if="pollAlarmMessage" class="alarm-text">
                {{ pollAlarmMessage }}
              </span>
              <span v-if="activeChannel.lastError" class="error-text">
                错误: {{ activeChannel.lastError }}
              </span>
            </div>
          </section>

          <section class="card-base points-panel flex-1 flex flex-col min-h-0">
            <div class="panel-head shrink-0">
              <h2>点位配置与调试</h2>
              <div class="panel-actions">
                <el-button
                  size="small"
                  type="primary"
                  plain
                  @click="openAddPointDialog"
                  >新增点位</el-button
                >
                <el-button
                  size="small"
                  type="warning"
                  plain
                  :disabled="!activeChannel.pollRunning"
                  @click="handlePollStop"
                >
                  停止当前轮询
                </el-button>
                <el-button
                  size="small"
                  type="success"
                  plain
                  :loading="loading.collect"
                  @click="handleCollectSelectedToDb"
                >
                  写入数据库(选中)
                </el-button>
                <el-button
                  size="small"
                  type="success"
                  :loading="loading.collect"
                  @click="handleCollectAllToDb"
                >
                  写入数据库(全部)
                </el-button>
              </div>
            </div>

            <div class="selection-meta mb-2">
              <span>已勾选: {{ selectedPointIds.length }} 条</span>
              <span>采集写库: {{ collectingPointIds.length }} 条</span>
            </div>

            <ModbusPointTable
              :points="points"
              :loading-row-id="loading.rowId"
              :polling-row-id="activeChannel.pollingRowId"
              :poll-running="activeChannel.pollRunning"
              :auto-read-row-id="autoReadPointId"
              :auto-read-running="isAutoReadRunning"
              :selected-point-ids="selectedPointIds"
              @read="handleRead"
              @write="handleWrite"
              @poll-start="handlePollStart"
              @remove="handleRemovePoint"
              @selection-change="handlePointSelectionChange"
            />
          </section>
        </main>
      </section>
    </div>
    <el-dialog
      v-model="addGatewayDialogVisible"
      title="新增网关范围"
      width="460px"
      destroy-on-close
    >
      <el-form label-width="110px" label-position="left">
        <el-form-item label="网关名称">
          <el-input
            v-model="addGatewayForm.name"
            placeholder="例如：网关A"
            maxlength="32"
          />
        </el-form-item>
        <el-form-item label="起始站号">
          <el-input-number
            v-model="addGatewayForm.startSlaveId"
            :min="1"
            :max="247"
            controls-position="right"
            class="w-full"
          />
        </el-form-item>
        <el-form-item label="结束站号">
          <el-input-number
            v-model="addGatewayForm.endSlaveId"
            :min="1"
            :max="247"
            controls-position="right"
            class="w-full"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addGatewayDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitAddGateway">确定</el-button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="addPointDialogVisible"
      title="新增点位"
      width="460px"
      destroy-on-close
    >
      <el-form label-width="110px" label-position="left">
        <el-form-item label="数据条数">
          <el-input-number
            v-model="addPointForm.count"
            :min="1"
            :max="500"
            controls-position="right"
            class="w-full"
          />
        </el-form-item>
        <el-form-item label="功能码">
          <el-select v-model="addPointForm.functionCode" class="w-full">
            <el-option
              v-for="option in pointFunctionCodeOptions"
              :key="option.value"
              :label="option.label"
              :value="option.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="起始数据地址">
          <el-input-number
            v-model="addPointForm.startAddress"
            :min="0"
            :max="65535"
            controls-position="right"
            class="w-full"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addPointDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitAddPoint">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped src="./modbus-page.css"></style>
