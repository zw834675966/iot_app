import { invoke, isTauri } from "@tauri-apps/api/core";

export type UserResult = {
  success: boolean;
  data: {
    /** 头像 */
    avatar: string;
    /** 用户名 */
    username: string;
    /** 昵称 */
    nickname: string;
    /** 当前登录用户的角色 */
    roles: Array<string>;
    /** 按钮级别权限 */
    permissions: Array<string>;
    /** `token` */
    accessToken: string;
    /** 用于调用刷新`accessToken`的接口时所需的`token` */
    refreshToken: string;
    /** `accessToken`的过期时间（格式'xxxx/xx/xx xx:xx:xx'） */
    expires: Date;
  };
};

export type RefreshTokenResult = {
  success: boolean;
  data: {
    /** `token` */
    accessToken: string;
    /** 用于调用刷新`accessToken`的接口时所需的`token` */
    refreshToken: string;
    /** `accessToken`的过期时间（格式'xxxx/xx/xx xx:xx:xx'） */
    expires: Date;
  };
};

export type AdminRegisterUserPayload = {
  operatorUsername: string;
  username: string;
  password: string;
  nickname: string;
  phone?: string;
  roles: string[];
  accountTermType: "permanent" | "days";
  accountValidDays?: number;
};

export type AdminRegisterUserResult = {
  success: boolean;
  data: {
    userId: number;
    username: string;
    roles: string[];
    isActive: boolean;
    accountIsPermanent: boolean;
    accountExpireAt?: number;
  };
};

export type AdminRenewUserPayload = {
  operatorUsername: string;
  userId: number;
  renewMode: "permanent" | "days";
  renewDays?: number;
};

export type AdminRenewUserResult = {
  success: boolean;
  data: {
    userId: number;
    accountIsPermanent: boolean;
    accountExpireAt?: number;
    isActive: boolean;
  };
};

export type AdminManagedUserData = {
  userId: number;
  username: string;
  nickname: string;
  phone?: string;
  roles: string[];
  isActive: boolean;
  accountIsPermanent: boolean;
  accountValidDays?: number;
  accountExpireAt?: number;
  createdAt?: number;
  updatedAt?: number;
  createdBy?: string;
};

export type AdminListUsersPayload = {
  operatorUsername: string;
};

export type AdminListUsersResult = {
  success: boolean;
  data: AdminManagedUserData[];
};

export type AdminUpdateUserPayload = {
  operatorUsername: string;
  userId: number;
  username: string;
  nickname: string;
  phone?: string;
  roles: string[];
  isActive: boolean;
  accountTermType: "permanent" | "days";
  accountValidDays?: number;
};

export type AdminUpdateUserResult = {
  success: boolean;
  data: AdminManagedUserData;
};

export type AdminDeleteUserPayload = {
  operatorUsername: string;
  userId: number;
};

export type AdminDeleteUserResult = {
  success: boolean;
  data: boolean;
};

export type AdminChangeUserPasswordPayload = {
  operatorUsername: string;
  userId: number;
  password: string;
};

export type AdminChangeUserPasswordResult = {
  success: boolean;
  data: {
    userId: number;
    username: string;
  };
};

export type UserDeviceScopeGetResult = {
  success: boolean;
  data: {
    implemented: boolean;
    message: string;
    scope: {
      allAreas: boolean;
      allFloors: boolean;
      allDevices: boolean;
      areas: string[];
      floors: string[];
      devices: string[];
    };
  };
};

/** 登录 */
export const getLogin = (data?: object) => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`getLogin` only supports Tauri desktop runtime.")
    );
  }
  return invoke<UserResult>("auth_login", { payload: data ?? {} });
};

/** 刷新`token` */
export const refreshTokenApi = (data?: object) => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`refreshTokenApi` only supports Tauri desktop runtime.")
    );
  }
  return invoke<RefreshTokenResult>("auth_refresh_token", {
    payload: data ?? {}
  });
};

export const adminRegisterUser = (payload: AdminRegisterUserPayload) => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`adminRegisterUser` only supports Tauri desktop runtime.")
    );
  }
  return invoke<AdminRegisterUserResult>("auth_admin_register_user", {
    payload
  });
};

export const adminRenewUserAccount = (payload: AdminRenewUserPayload) => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`adminRenewUserAccount` only supports Tauri desktop runtime.")
    );
  }
  return invoke<AdminRenewUserResult>("auth_admin_renew_user_account", {
    payload
  });
};

export const adminListUsers = (payload: AdminListUsersPayload) => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`adminListUsers` only supports Tauri desktop runtime.")
    );
  }
  return invoke<AdminListUsersResult>("auth_admin_list_users", { payload });
};

export const adminUpdateUser = (payload: AdminUpdateUserPayload) => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`adminUpdateUser` only supports Tauri desktop runtime.")
    );
  }
  return invoke<AdminUpdateUserResult>("auth_admin_update_user", { payload });
};

export const adminDeleteUser = (payload: AdminDeleteUserPayload) => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`adminDeleteUser` only supports Tauri desktop runtime.")
    );
  }
  return invoke<AdminDeleteUserResult>("auth_admin_delete_user", { payload });
};

export const adminChangeUserPassword = (
  payload: AdminChangeUserPasswordPayload
) => {
  if (!isTauri()) {
    return Promise.reject(
      new Error(
        "`adminChangeUserPassword` only supports Tauri desktop runtime."
      )
    );
  }
  return invoke<AdminChangeUserPasswordResult>(
    "auth_admin_change_user_password",
    {
      payload
    }
  );
};

export const getUserDeviceScope = (userId: number) => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`getUserDeviceScope` only supports Tauri desktop runtime.")
    );
  }
  return invoke<UserDeviceScopeGetResult>("user_device_scope_get", {
    payload: { userId }
  });
};

export const upsertUserDeviceScope = (payload: {
  userId: number;
  allAreas: boolean;
  allFloors: boolean;
  allDevices: boolean;
  areas: string[];
  floors: string[];
  devices: string[];
}) => {
  if (!isTauri()) {
    return Promise.reject(
      new Error("`upsertUserDeviceScope` only supports Tauri desktop runtime.")
    );
  }
  return invoke("user_device_scope_upsert", { payload });
};
