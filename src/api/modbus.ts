import { invokeWithTrace } from "./tauriInvoke";

export type ApiResult<T> = {
  success: boolean;
  data: T;
};

export type ModbusConnectPayload = {
  channelKey: string;
  host: string;
  port: number;
  slaveId: number;
};

export type ModbusConnectData = {
  channelKey: string;
  endpoint: string;
  slaveId: number;
  pollIntervalMs: number;
};

export type ModbusDisconnectPayload = {
  channelKey: string;
};

export type ModbusDisconnectData = {
  channelKey: string;
  disconnected: boolean;
};

export type ModbusPollIntervalPayload = {
  channelKey: string;
  intervalMs: number;
};

export type ModbusPollIntervalData = {
  channelKey: string;
  intervalMs: number;
};

export type ModbusPointPayload = {
  channelKey: string;
  slaveId: number;
  functionCode: number;
  address: number;
  quantity: number;
};

export type ModbusWritePointPayload = {
  channelKey: string;
  slaveId: number;
  functionCode: number;
  address: number;
  instruction: string;
};

export type ModbusReadPointData = {
  channelKey: string;
  slaveId: number;
  functionCode: number;
  address: number;
  quantity: number;
  values: string[];
};

export type ModbusWritePointData = {
  channelKey: string;
  slaveId: number;
  functionCode: number;
  address: number;
  quantity: number;
  accepted: boolean;
};

export type ModbusPollStartPayload = {
  channelKey: string;
  slaveId: number;
  functionCode: number;
  address: number;
  quantity: number;
  intervalMs?: number;
};

export type ModbusPollStartData = {
  channelKey: string;
  running: boolean;
  intervalMs: number;
};

export type ModbusPollStopPayload = {
  channelKey: string;
};

export type ModbusPollStopData = {
  channelKey: string;
  running: boolean;
};

export type ModbusPollSnapshotPayload = {
  channelKey: string;
};

export type ModbusPollStateData = {
  channelKey: string;
  running: boolean;
  intervalMs: number;
  latest?: ModbusReadPointData;
  lastError?: string;
  updatedAt?: number;
};

export type ModbusGatewayConfig = {
  id: string;
  name: string;
};

export type ModbusChannelConfig = {
  key: string;
  label: string;
  gatewayId: string;
  stationCode: string;
  host: string;
  port: number;
  slaveId: number;
  pollIntervalMs: number;
  autoReadEnabled: boolean;
  autoReadIntervalMs: number;
  pollReadTimeoutMs: number;
  pollReadRetryCount: number;
};

export type ModbusPointConfig = {
  id: string;
  name: string;
  functionCode: number;
  address: number;
  quantity: number;
  instruction: string;
  collectEnabled: boolean;
};

export type ModbusConfigLoadPayload = {
  activeChannelKey?: string;
};

export type ModbusConfigLoadData = {
  gateways: ModbusGatewayConfig[];
  channels: ModbusChannelConfig[];
  pointsByChannel: Record<string, ModbusPointConfig[]>;
  activeChannelKey: string;
};

export type ModbusConfigSavePayload = {
  mode: "online" | "offline";
  activeChannelKey: string;
  gateways: ModbusGatewayConfig[];
  channels: ModbusChannelConfig[];
  pointsByChannel: Record<string, ModbusPointConfig[]>;
  autoRenameDuplicateGateway: boolean;
};

export type ModbusConfigSaveData = {
  mode: string;
  savedGatewayNames: string[];
};

export type ModbusCollectPointsPayload = {
  channelKey: string;
  pointIds: string[];
  applyAll: boolean;
};

export type ModbusCollectPointsData = {
  channelKey: string;
  updatedCount: number;
};

export type ModbusTelemetrySamplePayload = {
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
};

export type ModbusTelemetryIngestPayload = {
  samples: ModbusTelemetrySamplePayload[];
};

export type ModbusTelemetryIngestData = {
  inserted: number;
};

export const modbusConnect = (payload: ModbusConnectPayload) => {
  return invokeWithTrace<ApiResult<ModbusConnectData>>(
    "modbusConnect",
    "modbus_connect",
    {
      payload
    }
  );
};

export const modbusDisconnect = (payload: ModbusDisconnectPayload) => {
  return invokeWithTrace<ApiResult<ModbusDisconnectData>>(
    "modbusDisconnect",
    "modbus_disconnect",
    {
      payload
    }
  );
};

export const modbusSetPollInterval = (payload: ModbusPollIntervalPayload) => {
  return invokeWithTrace<ApiResult<ModbusPollIntervalData>>(
    "modbusSetPollInterval",
    "modbus_set_poll_interval",
    {
      payload
    }
  );
};

export const modbusReadPoint = (payload: ModbusPointPayload) => {
  return invokeWithTrace<ApiResult<ModbusReadPointData>>(
    "modbusReadPoint",
    "modbus_read_point",
    {
      payload
    }
  );
};

export const modbusWritePoint = (payload: ModbusWritePointPayload) => {
  return invokeWithTrace<ApiResult<ModbusWritePointData>>(
    "modbusWritePoint",
    "modbus_write_point",
    {
      payload
    }
  );
};

export const modbusPollStart = (payload: ModbusPollStartPayload) => {
  return invokeWithTrace<ApiResult<ModbusPollStartData>>(
    "modbusPollStart",
    "modbus_poll_start",
    {
      payload
    }
  );
};

export const modbusPollStop = (payload: ModbusPollStopPayload) => {
  return invokeWithTrace<ApiResult<ModbusPollStopData>>(
    "modbusPollStop",
    "modbus_poll_stop",
    {
      payload
    }
  );
};

export const modbusPollSnapshot = (payload: ModbusPollSnapshotPayload) => {
  return invokeWithTrace<ApiResult<ModbusPollStateData>>(
    "modbusPollSnapshot",
    "modbus_poll_snapshot",
    {
      payload
    }
  );
};

export const modbusConfigLoad = (payload: ModbusConfigLoadPayload = {}) => {
  return invokeWithTrace<ApiResult<ModbusConfigLoadData>>(
    "modbusConfigLoad",
    "modbus_config_load",
    {
      payload
    }
  );
};

export const modbusConfigSave = (payload: ModbusConfigSavePayload) => {
  return invokeWithTrace<ApiResult<ModbusConfigSaveData>>(
    "modbusConfigSave",
    "modbus_config_save",
    {
      payload
    }
  );
};

export const modbusCollectPointsSet = (payload: ModbusCollectPointsPayload) => {
  return invokeWithTrace<ApiResult<ModbusCollectPointsData>>(
    "modbusCollectPointsSet",
    "modbus_collect_points_set",
    {
      payload
    }
  );
};

export const modbusTelemetryIngest = (
  payload: ModbusTelemetryIngestPayload
) => {
  return invokeWithTrace<ApiResult<ModbusTelemetryIngestData>>(
    "modbusTelemetryIngest",
    "modbus_telemetry_ingest",
    {
      payload
    }
  );
};
