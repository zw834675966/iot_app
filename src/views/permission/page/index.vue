<script setup lang="ts">
import {
  adminRegisterUser,
  adminRenewUserAccount,
  getUserDeviceScope,
  upsertUserDeviceScope
} from "@/api/user";
import { message } from "@/utils/message";
import { computed, reactive } from "vue";
import { useUserStoreHook } from "@/store/modules/user";

defineOptions({
  name: "PermissionPage"
});

const userStore = useUserStoreHook();
const isAdmin = computed(() => userStore.roles.includes("admin"));
const operatorUsername = computed(() => userStore.username);

const roleOptions = [
  { value: "operator", label: "操作员" },
  { value: "tenant", label: "租户" },
  { value: "maintainer", label: "维保" }
];

const registerForm = reactive({
  username: "",
  password: "",
  nickname: "",
  phone: "",
  roles: [] as string[],
  accountTermType: "days" as "permanent" | "days",
  accountValidDays: 30
});

const renewForm = reactive({
  userId: "",
  renewMode: "days" as "permanent" | "days",
  renewDays: 30
});

const areaOptions = ["华东", "华南", "华北"];
const floorOptions = ["1F", "2F", "3F"];
const deviceOptions = ["A-100", "B-200", "C-300"];

const deviceForm = reactive({
  userId: "",
  allAreas: false,
  allFloors: false,
  allDevices: false,
  areas: [] as string[],
  floors: [] as string[],
  devices: [] as string[]
});

function applyAllSelect(
  type: "areas" | "floors" | "devices",
  checked: boolean
) {
  if (type === "areas") {
    deviceForm.areas = checked ? [...areaOptions] : [];
    return;
  }
  if (type === "floors") {
    deviceForm.floors = checked ? [...floorOptions] : [];
    return;
  }
  deviceForm.devices = checked ? [...deviceOptions] : [];
}

async function handleRegister() {
  if (!isAdmin.value) {
    message("仅管理员可执行注册", { type: "warning" });
    return;
  }
  if (!operatorUsername.value) {
    message("当前登录用户无效，请重新登录", { type: "error" });
    return;
  }
  if (!registerForm.username.trim() || !registerForm.password.trim()) {
    message("账号和密码为必填项", { type: "warning" });
    return;
  }
  if (!registerForm.nickname.trim()) {
    message("名称为必填项", { type: "warning" });
    return;
  }
  if (registerForm.roles.length === 0) {
    message("请至少选择一个角色", { type: "warning" });
    return;
  }
  if (
    registerForm.accountTermType === "days" &&
    (!Number.isFinite(registerForm.accountValidDays) ||
      registerForm.accountValidDays <= 0)
  ) {
    message("账号期限天数必须大于 0", { type: "warning" });
    return;
  }

  try {
    await adminRegisterUser({
      operatorUsername: operatorUsername.value,
      username: registerForm.username.trim(),
      password: registerForm.password,
      nickname: registerForm.nickname.trim(),
      phone: registerForm.phone.trim() || undefined,
      roles: [...registerForm.roles],
      accountTermType: registerForm.accountTermType,
      accountValidDays:
        registerForm.accountTermType === "days"
          ? Number(registerForm.accountValidDays)
          : undefined
    });
    message("用户注册成功", { type: "success" });
    registerForm.username = "";
    registerForm.password = "";
    registerForm.nickname = "";
    registerForm.phone = "";
    registerForm.roles = [];
    registerForm.accountTermType = "days";
    registerForm.accountValidDays = 30;
  } catch (error: any) {
    message(error?.message ?? "用户注册失败", { type: "error" });
  }
}

async function handleRenew() {
  if (!isAdmin.value) {
    message("仅管理员可执行续期", { type: "warning" });
    return;
  }
  if (!operatorUsername.value) {
    message("当前登录用户无效，请重新登录", { type: "error" });
    return;
  }
  const userId = Number(renewForm.userId);
  if (!Number.isInteger(userId) || userId <= 0) {
    message("请输入有效的用户 ID", { type: "warning" });
    return;
  }
  if (
    renewForm.renewMode === "days" &&
    (!Number.isFinite(renewForm.renewDays) || renewForm.renewDays <= 0)
  ) {
    message("续期天数必须大于 0", { type: "warning" });
    return;
  }

  try {
    await adminRenewUserAccount({
      operatorUsername: operatorUsername.value,
      userId,
      renewMode: renewForm.renewMode,
      renewDays:
        renewForm.renewMode === "days" ? Number(renewForm.renewDays) : undefined
    });
    message("账号续期成功", { type: "success" });
  } catch (error: any) {
    message(error?.message ?? "账号续期失败", { type: "error" });
  }
}

async function handleLoadDeviceScope() {
  if (
    !Number.isInteger(Number(deviceForm.userId)) ||
    Number(deviceForm.userId) <= 0
  ) {
    message("请输入有效的用户 ID", { type: "warning" });
    return;
  }
  try {
    const result = await getUserDeviceScope(Number(deviceForm.userId));
    if (!result?.data) return;
    const { scope } = result.data;
    deviceForm.allAreas = scope.allAreas;
    deviceForm.allFloors = scope.allFloors;
    deviceForm.allDevices = scope.allDevices;
    deviceForm.areas = [...scope.areas];
    deviceForm.floors = [...scope.floors];
    deviceForm.devices = [...scope.devices];
    message(result.data.message || "预留接口返回完成", { type: "info" });
  } catch (error: any) {
    message(error?.message ?? "加载设备配置失败", { type: "error" });
  }
}

