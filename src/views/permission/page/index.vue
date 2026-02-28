<script setup lang="ts">
import {
  adminChangeUserPassword,
  adminDeleteUser,
  adminListUsers,
  adminRegisterUser,
  adminUpdateUser,
  getUserDeviceScope,
  type AdminManagedUserData,
  upsertUserDeviceScope
} from "@/api/user";
import { useUserStoreHook } from "@/store/modules/user";
import { message } from "@/utils/message";
import { ElMessageBox } from "element-plus";
import { computed, onMounted, reactive, ref } from "vue";

defineOptions({
  name: "PermissionPage"
});

const userStore = useUserStoreHook();
const isAdmin = computed(() => userStore.roles.includes("admin"));
const operatorUsername = computed(() => userStore.username);
const loadingUsers = ref(false);
const users = ref<AdminManagedUserData[]>([]);
const managementCollapsePanels = ref<string[]>([]);

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

const editDialogVisible = ref(false);
const editUserId = ref<number>(0);
const editForm = reactive({
  username: "",
  nickname: "",
  phone: "",
  roles: [] as string[],
  isActive: true,
  accountTermType: "days" as "permanent" | "days",
  accountValidDays: 30
});

const passwordDialogVisible = ref(false);
const passwordForm = reactive({
  userId: 0,
  username: "",
  password: ""
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

const formatTime = (value?: number) => {
  if (!value) return "-";
  return new Date(value).toLocaleString();
};

const accountText = (user: AdminManagedUserData) => {
  if (user.accountIsPermanent) return "永久";
  const days = user.accountValidDays ?? "-";
  const expireAt = formatTime(user.accountExpireAt);
  return `${days}天 / 到期: ${expireAt}`;
};

const isProtectedAdminUser = (user: AdminManagedUserData) =>
  user.username.toLowerCase() === "admin";

const getDaysFallback = (expireAt?: number) => {
  if (!expireAt) return 30;
  const ms = expireAt - Date.now();
  if (ms <= 0) return 1;
  return Math.max(1, Math.ceil(ms / (24 * 60 * 60 * 1000)));
};

const validateOperator = () => {
  if (!isAdmin.value) {
    message("仅管理员可执行该操作", { type: "warning" });
    return false;
  }
  if (!operatorUsername.value.trim()) {
    message("当前登录用户无效，请重新登录", { type: "error" });
    return false;
  }
  return true;
};

async function loadUsers() {
  if (!validateOperator()) return;
  loadingUsers.value = true;
  try {
    const result = await adminListUsers({
      operatorUsername: operatorUsername.value
    });
    users.value = [...(result?.data ?? [])];
  } catch (error: any) {
    message(error?.message ?? "加载用户列表失败", { type: "error" });
  } finally {
    loadingUsers.value = false;
  }
}

async function handleRegister() {
  if (!validateOperator()) return;
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
      phone: registerForm.phone,
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
    await loadUsers();
  } catch (error: any) {
    message(error?.message ?? "用户注册失败", { type: "error" });
  }
}

const openEditDialog = (user: AdminManagedUserData) => {
  if (isProtectedAdminUser(user)) {
    message("admin 用户仅允许修改密码", { type: "warning" });
    return;
  }
  editUserId.value = user.userId;
  editForm.username = user.username;
  editForm.nickname = user.nickname;
  editForm.phone = user.phone ?? "";
  editForm.roles = [...(user.roles ?? [])];
  editForm.isActive = user.isActive;
  editForm.accountTermType = user.accountIsPermanent ? "permanent" : "days";
  editForm.accountValidDays =
    user.accountValidDays ?? getDaysFallback(user.accountExpireAt);
  editDialogVisible.value = true;
};

async function handleUpdateUser() {
  if (!validateOperator()) return;
  if (editUserId.value <= 0) {
    message("用户ID无效", { type: "warning" });
    return;
  }
  if (!editForm.username.trim() || !editForm.nickname.trim()) {
    message("账号和名称为必填项", { type: "warning" });
    return;
  }
  if (editForm.roles.length === 0) {
    message("请至少选择一个角色", { type: "warning" });
    return;
  }
  if (
    editForm.accountTermType === "days" &&
    (!Number.isFinite(editForm.accountValidDays) ||
      editForm.accountValidDays <= 0)
  ) {
    message("账号期限天数必须大于 0", { type: "warning" });
    return;
  }

  try {
    await adminUpdateUser({
      operatorUsername: operatorUsername.value,
      userId: editUserId.value,
      username: editForm.username.trim(),
      nickname: editForm.nickname.trim(),
      phone: editForm.phone,
      roles: [...editForm.roles],
      isActive: editForm.isActive,
      accountTermType: editForm.accountTermType,
      accountValidDays:
        editForm.accountTermType === "days"
          ? Number(editForm.accountValidDays)
          : undefined
    });
    message("用户更新成功", { type: "success" });
    editDialogVisible.value = false;
    await loadUsers();
  } catch (error: any) {
    message(error?.message ?? "用户更新失败", { type: "error" });
  }
}

async function handleDeleteUser(user: AdminManagedUserData) {
  if (!validateOperator()) return;
  if (isProtectedAdminUser(user)) {
    message("admin 用户不可删除", { type: "warning" });
    return;
  }
  try {
    await ElMessageBox.confirm(
      `确认删除用户 [${user.username}] 吗？`,
      "删除确认",
      {
        type: "warning",
        confirmButtonText: "删除",
        cancelButtonText: "取消"
      }
    );
    await adminDeleteUser({
      operatorUsername: operatorUsername.value,
      userId: user.userId
    });
    message("删除成功", { type: "success" });
    await loadUsers();
  } catch (error: any) {
    if (error === "cancel") return;
    message(error?.message ?? "删除失败", { type: "error" });
  }
}

const openPasswordDialog = (user: AdminManagedUserData) => {
  passwordForm.userId = user.userId;
  passwordForm.username = user.username;
  passwordForm.password = "";
  passwordDialogVisible.value = true;
};

async function handleChangePassword() {
  if (!validateOperator()) return;
  if (passwordForm.userId <= 0 || !passwordForm.password.trim()) {
    message("请输入新密码", { type: "warning" });
    return;
  }
  try {
    await adminChangeUserPassword({
      operatorUsername: operatorUsername.value,
      userId: passwordForm.userId,
      password: passwordForm.password
    });
    message("密码修改成功", { type: "success" });
    passwordDialogVisible.value = false;
  } catch (error: any) {
    message(error?.message ?? "密码修改失败", { type: "error" });
  }
}

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
    const scope = result.data.scope;
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
  if (!validateOperator()) return;
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

onMounted(() => {
  if (isAdmin.value) {
    loadUsers();
  }
});
</script>

<template>
  <div class="space-y-4">
    <el-alert
      v-if="!isAdmin"
      type="warning"
      show-icon
      :closable="false"
      title="当前账号不是管理员，仅可查看页面。用户注册管理相关操作需要 admin 权限。"
    />

    <el-card v-if="isAdmin" shadow="never">
      <template #header>
        <div class="font-bold">已注册用户信息</div>
      </template>
      <el-table v-loading="loadingUsers" :data="users" border>
        <el-table-column prop="userId" label="ID" width="70" />
        <el-table-column prop="username" label="账号" min-width="130" />
        <el-table-column prop="nickname" label="名称" min-width="120" />
        <el-table-column label="电话" min-width="130">
          <template #default="{ row }">
            {{ row.phone || "-" }}
          </template>
        </el-table-column>
        <el-table-column label="角色" min-width="160">
          <template #default="{ row }">
            <el-tag
              v-for="role in row.roles"
              :key="`${row.userId}-${role}`"
              size="small"
              class="mr-1"
            >
              {{ role }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="90">
          <template #default="{ row }">
            <el-tag :type="row.isActive ? 'success' : 'danger'" size="small">
              {{ row.isActive ? "启用" : "禁用" }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="账号期限" min-width="180">
          <template #default="{ row }">
            {{ accountText(row) }}
          </template>
        </el-table-column>
        <el-table-column label="创建人" min-width="110">
          <template #default="{ row }">
            {{ row.createdBy || "-" }}
          </template>
        </el-table-column>
        <el-table-column label="创建时间" min-width="170">
          <template #default="{ row }">
            {{ formatTime(row.createdAt) }}
          </template>
        </el-table-column>
        <el-table-column label="更新时间" min-width="170">
          <template #default="{ row }">
            {{ formatTime(row.updatedAt) }}
          </template>
        </el-table-column>
        <el-table-column fixed="right" label="操作" width="230">
          <template #default="{ row }">
            <el-button
              link
              type="primary"
              :disabled="isProtectedAdminUser(row)"
              @click="openEditDialog(row)"
            >
              编辑
            </el-button>
            <el-button link type="primary" @click="openPasswordDialog(row)">
              改密
            </el-button>
            <el-button
              link
              type="danger"
              :disabled="isProtectedAdminUser(row)"
              @click="handleDeleteUser(row)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-collapse v-model="managementCollapsePanels">
      <el-collapse-item v-if="isAdmin" name="user-register-management">
        <template #title>
          <div class="font-bold">用户注册管理</div>
        </template>
        <el-form label-width="110px" class="max-w-[760px]">
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

          <el-form-item label="角色" required>
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
            <el-button type="primary" @click="handleRegister"
              >注册用户</el-button
            >
            <el-button @click="loadUsers">刷新列表</el-button>
          </el-form-item>
        </el-form>
      </el-collapse-item>

      <el-collapse-item name="user-device-reserved">
        <template #title>
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
      </el-collapse-item>
    </el-collapse>

    <el-dialog v-model="editDialogVisible" title="编辑用户" width="680px">
      <el-form label-width="110px">
        <el-form-item label="账号" required>
          <el-input v-model="editForm.username" />
        </el-form-item>
        <el-form-item label="名称" required>
          <el-input v-model="editForm.nickname" />
        </el-form-item>
        <el-form-item label="电话">
          <el-input v-model="editForm.phone" />
        </el-form-item>
        <el-form-item label="角色" required>
          <el-select
            v-model="editForm.roles"
            multiple
            collapse-tags
            collapse-tags-tooltip
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
        <el-form-item label="状态">
          <el-switch v-model="editForm.isActive" />
        </el-form-item>
        <el-form-item label="账号期限">
          <el-radio-group v-model="editForm.accountTermType">
            <el-radio label="days">按天</el-radio>
            <el-radio label="permanent">永久</el-radio>
          </el-radio-group>
        </el-form-item>
        <el-form-item
          v-if="editForm.accountTermType === 'days'"
          label="期限天数"
          required
        >
          <el-input-number
            v-model="editForm.accountValidDays"
            :min="1"
            :max="36500"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="editDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleUpdateUser">保存</el-button>
      </template>
    </el-dialog>

    <el-dialog v-model="passwordDialogVisible" title="修改密码" width="520px">
      <el-form label-width="100px">
        <el-form-item label="账号">
          <el-input :model-value="passwordForm.username" disabled />
        </el-form-item>
        <el-form-item label="新密码" required>
          <el-input
            v-model="passwordForm.password"
            type="password"
            show-password
            placeholder="请输入新密码"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="passwordDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleChangePassword">保存</el-button>
      </template>
    </el-dialog>
  </div>
</template>
