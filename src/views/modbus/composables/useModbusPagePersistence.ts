import { isTauri } from "@tauri-apps/api/core";
import {
  BaseDirectory,
  exists,
  mkdir,
  readTextFile,
  writeTextFile
} from "@tauri-apps/plugin-fs";
import {
  AUTO_READ_MAX_INTERVAL_MS,
  AUTO_READ_MIN_INTERVAL_MS,
  POLL_READ_RETRY_MAX,
  POLL_READ_RETRY_MIN,
  POLL_READ_TIMEOUT_MAX_MS,
  POLL_READ_TIMEOUT_MIN_MS,
  buildChannelFromGateway,
  buildDefaultGateway,
  createPointRow,
  defaultRows,
  type ModbusChannel,
  type ModbusGateway,
  type ModbusPointRow
} from "./modbusPage.types";

type ModbusPageState = {
  activeChannelKey: string;
  gateways: ModbusGateway[];
  channels: ModbusChannel[];
  pointsByChannel: Record<string, ModbusPointRow[]>;
};

type PersistedEnvelope = {
  version: 1;
  state: ModbusPageState;
};

const STORAGE_DIR = "modbus";
const STORAGE_FILE = `${STORAGE_DIR}/page-state.json`;
const STORAGE_KEY = "modbus-page-state";

function toInteger(value: unknown, fallback: number): number {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.trunc(parsed);
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function sanitizeGateway(raw: unknown, index: number): ModbusGateway {
  const fallback = buildDefaultGateway(index);
  const source = raw as Partial<ModbusGateway> | undefined;
  const id = String(source?.id ?? "").trim();
  const name = String(source?.name ?? "").trim();
  return {
    id: id || fallback.id,
    name: name || fallback.name
  };
}

function sanitizeChannel(
  raw: unknown,
  gatewayMap: Map<string, ModbusGateway>
): ModbusChannel | null {
  const source = raw as Partial<ModbusChannel> | undefined;
  const gatewayId = String(source?.gatewayId ?? "").trim();
  const gateway = gatewayMap.get(gatewayId);
  if (!gateway) return null;

  const slaveId = clamp(toInteger(source?.slaveId, 1), 1, 247);
  const base = buildChannelFromGateway(gateway, slaveId);
  const key = String(source?.key ?? "").trim() || base.key;

  return {
    ...base,
    key,
    label: String(source?.label ?? "").trim() || base.label,
    stationCode: String(source?.stationCode ?? "").trim() || base.stationCode,
    host: String(source?.host ?? "").trim() || base.host,
    port: clamp(toInteger(source?.port, 502), 1, 65535),
    slaveId,
    pollIntervalMs: clamp(toInteger(source?.pollIntervalMs, 1000), 100, 60_000),
    autoReadEnabled: Boolean(source?.autoReadEnabled),
    autoReadIntervalMs: clamp(
      toInteger(source?.autoReadIntervalMs, 1000),
      AUTO_READ_MIN_INTERVAL_MS,
      AUTO_READ_MAX_INTERVAL_MS
    ),
    pollReadTimeoutMs: clamp(
      toInteger(source?.pollReadTimeoutMs, 5000),
      POLL_READ_TIMEOUT_MIN_MS,
      POLL_READ_TIMEOUT_MAX_MS
    ),
    pollReadRetryCount: clamp(
      toInteger(source?.pollReadRetryCount, 3),
      POLL_READ_RETRY_MIN,
      POLL_READ_RETRY_MAX
    ),
    connected: false,
    endpoint: "",
    pollRunning: false,
    pollingRowId: "",
    lastError: "",
    pollReadTimeoutHitCount: 0,
    pollReadAlarm: false,
    updatedAt: undefined
  };
}

function sanitizePoint(raw: unknown, index: number): ModbusPointRow {
  const source = raw as Partial<ModbusPointRow> | undefined;
  const fallback = createPointRow(index);
  const id = String(source?.id ?? "").trim() || fallback.id;
  return {
    id,
    name: String(source?.name ?? "").trim() || fallback.name,
    functionCode: toInteger(source?.functionCode, 3),
    address: Math.max(0, toInteger(source?.address, 0)),
    quantity: Math.max(1, toInteger(source?.quantity, 1)),
    instruction: String(source?.instruction ?? "1"),
    valueText: String(source?.valueText ?? "--"),
    collectEnabled: Boolean(source?.collectEnabled),
    lastError: "",
    updatedAt: undefined
  };
}

function normalizeState(raw: unknown): ModbusPageState | null {
  const source = raw as Partial<PersistedEnvelope> | null;
  const state = source?.state;
  if (!state) return null;

  const sourceGateways = Array.isArray(state.gateways) ? state.gateways : [];
  const gateways =
    sourceGateways.length > 0
      ? sourceGateways.map((item, index) => sanitizeGateway(item, index))
      : [buildDefaultGateway(0)];

  const gatewayMap = new Map(gateways.map(gateway => [gateway.id, gateway]));

  const seenChannelKeys = new Set<string>();
  const sourceChannels = Array.isArray(state.channels) ? state.channels : [];
  const channels = sourceChannels
    .map(item => sanitizeChannel(item, gatewayMap))
    .filter((item): item is ModbusChannel => Boolean(item))
    .filter(item => {
      if (seenChannelKeys.has(item.key)) return false;
      seenChannelKeys.add(item.key);
      return true;
    });

  if (channels.length === 0) {
    channels.push(buildChannelFromGateway(gateways[0], 1));
  }

  const sourcePointsByChannel =
    state.pointsByChannel && typeof state.pointsByChannel === "object"
      ? state.pointsByChannel
      : {};
  const pointsByChannel: Record<string, ModbusPointRow[]> = {};
  channels.forEach(channel => {
    const rawPoints = Array.isArray(sourcePointsByChannel[channel.key])
      ? sourcePointsByChannel[channel.key]
      : [];
    pointsByChannel[channel.key] =
      rawPoints.length > 0
        ? rawPoints.map((point, index) => sanitizePoint(point, index))
        : defaultRows();
  });

  const activeChannelKey = channels.some(
    channel => channel.key === state.activeChannelKey
  )
    ? state.activeChannelKey
    : channels[0].key;

  return {
    activeChannelKey,
    gateways,
    channels,
    pointsByChannel
  };
}

export async function loadModbusPageState(): Promise<ModbusPageState | null> {
  try {
    if (isTauri()) {
      const hasFile = await exists(STORAGE_FILE, {
        baseDir: BaseDirectory.AppConfig
      });
      if (!hasFile) return null;
      const raw = await readTextFile(STORAGE_FILE, {
        baseDir: BaseDirectory.AppConfig
      });
      return normalizeState(JSON.parse(raw));
    }

    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) return null;
    return normalizeState(JSON.parse(raw));
  } catch (error) {
    console.warn("[modbus] load persisted state failed", error);
    return null;
  }
}

