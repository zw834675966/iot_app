import { message } from "@/utils/message";
import {
  POLL_READ_RETRY_MAX,
  POLL_READ_RETRY_MIN,
  POLL_READ_TIMEOUT_MAX_MS,
  POLL_READ_TIMEOUT_MIN_MS,
  type ModbusChannel
} from "./modbusPage.types";

type UsePollTimeoutAlarmOptions = {
  clearSnapshotTimer: () => void;
};

const READ_TIMEOUT_TAG = "modbus read timeout";
const CHANNEL_NOT_FOUND_TAG = "channel not found";
const TIMEOUT_TAG = "timeout";
const TIMEOUT_CN_TAG = "超时";

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, Math.round(value)));
}

export function usePollTimeoutAlarm(options: UsePollTimeoutAlarmOptions) {
  function normalizePollTimeoutConfig(channel: ModbusChannel) {
    channel.pollReadTimeoutMs = clamp(
      channel.pollReadTimeoutMs,
      POLL_READ_TIMEOUT_MIN_MS,
      POLL_READ_TIMEOUT_MAX_MS
    );
    channel.pollReadRetryCount = clamp(
      channel.pollReadRetryCount,
      POLL_READ_RETRY_MIN,
      POLL_READ_RETRY_MAX
    );
  }

  function resetPollTimeoutState(channel: ModbusChannel) {
    channel.pollReadTimeoutHitCount = 0;
    channel.pollReadAlarm = false;
  }

  function buildAlarmText(channel: ModbusChannel): string {
    const threshold = channel.pollReadTimeoutMs * channel.pollReadRetryCount;
    return `轮询读异常报警：超时 ${channel.pollReadTimeoutMs}ms × 重发 ${channel.pollReadRetryCount} 次 = ${threshold}ms`;
  }

  function isPollingTimeoutLikeError(errorMessage: string): boolean {
    const normalized = errorMessage.toLowerCase();
    return (
      normalized.includes(READ_TIMEOUT_TAG) ||
      normalized.includes(CHANNEL_NOT_FOUND_TAG) ||
      normalized.includes(TIMEOUT_TAG) ||
      errorMessage.includes(TIMEOUT_CN_TAG)
    );
  }

  function handlePollingError(channel: ModbusChannel, errorMessage: string) {
    normalizePollTimeoutConfig(channel);

    if (!isPollingTimeoutLikeError(errorMessage)) {
      resetPollTimeoutState(channel);
      return;
    }

    channel.pollReadTimeoutHitCount += 1;

    const threshold = channel.pollReadTimeoutMs * channel.pollReadRetryCount;
    const elapsedByRule =
      channel.pollReadTimeoutMs * channel.pollReadTimeoutHitCount;

    if (elapsedByRule < threshold) {
      channel.pollReadAlarm = false;
      return;
    }

    if (!channel.pollReadAlarm) {
      message(buildAlarmText(channel), { type: "error" });
    }

    channel.pollReadAlarm = true;
    channel.pollRunning = false;
    channel.connected = false;
    channel.endpoint = "";
    channel.pollingRowId = "";
    options.clearSnapshotTimer();
  }

  function applyPollTimeoutConfig(channel: ModbusChannel) {
    normalizePollTimeoutConfig(channel);
    message(
      `轮询超时策略已应用：${channel.pollReadTimeoutMs}ms × ${channel.pollReadRetryCount} 次`,
      {
        type: "success"
      }
    );
  }

  function getPollAlarmMessage(channel: ModbusChannel): string {
    if (!channel.pollReadAlarm) return "";
    return buildAlarmText(channel);
  }

  return {
    normalizePollTimeoutConfig,
    resetPollTimeoutState,
    handlePollingError,
    applyPollTimeoutConfig,
    getPollAlarmMessage
  };
}
