const Layout = () => import("@/layout/index.vue");

export default {
  path: "/modbus",
  name: "Modbus",
  component: Layout,
  redirect: "/modbus/index",
  meta: {
    icon: "ri/database-2-line",
    title: "Modbus",
    rank: 2
  },
  children: [
    {
      path: "/modbus/index",
      name: "ModbusTcpWorkbench",
      component: () => import("@/views/modbus/index.vue"),
      meta: {
        title: "Modbus TCP",
        icon: "ri/terminal-window-line"
      }
    }
  ]
} satisfies RouteConfigsTable;
