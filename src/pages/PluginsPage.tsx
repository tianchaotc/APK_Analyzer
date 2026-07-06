import { PageHeader, Section } from "../components/common/DataTable";
import { PluginManagerPanel } from "../components/plugin/PluginManagerPanel";

/// 插件管理路由页面。
/// 由侧边栏的 "Plugins" 入口进入，展示已发现插件的列表、状态与启用开关。
export function PluginsPage() {
  return (
    <div>
      <PageHeader
        title="Plugins"
        subtitle="Discover, enable, and inspect native analyzer plugins"
      />
      <Section title="Installed Plugins">
        <PluginManagerPanel />
      </Section>
    </div>
  );
}