async function handleSaveDeviceScope() {
  if (
    !Number.isInteger(Number(deviceForm.userId)) ||
    Number(deviceForm.userId) <= 0
  ) {
    message("请输入有效的用户 ID", { type: "warning" });
    return;
  }
  try {
    await upsertUserDeviceScope({
      userId: Number(deviceForm.userId),
      allAreas: deviceForm.allAreas,
      allFloors: deviceForm.allFloors,
      allDevices: deviceForm.allDevices,
      areas: [...deviceForm.areas],
      floors: [...deviceForm.floors],
      devices: [...deviceForm.devices]
    });
    message("设备配置已保存", { type: "success" });
  } catch (error: any) {
    if ((error?.message ?? "").includes("RESERVED_API_NOT_IMPLEMENTED")) {
      message("设备配置接口为预留态，暂未开放保存", { type: "warning" });
      return;
    }
    message(error?.message ?? "保存设备配置失败", { type: "error" });
  }
}
</script>

<template>
  <div class="space-y-4">
    <el-alert
      v-if="!isAdmin"
      type="warning"
      show-icon
      :closable="false"
      title="当前账号不是管理员，仅可查看页面。注册与续期功能需管理员权限。"
    />

    <el-card shadow="never">
      <template #header>
        <div class="font-bold">管理员用户注册</div>
      </template>
      <el-form label-width="110px" class="max-w-[720px]">
        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item label="账号" required>
              <el-input
                v-model="registerForm.username"
                placeholder="请输入账号"
              />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="密码" required>
              <el-input
                v-model="registerForm.password"
                type="password"
                show-password
                placeholder="请输入密码"
              />
            </el-form-item>
          </el-col>
        </el-row>

        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item label="名称" required>
              <el-input
                v-model="registerForm.nickname"
                placeholder="请输入名称"
              />
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="电话">
              <el-input v-model="registerForm.phone" placeholder="可选" />
            </el-form-item>
          </el-col>
        </el-row>

        <el-form-item label="权限角色" required>
          <el-select
            v-model="registerForm.roles"
            multiple
            collapse-tags
            collapse-tags-tooltip
            placeholder="请选择角色（可多选）"
            class="w-full"
          >
            <el-option
              v-for="item in roleOptions"
              :key="item.value"
              :label="item.label"
              :value="item.value"
            />
          </el-select>
        </el-form-item>

        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item label="账号期限">
              <el-radio-group v-model="registerForm.accountTermType">
                <el-radio label="days">按天</el-radio>
                <el-radio label="permanent">永久</el-radio>
              </el-radio-group>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item
              v-if="registerForm.accountTermType === 'days'"
              label="期限天数"
              required
            >
              <el-input-number
                v-model="registerForm.accountValidDays"
                :min="1"
                :max="36500"
                controls-position="right"
              />
            </el-form-item>
          </el-col>
        </el-row>

        <el-form-item>
          <el-button
            type="primary"
            :disabled="!isAdmin"
            @click="handleRegister"
          >
            注册用户
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card shadow="never">
      <template #header>
        <div class="font-bold">账号续期</div>
      </template>
      <el-form inline>
        <el-form-item label="用户 ID" required>
          <el-input v-model="renewForm.userId" placeholder="请输入用户ID" />
        </el-form-item>
        <el-form-item label="续期模式">
          <el-radio-group v-model="renewForm.renewMode">
            <el-radio label="days">按天</el-radio>
            <el-radio label="permanent">永久</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item v-if="renewForm.renewMode === 'days'" label="续期天数">
          <el-input-number
            v-model="renewForm.renewDays"
            :min="1"
            :max="36500"
          />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" :disabled="!isAdmin" @click="handleRenew">
            提交续期
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card shadow="never">
      <template #header>
        <div class="font-bold">用户设备配置（预留）</div>
      </template>
      <el-form label-width="110px" class="max-w-[860px]">
        <el-form-item label="用户 ID" required>
          <el-input
            v-model="deviceForm.userId"
            placeholder="请输入用户ID"
            class="w-[220px]"
          />
          <el-button class="ml-3" @click="handleLoadDeviceScope"
            >加载预留配置</el-button
          >
        </el-form-item>

        <el-form-item label="区域">
          <el-checkbox
            v-model="deviceForm.allAreas"
            @change="value => applyAllSelect('areas', Boolean(value))"
          >
            全选
          </el-checkbox>
          <el-select
            v-model="deviceForm.areas"
            multiple
            collapse-tags
            class="ml-3 w-[460px]"
            placeholder="请选择区域"
          >
            <el-option
              v-for="area in areaOptions"
              :key="area"
              :label="area"
              :value="area"
            />
          </el-select>
        </el-form-item>

        <el-form-item label="楼层">
          <el-checkbox
            v-model="deviceForm.allFloors"
            @change="value => applyAllSelect('floors', Boolean(value))"
          >
            全选
          </el-checkbox>
          <el-select
            v-model="deviceForm.floors"
            multiple
            collapse-tags
            class="ml-3 w-[460px]"
            placeholder="请选择楼层"
          >
            <el-option
              v-for="floor in floorOptions"
              :key="floor"
              :label="floor"
              :value="floor"
            />
          </el-select>
        </el-form-item>

        <el-form-item label="设备">
          <el-checkbox
            v-model="deviceForm.allDevices"
            @change="value => applyAllSelect('devices', Boolean(value))"
          >
            全选
          </el-checkbox>
          <el-select
            v-model="deviceForm.devices"
            multiple
            collapse-tags
            class="ml-3 w-[460px]"
            placeholder="请选择设备"
          >
            <el-option
              v-for="device in deviceOptions"
              :key="device"
              :label="device"
              :value="device"
            />
          </el-select>
        </el-form-item>

        <el-form-item>
          <el-button
            type="primary"
            :disabled="!isAdmin"
            @click="handleSaveDeviceScope"
          >
            保存配置（预留）
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>
