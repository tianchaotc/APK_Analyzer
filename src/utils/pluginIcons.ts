// 插件图标解析器
//
// 插件 manifest 的 ui_tab.icon 字段是 lucide-react 图标名字符串（如 "ShieldCheck"）。
// 由于 lucide-react 默认导出是动态对象，Tree-shaking 后无法通过字符串索引访问，
// 这里维护一个常用图标的白名单映射，平衡灵活性 + bundle 大小。
//
// 未识别的图标名会 fallback 到 DEFAULT_ICON（FlaskConical）。
// 想新增支持的图标：在 ICON_MAP 中加入即可。

import {
  // 通用 / 默认
  FlaskConical, Puzzle, Boxes, Package, Layers, Component, Cpu, Code, FileCode,
  Binary, Braces, Terminal, Workflow, GitBranch, Box,
  // 安全 / 风险
  ShieldCheck, Shield, ShieldAlert, ShieldX, ShieldQuestion, Lock, Unlock,
  Key, KeyRound, Fingerprint, AlertTriangle, AlertOctagon, AlertCircle,
  Bug, Skull, Eye, EyeOff, ScanSearch, Search, SearchCode,
  // 权限 / 隐私
  UserCheck, Users, UserX, UserCog, UserPlus, IdCard, BadgeCheck, BadgeAlert,
  // 网络 / 通信
  Wifi, Globe, Network, Server, Cloud, Download, Upload, Send, Mail, Link,
  Link2, ExternalLink, Radio, Signal, HardDrive, Database,
  // 文件 / 资源
  FileText, FileImage, FileArchive, FileCode2, Folder, FolderOpen, Image,
  Film, Music, Video, File, Files, Archive, Save, FileSearch,
  // 性能 / 分析
  Gauge, Activity, TrendingUp, TrendingDown, BarChart3, PieChart, LineChart,
  ChartBar, ChartLine, ChartPie, Timer, Clock, Zap, ZapOff, Flame, Rocket,
  // 信息 / 状态
  Info, HelpCircle, CheckCircle2, XCircle, CircleAlert, Lightbulb, BookOpen,
  BookMarked, GraduationCap, Microscope, Beaker, TestTube, Microchip,
  // 工具
  Wrench, Hammer, Settings, Settings2, Sliders, Cog,
  // 其他常用
  Star, Flag, Tag, Bookmark, Heart, Award, Trophy, Medal, Crown, Sparkles,
  FileCheck, FileWarning, FileX, ClipboardList, ClipboardCheck, ListChecks,
  type LucideIcon,
} from "lucide-react";

/// 默认插件图标（manifest 未指定或未识别时使用）
export const DEFAULT_PLUGIN_ICON = FlaskConical;

/// 图标名 → 组件映射表
const ICON_MAP: Record<string, LucideIcon> = {
  // 通用
  FlaskConical, Puzzle, Boxes, Package, Layers, Component, Cpu, Code, FileCode,
  Binary, Braces, Terminal, Workflow, GitBranch, Box,
  // 安全
  ShieldCheck, Shield, ShieldAlert, ShieldX, ShieldQuestion, Lock, Unlock,
  Key, KeyRound, Fingerprint, AlertTriangle, AlertOctagon, AlertCircle,
  Bug, Skull, Eye, EyeOff, ScanSearch, Search, SearchCode,
  // 权限
  UserCheck, Users, UserX, UserCog, UserPlus, IdCard, BadgeCheck, BadgeAlert,
  // 网络
  Wifi, Globe, Network, Server, Cloud, Download, Upload, Send, Mail, Link,
  Link2, ExternalLink, Radio, Signal, HardDrive, Database,
  // 文件
  FileText, FileImage, FileArchive, FileCode2, Folder, FolderOpen, Image,
  Film, Music, Video, File, Files, Archive, Save, FileSearch,
  // 性能
  Gauge, Activity, TrendingUp, TrendingDown, BarChart3, PieChart, LineChart,
  ChartBar, ChartLine, ChartPie, Timer, Clock, Zap, ZapOff, Flame, Rocket,
  // 信息
  Info, HelpCircle, CheckCircle2, XCircle, CircleAlert, Lightbulb, BookOpen,
  BookMarked, GraduationCap, Microscope, Beaker, TestTube, Microchip,
  // 工具
  Wrench, Hammer, Settings, Settings2, Sliders, Cog,
  // 其他
  Star, Flag, Tag, Bookmark, Heart, Award, Trophy, Medal, Crown, Sparkles,
  FileCheck, FileWarning, FileX, ClipboardList, ClipboardCheck, ListChecks,
};

/// 根据图标名解析组件。
/// 未识别返回 DEFAULT_PLUGIN_ICON。
///
/// 名称大小写敏感：要求与 lucide-react 导出名完全一致（PascalCase）。
export function resolvePluginIcon(name?: string | null): LucideIcon {
  if (!name) return DEFAULT_PLUGIN_ICON;
  return ICON_MAP[name] ?? DEFAULT_PLUGIN_ICON;
}
