import { invoke, isTauri } from "@tauri-apps/api/core";

type InvokeArgs = Record<string, unknown>;

type TraceContext = {
  requestId: string;
};

const TRACE_SESSION_PREFIX = `${Date.now().toString(36)}-${Math.random()
  .toString(36)
  .slice(2, 8)}`;
let traceCounter = 0;

function nextRequestId(command: string): string {
  traceCounter += 1;
  return `fe-${TRACE_SESSION_PREFIX}-${command}-${traceCounter}`;
}

export function invokeWithTrace<T>(
  apiName: string,
  command: string,
  args?: InvokeArgs
): Promise<T> {
  if (!isTauri()) {
    return Promise.reject(
      new Error(`\`${apiName}\` only supports Tauri desktop runtime.`)
    );
  }

  const trace: TraceContext = {
    requestId: nextRequestId(command)
  };

  return invoke<T>(command, {
    ...(args ?? {}),
    trace
  });
}
