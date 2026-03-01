<script setup lang="ts">
import { ref } from "vue";
import FlashlightFill from "~icons/ri/flashlight-fill";
import LeafFill from "~icons/ri/leaf-fill";
import MoneyCnyCircleFill from "~icons/ri/money-cny-circle-fill";
import Dashboard3Fill from "~icons/ri/dashboard-3-fill";
import ArrowUpLine from "~icons/ri/arrow-up-line";
import ArrowDownLine from "~icons/ri/arrow-down-line";
import ServerFill from "~icons/ri/server-fill";

defineOptions({
  name: "Welcome"
});

const cards = [
  {
    title: "当日总耗电量",
    value: "1,245",
    unit: "kWh",
    icon: FlashlightFill,
    color: "text-blue-500",
    bg: "bg-blue-50 dark:bg-blue-500/10",
    trend: "-5.2%",
    isUp: false
  },
  {
    title: "实时总功率",
    value: "45.2",
    unit: "kW",
    icon: Dashboard3Fill,
    color: "text-purple-500",
    bg: "bg-purple-50 dark:bg-purple-500/10",
    trend: "+1.2%",
    isUp: true
  },
  {
    title: "碳排放量",
    value: "2.4",
    unit: "Tons",
    icon: LeafFill,
    color: "text-green-500",
    bg: "bg-green-50 dark:bg-green-500/10",
    trend: "-12.0%",
    isUp: false
  },
  {
    title: "预计能源成本",
    value: "3,450",
    unit: "¥",
    icon: MoneyCnyCircleFill,
    color: "text-orange-500",
    bg: "bg-orange-50 dark:bg-orange-500/10",
    trend: "+2.4%",
    isUp: true
  }
];

const devices = [
  {
    name: "1号厂房空调机组",
    status: "运行中",
    power: "12.4 kW",
    type: "success"
  },
  {
    name: "2号车间照明系统",
    status: "运行中",
    power: "5.2 kW",
    type: "success"
  },
  { name: "冷却水塔循环泵", status: "异常", power: "0.0 kW", type: "danger" },
  {
    name: "数据中心服务器机架",
    status: "运行中",
    power: "24.5 kW",
    type: "success"
  },
  { name: "办公楼电梯A", status: "待机", power: "1.2 kW", type: "info" }
];

const powerData = [
  { time: "00:00", value: 30 },
  { time: "04:00", value: 25 },
  { time: "08:00", value: 45 },
  { time: "12:00", value: 60 },
  { time: "16:00", value: 55 },
  { time: "20:00", value: 40 },
  { time: "24:00", value: 35 }
];
</script>

