import { message } from "@/utils/message";
import { computed, reactive } from "vue";
import type { ComputedRef } from "vue";
import {
  AUTO_READ_MAX_INTERVAL_MS,
  AUTO_READ_MIN_INTERVAL_MS,
  type ModbusChannel
} from "./modbusPage.types";

type UseAutoReadRefreshOptions = {
  activeChannel: ComputedRef<ModbusChannel | undefined>;
  readPointSilently: (pointId: string) => Promise<boolean>;
};

export function useAutoReadRefresh(options: UseAutoReadRefreshOptions) {
  const autoRead = reactive({
    running: false,
    channelKey: "",
    pointId: ""
  });

  let autoReadTimer: number | undefined;

  const isAutoReadRunning = computed(() => {
    return Boolean(
      options.activeChannel.value &&
        autoRead.running &&
        autoRead.channelKey === options.activeChannel.value.key
    );
  });

  const autoReadPointId = computed(() => {
    if (!isAutoReadRunning.value) return "";
    return autoRead.pointId;
  });

  function normalizeAutoReadInterval(intervalMs: number): number {
    return Math.min(
      AUTO_READ_MAX_INTERVAL_MS,
      Math.max(AUTO_READ_MIN_INTERVAL_MS, Math.round(intervalMs))
    );
  }

  function clearAutoReadTimer() {
    if (autoReadTimer !== undefined) {
      window.clearInterval(autoReadTimer);
      autoReadTimer = undefined;
    }
  }

  function stopAutoRead(notify = false) {
    const shouldNotify = notify && autoRead.running;
    clearAutoReadTimer();
    autoRead.running = false;
    autoRead.channelKey = "";
    autoRead.pointId = "";
    if (shouldNotify) {
      message("非轮询自动刷新已停止", { type: "info" });
    }
  }

  async function tickAutoRead() {
    if (!autoRead.running) return;

    const channel = options.activeChannel.value;
    if (
      !channel ||
      !channel.connected ||
      channel.key !== autoRead.channelKey ||
      channel.pollRunning
    ) {
      stopAutoRead();
      return;
    }

    const success = await options.readPointSilently(autoRead.pointId);
    if (!success) {
      stopAutoRead();
    }
  }

  function restartAutoReadTimer() {
    clearAutoReadTimer();

    const channel = options.activeChannel.value;
    if (!channel || !autoRead.running || autoRead.channelKey !== channel.key) {
      return;
    }

    channel.autoReadIntervalMs = normalizeAutoReadInterval(
      channel.autoReadIntervalMs
    );

    autoReadTimer = window.setInterval(() => {
      void tickAutoRead();
    }, channel.autoReadIntervalMs);
  }

  function startAutoRead(pointId: string) {
    const channel = options.activeChannel.value;
    if (!channel?.connected || !channel.autoReadEnabled) return;

    if (channel.pollRunning) {
      message("后端轮询运行中，不能启动非轮询自动刷新", {
        type: "warning"
      });
      return;
    }

    const sameTarget =
      autoRead.running &&
      autoRead.channelKey === channel.key &&
      autoRead.pointId === pointId;

    channel.autoReadIntervalMs = normalizeAutoReadInterval(
      channel.autoReadIntervalMs
    );
    autoRead.running = true;
    autoRead.channelKey = channel.key;
    autoRead.pointId = pointId;
    restartAutoReadTimer();

    if (!sameTarget) {
      message(`已开启非轮询自动刷新：${channel.autoReadIntervalMs} ms`, {
        type: "success"
      });
    }
  }

  function setAutoReadEnabled(enabled: boolean) {
    const channel = options.activeChannel.value;
    if (!channel) return;

    channel.autoReadEnabled = enabled;
    channel.autoReadIntervalMs = normalizeAutoReadInterval(
      channel.autoReadIntervalMs
    );

    if (!enabled && autoRead.running && autoRead.channelKey === channel.key) {
      stopAutoRead(true);
      return;
    }

    if (enabled) {
      message("已启用非轮询自动刷新，手动读取成功后开始", {
        type: "info"
      });
    }
  }

  function updateAutoReadInterval() {
    const channel = options.activeChannel.value;
    if (!channel) return;

    channel.autoReadIntervalMs = normalizeAutoReadInterval(
      channel.autoReadIntervalMs
    );

    if (autoRead.running && autoRead.channelKey === channel.key) {
      restartAutoReadTimer();
      message(`自动刷新周期已更新为 ${channel.autoReadIntervalMs} ms`, {
        type: "success"
      });
      return;
    }

    message(`自动刷新周期已设置为 ${channel.autoReadIntervalMs} ms`, {
      type: "success"
    });
  }

  function stopForChannel(channelKey: string) {
    if (autoRead.running && autoRead.channelKey === channelKey) {
      stopAutoRead();
    }
  }

  function stopForPoint(pointId: string) {
    if (autoRead.running && autoRead.pointId === pointId) {
      stopAutoRead();
    }
  }

  function cleanupAutoRead() {
    stopAutoRead();
  }

  return {
    isAutoReadRunning,
    autoReadPointId,
    startAutoRead,
    stopAutoRead,
    setAutoReadEnabled,
    updateAutoReadInterval,
    stopForChannel,
    stopForPoint,
    cleanupAutoRead
  };
}