export async function saveModbusPageState(
  state: ModbusPageState
): Promise<void> {
  try {
    const channelKeys = new Set(state.channels.map(channel => channel.key));
    const normalizedPoints: Record<string, ModbusPointRow[]> = {};
    state.channels.forEach(channel => {
      normalizedPoints[channel.key] = (
        state.pointsByChannel[channel.key] ?? []
      ).map((point, index) => sanitizePoint(point, index));
    });

    const payload: PersistedEnvelope = {
      version: 1,
      state: {
        activeChannelKey: channelKeys.has(state.activeChannelKey)
          ? state.activeChannelKey
          : (state.channels[0]?.key ?? ""),
        gateways: state.gateways.map((gateway, index) =>
          sanitizeGateway(gateway, index)
        ),
        channels: state.channels.map(channel => ({
          ...channel,
          connected: false,
          endpoint: "",
          pollRunning: false,
          pollingRowId: "",
          lastError: "",
          pollReadTimeoutHitCount: 0,
          pollReadAlarm: false,
          updatedAt: undefined
        })),
        pointsByChannel: normalizedPoints
      }
    };
    const content = JSON.stringify(payload);

    if (isTauri()) {
      await mkdir(STORAGE_DIR, {
        baseDir: BaseDirectory.AppConfig,
        recursive: true
      });
      await writeTextFile(STORAGE_FILE, content, {
        baseDir: BaseDirectory.AppConfig
      });
      return;
    }

    window.localStorage.setItem(STORAGE_KEY, content);
  } catch (error) {
    console.warn("[modbus] save persisted state failed", error);
  }
}