<template>
  <div class="apple-dashboard min-h-full font-sans p-2 sm:p-4">
    <div class="max-w-7xl mx-auto space-y-6">
      <!-- Header -->
      <header
        class="flex flex-col sm:flex-row justify-between items-start sm:items-end mb-8 apple-glass p-6 rounded-[24px]"
      >
        <div>
          <h1
            class="text-3xl font-bold tracking-tight text-slate-900 dark:text-white mb-2"
          >
            能源概览
          </h1>
          <p class="text-sm text-slate-500 dark:text-slate-400">
            星期五, 2026年2月27日
          </p>
        </div>
        <div class="mt-4 sm:mt-0 flex items-center space-x-2">
          <div
            class="inline-flex items-center px-4 py-2 rounded-full text-sm font-medium bg-green-50 text-green-700 dark:bg-green-500/10 dark:text-green-400 border border-green-200/50 dark:border-green-500/20"
          >
            <span
              class="w-2.5 h-2.5 rounded-full bg-green-500 mr-2.5 relative flex justify-center items-center"
            >
              <span
                class="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"
              />
            </span>
            系统运行正常
          </div>
        </div>
      </header>

      <!-- KPI Cards -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div
          v-for="(card, index) in cards"
          :key="index"
          class="apple-card group"
        >
          <div class="flex justify-between items-start mb-4">
            <div
              :class="[
                'w-12 h-12 rounded-[16px] flex items-center justify-center transition-transform group-hover:scale-110 duration-300',
                card.bg,
                card.color
              ]"
            >
              <component :is="card.icon" class="w-6 h-6" />
            </div>
            <div
              :class="[
                'flex items-center text-xs font-semibold px-2.5 py-1 rounded-full',
                card.isUp
                  ? 'bg-red-50 text-red-600 dark:bg-red-500/10 dark:text-red-400'
                  : 'bg-green-50 text-green-600 dark:bg-green-500/10 dark:text-green-400'
              ]"
            >
              <component
                :is="card.isUp ? ArrowUpLine : ArrowDownLine"
                class="w-3 h-3 mr-1"
              />
              {{ card.trend }}
            </div>
          </div>
          <div
            class="text-slate-500 dark:text-slate-400 text-sm font-medium mb-1"
          >
            {{ card.title }}
          </div>
          <div class="flex items-baseline space-x-1">
            <span
              class="text-3xl font-bold tracking-tight text-slate-900 dark:text-white"
              >{{ card.value }}</span
            >
            <span class="text-sm font-medium text-slate-400">{{
              card.unit
            }}</span>
          </div>
        </div>
      </div>

      <!-- Main Content Area -->
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- Power Curve (Left 2 cols) -->
        <div class="lg:col-span-2 apple-card flex flex-col">
          <div class="flex justify-between items-center mb-6">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white">
              24小时用电负荷趋势
            </h2>
            <el-link
              type="primary"
              :underline="false"
              class="!text-sm !font-medium"
            >
              查看详情
            </el-link>
          </div>
          <div
            class="flex-1 flex items-end justify-between relative pt-6 pb-2 min-h-[250px]"
          >
            <!-- Simulated Chart using Tailwind -->
            <div
              class="absolute inset-0 flex flex-col justify-between pt-6 pb-8 z-0"
            >
              <div
                class="border-b border-slate-100 dark:border-slate-800 w-full h-0"
              />
              <div
                class="border-b border-slate-100 dark:border-slate-800 w-full h-0"
              />
              <div
                class="border-b border-slate-100 dark:border-slate-800 w-full h-0"
              />
              <div
                class="border-b border-slate-100 dark:border-slate-800 w-full h-0"
              />
            </div>
            <div
              v-for="(point, i) in powerData"
              :key="i"
              class="flex-1 flex flex-col items-center z-10 group"
            >
              <div class="w-full flex justify-center h-48 items-end pb-2">
                <div
                  class="w-8 sm:w-12 bg-gradient-to-t from-blue-500 to-cyan-400 dark:from-blue-600 dark:to-cyan-500 rounded-t-md opacity-80 group-hover:opacity-100 transition-all duration-300 relative cursor-pointer"
                  :style="{ height: `${point.value}%` }"
                >
                  <!-- Tooltip -->
                  <div
                    class="absolute -top-10 left-1/2 -translate-x-1/2 bg-slate-800 text-white text-xs py-1 px-2 rounded opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none whitespace-nowrap shadow-lg"
                  >
                    {{ point.value }} kW
                  </div>
                </div>
              </div>
              <span class="text-xs text-slate-400 mt-2 font-medium">{{
                point.time
              }}</span>
            </div>
          </div>
        </div>

        <!-- Device Status (Right 1 col) -->
        <div class="apple-card flex flex-col">
          <div class="flex justify-between items-center mb-6">
            <h2 class="text-lg font-bold text-slate-900 dark:text-white">
              重点设备状态
            </h2>
            <div
              class="w-8 h-8 rounded-full bg-slate-50 dark:bg-slate-800 flex items-center justify-center"
            >
              <ServerFill class="w-4 h-4 text-slate-500" />
            </div>
          </div>
          <div class="flex-1 space-y-3 pr-1">
            <div
              v-for="(device, i) in devices"
              :key="i"
              class="flex items-center justify-between p-3 rounded-[16px] hover:bg-slate-50 dark:hover:bg-white/5 transition-colors cursor-pointer group"
            >
              <div class="flex items-center space-x-3">
                <div
                  :class="[
                    'w-2 h-2 rounded-full',
                    device.type === 'success'
                      ? 'bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]'
                      : device.type === 'danger'
                        ? 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.6)] animate-pulse'
                        : 'bg-slate-400 shadow-[0_0_8px_rgba(148,163,184,0.6)]'
                  ]"
                />
                <div>
                  <div
                    class="text-sm font-semibold text-slate-700 dark:text-slate-200 group-hover:text-blue-500 transition-colors"
                  >
                    {{ device.name }}
                  </div>
                  <div class="text-xs text-slate-400 mt-0.5">
                    {{ device.status }}
                  </div>
                </div>
              </div>
              <div class="text-sm font-bold text-slate-900 dark:text-white">
                {{ device.power }}
              </div>
            </div>
          </div>
          <el-button class="w-full mt-4" size="large" plain>
            查看全部设备
          </el-button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
@reference "../../style/tailwind.css";

/* SF Pro Font Fallback */
.font-sans {
  font-family:
    -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", Arial,
    sans-serif;
}

.apple-glass {
  @apply bg-white/70 backdrop-blur-3xl border border-white/40 shadow-[0_8px_30px_rgba(0,0,0,0.04)] dark:bg-[#1c1c1e]/70 dark:border-white/10 dark:shadow-[0_8px_30px_rgba(0,0,0,0.2)];
}

.apple-card {
  @apply bg-white/70 backdrop-blur-3xl border border-white/40 p-6 rounded-[24px] shadow-[0_8px_30px_rgba(0,0,0,0.04)] transition-all duration-500 hover:shadow-[0_20px_40px_rgba(0,0,0,0.08)] dark:bg-[#1c1c1e]/70 dark:border-white/10 dark:shadow-[0_8px_30px_rgba(0,0,0,0.2)] dark:hover:shadow-[0_20px_40px_rgba(0,0,0,0.4)];
}
</style>
