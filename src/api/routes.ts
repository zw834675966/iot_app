import { invokeWithTrace } from "./tauriInvoke";

type Result = {
  success: boolean;
  data: Array<any>;
};

export const getAsyncRoutes = () => {
  return invokeWithTrace<Result>("getAsyncRoutes", "auth_get_async_routes");
};
