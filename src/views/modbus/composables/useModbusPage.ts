import {
  modbusCollectPointsSet,
  modbusConfigLoad,
  modbusConfigSave,
  modbusConnect,
  modbusDisconnect,
  modbusPollSnapshot,
  modbusPollStart,
  modbusPollStop,
  modbusReadPoint,
  modbusSetPollInterval,
  modbusTelemetryIngest,
  modbusWritePoint,
  type ModbusReadPointData
} from "@/api/modbus";
import { message } from "@/utils/message";
import { computed, onMounted, onUnmounted, reactive, ref, watch } from "vue";
import {
  buildChannelFromGateway,
  buildDefaultGateway,
  createPointRowsByBatch,
  defaultRows,
  type ModbusBatchPointPayload,
  type ModbusChannel,
  type ModbusGateway,
  type ModbusPointRow
} from "./modbusPage.types";
import { useAutoReadRefresh } from "./useAutoReadRefresh";
import {
  loadModbusPageState,
  saveModbusPageState
} from "./useModbusPagePersistence";
import { usePollTimeoutAlarm } from "./usePollTimeoutAlarm";

export type {
  ModbusBatchPointPayload,
  ModbusChannel,
  ModbusGateway,
  ModbusPointRow
} from "./modbusPage.types";

export function useModbusPage() {
  const gateways = ref<ModbusGateway[]>([buildDefaultGateway(0)]);
  const channels = ref<ModbusChannel[]>([
    buildChannelFromGateway(gateways.value[0], 1)
  ]);
  const activeChannelKey = ref(channels.value[0].key);
  const pointsByChannel = ref<Record<string, ModbusPointRow[]>>({
    [channels.value[0].key]: defaultRows()
  });
  const points = computed(() => {
    const channel = activeChannel.value;
    if (!channel) return [];
    return pointsByChannel.value[channel.key] ?? [];
  });

  const loading = reactive({
    connect: false,
    disconnect: false,
    snapshot: false,
    rowId: "",
    sync: false,
    collect: false
  });

  const syncMode = ref<"online" | "offline">("offline");
  const selectedPointIds = ref<string[]>([]);
  const collectingPointIds = ref<string[]>([]);
  const collectingTimer = ref<number | undefined>();

  let snapshotTimer: number | undefined;
  let persistTimer: number | undefined;
  const restoringState = ref(true);

  const activeChannel = computed(() => {
    return (
      channels.value.find(channel => channel.key === activeChannelKey.value) ??
      channels.value[0]
    );
  });

  const activeGateway = computed(() => {
    const channel = activeChannel.value;
    if (!channel) return undefined;
    return gateways.value.find(item => item.id === channel.gatewayId);
  });

  const channelsByGateway = computed(() => {
    return gateways.value.map(gateway => ({
      ...gateway,
      channels: channels.value.filter(
        channel => channel.gatewayId === gateway.id
      )
    }));
  });

  function getPointsByChannelKey(channelKey: string): ModbusPointRow[] {
    if (!pointsByChannel.value[channelKey]) {
      pointsByChannel.value[channelKey] = defaultRows();
    }
    return pointsByChannel.value[channelKey];
  }

  function patchPoint(
    channelKey: string,
    pointId: string,
    patch: Partial<ModbusPointRow>
  ) {
    const list = getPointsByChannelKey(channelKey);
    const pointIndex = list.findIndex(point => point.id === pointId);
    if (pointIndex < 0) return;
    list[pointIndex] = {
      ...list[pointIndex],
      ...patch
    };
    pointsByChannel.value[channelKey] = [...list];
  }

  function getChannelOrWarn(): ModbusChannel | undefined {
    const channel = activeChannel.value;
    if (!channel) {
      message("请先选择通道", { type: "warning" });
      return undefined;
    }
    return channel;
  }

  function clearSnapshotTimer() {
    if (snapshotTimer !== undefined) {
      window.clearInterval(snapshotTimer);
      snapshotTimer = undefined;
    }
  }

  function clearPersistTimer() {
    if (persistTimer !== undefined) {
      window.clearTimeout(persistTimer);
      persistTimer = undefined;
    }
  }

  function schedulePersist() {
    if (restoringState.value) return;
    clearPersistTimer();
    persistTimer = window.setTimeout(() => {
      void saveModbusPageState({
        activeChannelKey: activeChannelKey.value,
        gateways: gateways.value,
        channels: channels.value,
        pointsByChannel: pointsByChannel.value
      });
    }, 300);
  }

  async function restorePersistedState() {
    const persisted = await loadModbusPageState();
    if (persisted) {
      gateways.value = persisted.gateways;
      channels.value = persisted.channels;
      pointsByChannel.value = persisted.pointsByChannel;
      activeChannelKey.value = persisted.activeChannelKey;
    }
    restoringState.value = false;
  }

  function stopCollectingTimer() {
    if (collectingTimer.value !== undefined) {
      window.clearInterval(collectingTimer.value);
      collectingTimer.value = undefined;
    }
  }

  function buildConfigPayload(autoRenameDuplicateGateway: boolean) {
    return {
      mode: syncMode.value,
      activeChannelKey: activeChannelKey.value,
      autoRenameDuplicateGateway,
      gateways: gateways.value.map(gateway => ({
        id: gateway.id,
        name: gateway.name
      })),
      channels: channels.value.map(channel => ({
        key: channel.key,
        label: channel.label,
        gatewayId: channel.gatewayId,
        stationCode: channel.stationCode,
        host: channel.host,
        port: channel.port,
        slaveId: channel.slaveId,
        pollIntervalMs: channel.pollIntervalMs,
        autoReadEnabled: channel.autoReadEnabled,
        autoReadIntervalMs: channel.autoReadIntervalMs,
        pollReadTimeoutMs: channel.pollReadTimeoutMs,
        pollReadRetryCount: channel.pollReadRetryCount
      })),
      pointsByChannel: Object.fromEntries(
        Object.entries(pointsByChannel.value).map(([channelKey, rows]) => [
          channelKey,
          rows.map(row => ({
            id: row.id,
            name: row.name,
            functionCode: row.functionCode,
            address: row.address,
            quantity: row.quantity,
            instruction: row.instruction,
            collectEnabled: row.collectEnabled
          }))
        ])
      )
    };
  }

  async function loadConfigFromDatabase() {
    loading.sync = true;
    try {
      const response = await modbusConfigLoad({
        activeChannelKey: activeChannelKey.value
      });
      gateways.value = response.data.gateways;
      channels.value = response.data.channels.map(channel => ({
        ...channel,
        connected: false,
        endpoint: "",
        pollRunning: false,
        pollingRowId: "",
        lastError: "",
        pollReadTimeoutHitCount: 0,
        pollReadAlarm: false
      }));
      pointsByChannel.value = Object.fromEntries(
        Object.entries(response.data.pointsByChannel).map(
          ([channelKey, rows]) => [
            channelKey,
            rows.map(row => ({
              ...row,
              valueText: "--",
              lastError: "",
              updatedAt: undefined
            }))
          ]
        )
      );
      activeChannelKey.value =
        response.data.activeChannelKey || channels.value[0]?.key || "";
      selectedPointIds.value = [];
      collectingPointIds.value = [];
      stopCollectingTimer();
      message("已加载在线配置", { type: "success" });
    } catch (error: any) {
      message(error?.message ?? "读取数据库配置失败", { type: "error" });
    } finally {
      loading.sync = false;
    }
  }

  async function saveConfigToDatabase(autoRenameDuplicateGateway = false) {
    loading.sync = true;
    try {
      const response = await modbusConfigSave(
        buildConfigPayload(autoRenameDuplicateGateway)
      );
      message(
        `配置已保存(${response.data.mode}): ${response.data.savedGatewayNames.join(
          ", "
        )}`,
        { type: "success" }
      );
      if (syncMode.value === "online") {
        await loadConfigFromDatabase();
      }
      return true;
    } catch (error: any) {
      const errorMessage = error?.message ?? "保存配置失败";
      if (
        syncMode.value === "offline" &&
        !autoRenameDuplicateGateway &&
        errorMessage.includes("duplicate gateway name")
      ) {
        return false;
      }
      message(errorMessage, { type: "error" });
      throw error;
    } finally {
      loading.sync = false;
    }
  }

  async function setConfigMode(nextMode: "online" | "offline") {
    if (syncMode.value === nextMode) return;
    syncMode.value = nextMode;
    if (nextMode === "online") {
      await loadConfigFromDatabase();
    } else {
      message("已切换为离线配置模式", { type: "info" });
    }
  }

  function onPointSelectionChange(pointIds: string[]) {
    selectedPointIds.value = pointIds;
  }

  async function applyCollectPointsToDatabase(applyAll: boolean) {
    const channel = getChannelOrWarn();
    if (!channel) return;

    const ids = applyAll
      ? getPointsByChannelKey(channel.key).map(row => row.id)
      : [...selectedPointIds.value];
    if (!applyAll && ids.length === 0) {
      message("请先勾选点位", { type: "warning" });
      return;
    }

    loading.collect = true;
    try {
      const response = await modbusCollectPointsSet({
        channelKey: channel.key,
        pointIds: ids,
        applyAll
      });
      const allowedSet = new Set(ids);
      getPointsByChannelKey(channel.key).forEach(row => {
        row.collectEnabled = applyAll ? true : allowedSet.has(row.id);
      });
      collectingPointIds.value = getPointsByChannelKey(channel.key)
        .filter(row => row.collectEnabled)
        .map(row => row.id);
      message(`已写入数据库采集点位 ${response.data.updatedCount} 条`, {
        type: "success"
      });
      ensureCollectingTimer();
    } catch (error: any) {
      message(error?.message ?? "写入数据库采集点位失败", { type: "error" });
    } finally {
      loading.collect = false;
    }
  }

  async function collectSelectedTelemetry() {
    const channel = activeChannel.value;
    if (!channel?.connected) return;

    const targetPoints = getPointsByChannelKey(channel.key).filter(
      row => row.collectEnabled
    );
    if (targetPoints.length === 0) return;

    const gatewayName = activeGateway.value?.name ?? "";
    if (!gatewayName) return;

    const samples: {
      sampledAtMillis?: number;
      gatewayName: string;
      channelKey: string;
      host: string;
      port: number;
      slaveId: number;
      pointId: string;
      pointName: string;
      functionCode: number;
      address: number;
      quantity: number;
      values: string[];
    }[] = [];

    for (const point of targetPoints) {
      const ok = await readPointById(point.id, {
        silentSuccess: true,
        silentError: true,
        showLoading: false
      });
      if (!ok) continue;
      const values = point.valueText
        .split(",")
        .map(value => value.trim())
        .filter(Boolean);
      samples.push({
        sampledAtMillis: Date.now(),
        gatewayName,
        channelKey: channel.key,
        host: channel.host,
        port: channel.port,
        slaveId: channel.slaveId,
        pointId: point.id,
        pointName: point.name,
        functionCode: point.functionCode,
        address: point.address,
        quantity: point.quantity,
        values
      });
    }
    if (samples.length === 0) return;
    await modbusTelemetryIngest({ samples });
  }

  function ensureCollectingTimer() {
    stopCollectingTimer();
    const channel = activeChannel.value;
    if (!channel?.connected) return;
    if (collectingPointIds.value.length === 0) return;
    collectingTimer.value = window.setInterval(
      () => {
        void collectSelectedTelemetry().catch(error => {
          console.warn("[modbus] collect telemetry failed", error);
        });
      },
      Math.max(100, Number(channel.autoReadIntervalMs || 1000))
    );
  }

  function markPointFromRead(
    channelKey: string,
    pointId: string,
    readData: ModbusReadPointData
  ) {
    patchPoint(channelKey, pointId, {
      valueText: readData.values.join(", "),
      updatedAt: Date.now(),
      lastError: ""
    });
  }

  async function readPointById(
    pointId: string,
    customOptions?: Partial<{
      silentSuccess: boolean;
      silentError: boolean;
      showLoading: boolean;
    }>
  ): Promise<boolean> {
    const options = {
      silentSuccess: false,
      silentError: false,
      showLoading: true,
      ...(customOptions ?? {})
    };

    const channel = activeChannel.value;
    if (!channel?.connected) {
      if (!options.silentError) {
        message("请先连接通道", { type: "warning" });
      }
      return false;
    }

    const point = getPointsByChannelKey(channel.key).find(
      item => item.id === pointId
    );
    if (!point) {
      if (!options.silentError) {
        message("点位不存在或已删除", { type: "warning" });
      }
      return false;
    }

    if (options.showLoading) {
      loading.rowId = pointId;
    }

    try {
      const response = await modbusReadPoint({
        channelKey: channel.key,
        slaveId: Number(channel.slaveId),
        functionCode: Number(point.functionCode),
        address: Number(point.address),
        quantity: Number(point.quantity)
      });
      markPointFromRead(channel.key, pointId, response.data);
      if (!options.silentSuccess) {
        message(`读取成功: ${point.name || point.id}`, { type: "success" });
      }
      return true;
    } catch (error: any) {
      const errorMessage = error?.message ?? "读取失败";
      patchPoint(channel.key, pointId, {
        lastError: errorMessage,
        updatedAt: Date.now()
      });
      channel.lastError = errorMessage;
      channel.updatedAt = Date.now();
      if (!options.silentError) {
        message(errorMessage, { type: "error" });
      }
      return false;
    } finally {
      if (options.showLoading && loading.rowId === pointId) {
        loading.rowId = "";
      }
    }
  }

  const {
    isAutoReadRunning,
    autoReadPointId,
    startAutoRead,
    stopAutoRead,
    setAutoReadEnabled,
    updateAutoReadInterval,
    stopForChannel,
    stopForPoint,
    cleanupAutoRead
  } = useAutoReadRefresh({
    activeChannel,
    readPointSilently: pointId => {
      return readPointById(pointId, {
        silentSuccess: true,
        silentError: true,
        showLoading: false
      });
    }
  });

  const {
    normalizePollTimeoutConfig,
    resetPollTimeoutState,
    handlePollingError,
    applyPollTimeoutConfig,
    getPollAlarmMessage
  } = usePollTimeoutAlarm({
    clearSnapshotTimer
  });

  async function syncSnapshot() {
    const channel = getChannelOrWarn();
    if (!channel?.connected) return;

    loading.snapshot = true;
    try {
      const response = await modbusPollSnapshot({ channelKey: channel.key });
      const snapshot = response.data;
      channel.pollRunning = snapshot.running;
      channel.pollIntervalMs = snapshot.intervalMs;
      channel.lastError = snapshot.lastError ?? "";
      channel.updatedAt = snapshot.updatedAt;

      if (snapshot.lastError) {
        handlePollingError(channel, snapshot.lastError);
      } else {
        resetPollTimeoutState(channel);
      }

      if (snapshot.latest && channel.pollingRowId) {
        const target = getPointsByChannelKey(channel.key).find(
          point => point.id === channel.pollingRowId
        );
        if (target) {
          markPointFromRead(channel.key, target.id, snapshot.latest);
        }
      }

      if (!snapshot.running) {
        channel.pollingRowId = "";
        clearSnapshotTimer();
      }
    } catch (error: any) {
      channel.lastError = error?.message ?? "轮询快照获取失败";
      channel.updatedAt = Date.now();
      handlePollingError(channel, channel.lastError);
    } finally {
      loading.snapshot = false;
    }
  }

  function ensureSnapshotTimer() {
    clearSnapshotTimer();
    snapshotTimer = window.setInterval(() => {
      void syncSnapshot();
    }, 1000);
  }

  async function connectChannel() {
    const channel = getChannelOrWarn();
    if (!channel) return;

    loading.connect = true;
    try {
      const response = await modbusConnect({
        channelKey: channel.key,
        host: channel.host,
        port: Number(channel.port),
        slaveId: Number(channel.slaveId)
      });

      channel.connected = true;
      channel.endpoint = response.data.endpoint;
      channel.pollIntervalMs = response.data.pollIntervalMs;
      channel.lastError = "";
      resetPollTimeoutState(channel);
      message(`通道已连接: ${response.data.endpoint}`, { type: "success" });
      await syncSnapshot();
      ensureCollectingTimer();
    } catch (error: any) {
      channel.connected = false;
      channel.endpoint = "";
      message(error?.message ?? "Modbus 连接失败", { type: "error" });
    } finally {
      loading.connect = false;
    }
  }

  async function disconnectChannel() {
    const channel = getChannelOrWarn();
    if (!channel) return;

    loading.disconnect = true;
    try {
      await modbusDisconnect({ channelKey: channel.key });
      stopAutoRead();
      channel.connected = false;
      channel.pollRunning = false;
      channel.pollingRowId = "";
      channel.endpoint = "";
      channel.lastError = "";
      resetPollTimeoutState(channel);
      clearSnapshotTimer();
      stopCollectingTimer();
      message("通道已断开", { type: "success" });
    } catch (error: any) {
      message(error?.message ?? "断开连接失败", { type: "error" });
    } finally {
      loading.disconnect = false;
    }
  }

  async function updatePollInterval() {
    const channel = getChannelOrWarn();
    if (!channel?.connected) {
      message("请先连接通道", { type: "warning" });
      return;
    }

    try {
      const response = await modbusSetPollInterval({
        channelKey: channel.key,
        intervalMs: Number(channel.pollIntervalMs)
      });
      channel.pollIntervalMs = response.data.intervalMs;
      message(`轮询周期已更新为 ${channel.pollIntervalMs} ms`, {
        type: "success"
      });
    } catch (error: any) {
      message(error?.message ?? "设置轮询周期失败", { type: "error" });
    }
  }

  async function readPoint(row: ModbusPointRow) {
    const success = await readPointById(row.id);
    if (!success) return;
    startAutoRead(row.id);
  }

  async function writePoint(row: ModbusPointRow) {
    const channel = getChannelOrWarn();
    if (!channel?.connected) {
      message("请先连接通道", { type: "warning" });
      return;
    }

    if (![5, 6, 15, 16].includes(Number(row.functionCode))) {
      message("写入仅支持功能码 05/06/15/16", { type: "warning" });
      return;
    }

    loading.rowId = row.id;
    try {
      await modbusWritePoint({
        channelKey: channel.key,
        slaveId: Number(channel.slaveId),
        functionCode: Number(row.functionCode),
        address: Number(row.address),
        instruction: row.instruction
      });
      patchPoint(channel.key, row.id, {
        lastError: "",
        updatedAt: Date.now()
      });
      message(`写入成功: ${row.name || row.id}`, { type: "success" });
    } catch (error: any) {
      const errorMessage = error?.message ?? "写入失败";
      patchPoint(channel.key, row.id, {
        lastError: errorMessage,
        updatedAt: Date.now()
      });
      message(errorMessage, { type: "error" });
    } finally {
      loading.rowId = "";
    }
  }

  async function startPolling(row: ModbusPointRow) {
    const channel = getChannelOrWarn();
    if (!channel?.connected) {
      message("请先连接通道", { type: "warning" });
      return;
    }
    if (![1, 2, 3, 4].includes(Number(row.functionCode))) {
      message("轮询仅支持读功能码 01/02/03/04", { type: "warning" });
      return;
    }

    stopAutoRead();
    normalizePollTimeoutConfig(channel);
    resetPollTimeoutState(channel);

    try {
      const response = await modbusPollStart({
        channelKey: channel.key,
        slaveId: Number(channel.slaveId),
        functionCode: Number(row.functionCode),
        address: Number(row.address),
        quantity: Number(row.quantity),
        intervalMs: Number(channel.pollIntervalMs)
      });
      channel.pollRunning = response.data.running;
      channel.pollIntervalMs = response.data.intervalMs;
      channel.pollingRowId = row.id;
      channel.lastError = "";
      ensureSnapshotTimer();
      message(`轮询已启动: ${row.name || row.id}`, { type: "success" });
      await syncSnapshot();
      ensureCollectingTimer();
    } catch (error: any) {
      message(error?.message ?? "启动轮询失败", { type: "error" });
    }
  }

  async function stopPolling() {
    const channel = getChannelOrWarn();
    if (!channel?.connected) {
      message("请先连接通道", { type: "warning" });
      return;
    }

    try {
      await modbusPollStop({ channelKey: channel.key });
      channel.pollRunning = false;
      channel.pollingRowId = "";
      resetPollTimeoutState(channel);
      clearSnapshotTimer();
      stopCollectingTimer();
      message("轮询已停止", { type: "success" });
      await syncSnapshot();
    } catch (error: any) {
      message(error?.message ?? "停止轮询失败", { type: "error" });
    }
  }

  function addGatewayWithRange(payload: {
    name: string;
    startSlaveId: number;
    endSlaveId: number;
  }) {
    const name = payload.name.trim();
    const start = Number(payload.startSlaveId);
    const end = Number(payload.endSlaveId);

    if (!name) {
      message("请输入网关名称", { type: "warning" });
      return false;
    }
    if (start < 1 || end > 247 || start > end) {
      message("从站范围无效，请输入 1-247 且起始 <= 结束", {
        type: "warning"
      });
      return false;
    }

    const gateway = buildDefaultGateway(gateways.value.length);
    gateway.name = name;
    gateways.value.push(gateway);

    const createdChannels = Array.from({ length: end - start + 1 }, (_, idx) =>
      buildChannelFromGateway(gateway, start + idx)
    );
    channels.value.push(...createdChannels);
    createdChannels.forEach(channel => {
      pointsByChannel.value[channel.key] = defaultRows();
    });
    stopAutoRead();
    activeChannelKey.value = createdChannels[0]?.key ?? activeChannelKey.value;
    return true;
  }

  function updateGatewayName(gatewayId: string, nextName: string) {
    const gateway = gateways.value.find(item => item.id === gatewayId);
    if (!gateway) return;
    const normalized = nextName.trim();
    if (!normalized) return;
    gateway.name = normalized;
  }

  function setGatewayNameDraft(gatewayId: string, nextName: string) {
    const gateway = gateways.value.find(item => item.id === gatewayId);
    if (!gateway) return;
    gateway.name = nextName;
  }

  async function removeChannel(channelKey: string) {
    if (channels.value.length <= 1) {
      message("至少保留一个通道", { type: "warning" });
      return;
    }

    const target = channels.value.find(item => item.key === channelKey);
    if (target?.connected) {
      try {
        if (target.pollRunning) {
          await modbusPollStop({ channelKey: target.key });
        }
        await modbusDisconnect({ channelKey: target.key });
      } catch {
        // Ignore disconnect error during local channel removal.
      }
    }

    stopForChannel(channelKey);
    stopCollectingTimer();

    channels.value = channels.value.filter(
      channel => channel.key !== channelKey
    );
    delete pointsByChannel.value[channelKey];

    const linkedGatewayIds = new Set(
      channels.value.map(item => item.gatewayId)
    );
    gateways.value = gateways.value.filter(item =>
      linkedGatewayIds.has(item.id)
    );

    if (activeChannelKey.value === channelKey) {
      activeChannelKey.value = channels.value[0]?.key ?? "";
    }
  }

  function appendPointByBatch(payload: ModbusBatchPointPayload) {
    const channel = getChannelOrWarn();
    if (!channel) return false;

    const count = Number(payload.count);
    const functionCode = Number(payload.functionCode);
    const startAddress = Number(payload.startAddress);

    if (!Number.isInteger(count) || count < 1) {
      message("数据条数至少为 1", { type: "warning" });
      return false;
    }
    if (!Number.isInteger(startAddress) || startAddress < 0) {
      message("起始数据地址必须为 0 或正整数", { type: "warning" });
      return false;
    }

    const list = getPointsByChannelKey(channel.key);
    const nextRows = createPointRowsByBatch(
      {
        count,
        functionCode,
        startAddress
      },
      list.length
    );
    pointsByChannel.value[channel.key] = [...list, ...nextRows];
    return true;
  }

  function removePoint(id: string) {
    const channel = activeChannel.value;
    if (!channel) return;
    stopForPoint(id);
    pointsByChannel.value[channel.key] = getPointsByChannelKey(
      channel.key
    ).filter(item => item.id !== id);
  }

  watch(
    [gateways, channels, pointsByChannel, activeChannelKey],
    () => {
      schedulePersist();
      ensureCollectingTimer();
    },
    { deep: true }
  );

  onMounted(() => {
    void restorePersistedState();
  });

  onUnmounted(() => {
    clearSnapshotTimer();
    clearPersistTimer();
    stopCollectingTimer();
    void saveModbusPageState({
      activeChannelKey: activeChannelKey.value,
      gateways: gateways.value,
      channels: channels.value,
      pointsByChannel: pointsByChannel.value
    });
    cleanupAutoRead();
  });

  return {
    gateways,
    channels,
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
  };
}
