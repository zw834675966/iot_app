export type ModbusGateway = {
  id: string;
  name: string;
};

export type ModbusChannel = {
  key: string;
  label: string;
  gatewayId: string;
  stationCode: string;
  host: string;
  port: number;
  slaveId: number;
  pollIntervalMs: number;
  connected: boolean;
  endpoint: string;
  pollRunning: boolean;
  pollingRowId: string;
  lastError: string;
  updatedAt?: number;
  autoReadEnabled: boolean;
  autoReadIntervalMs: number;
  pollReadTimeoutMs: number;
  pollReadRetryCount: number;
  pollReadTimeoutHitCount: number;
  pollReadAlarm: boolean;
};

export type ModbusPointRow = {
  id: string;
  name: string;
  functionCode: number;
  address: number;
  quantity: number;
  instruction: string;
  valueText: string;
  collectEnabled: boolean;
  lastError: string;
  updatedAt?: number;
};

export type ModbusBatchPointPayload = {
  count: number;
  functionCode: number;
  startAddress: number;
};

export const AUTO_READ_MIN_INTERVAL_MS = 100;
export const AUTO_READ_MAX_INTERVAL_MS = 60_000;
export const POLL_READ_TIMEOUT_MIN_MS = 100;
export const POLL_READ_TIMEOUT_MAX_MS = 60_000;
export const POLL_READ_RETRY_MIN = 1;
export const POLL_READ_RETRY_MAX = 20;

function buildGatewayId(index: number): string {
  return `gateway-${index + 1}`;
}

function buildGatewayDefaultName(index: number): string {
  return `网关${index + 1}`;
}

function buildChannelKey(gatewayId: string, slaveId: number): string {
  return `${gatewayId}-id-${slaveId}`;
}

export function buildDefaultGateway(index = 0): ModbusGateway {
  return {
    id: buildGatewayId(index),
    name: buildGatewayDefaultName(index)
  };
}

export function buildChannelFromGateway(
  gateway: ModbusGateway,
  slaveId: number
): ModbusChannel {
  const stationCode = `网关-ID${slaveId}`;
  return {
    key: buildChannelKey(gateway.id, slaveId),
    label: stationCode,
    gatewayId: gateway.id,
    stationCode,
    host: "127.0.0.1",
    port: 502,
    slaveId,
    pollIntervalMs: 1000,
    connected: false,
    endpoint: "",
    pollRunning: false,
    pollingRowId: "",
    lastError: "",
    autoReadEnabled: false,
    autoReadIntervalMs: 1000,
    pollReadTimeoutMs: 5000,
    pollReadRetryCount: 3,
    pollReadTimeoutHitCount: 0,
    pollReadAlarm: false
  };
}

export function defaultRows(): ModbusPointRow[] {
  return [
    createPointRow(0, {
      name: "线圈状态 0",
      functionCode: 1,
      address: 0,
      quantity: 1,
      instruction: "1",
      valueText: "--"
    }),
    createPointRow(1, {
      name: "保持寄存器 0",
      functionCode: 3,
      address: 0,
      quantity: 1,
      instruction: "1",
      valueText: "--"
    })
  ];
}

export function createPointRow(
  index: number,
  overrides?: Partial<ModbusPointRow>
): ModbusPointRow {
  const randomSeed = Math.random().toString(36).slice(2, 8);
  return {
    id: `point-${Date.now()}-${index}-${randomSeed}`,
    name: `点位 ${index + 1}`,
    functionCode: 3,
    address: 0,
    quantity: 1,
    instruction: "1",
    valueText: "--",
    collectEnabled: false,
    lastError: "",
    ...(overrides ?? {})
  };
}

export function createPointRowsByBatch(
  payload: ModbusBatchPointPayload,
  existingCount: number
): ModbusPointRow[] {
  return Array.from({ length: payload.count }, (_, index) => {
    const address = payload.startAddress + index;
    return createPointRow(existingCount + index, {
      functionCode: payload.functionCode,
      address,
      quantity: 1,
      name: `点位 ${existingCount + index + 1}`
    });
  });
}
