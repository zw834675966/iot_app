import { invoke, isTauri } from "@tauri-apps/api/core";

type Result = {
  success: boolean;
  data: Array<any>;
};

export const getAsyncRoutes = () => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`getAsyncRoutes` only supports Tauri desktop runtime.")
    );
  }
  return invoke<Result>("auth_get_async_routes");
};
