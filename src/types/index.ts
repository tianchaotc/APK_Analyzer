export interface ApkAnalysis {
  file_path: string;
  file_name: string;
  file_size: number;
  analyzed_at: string;
  overview: OverviewInfo;
  manifest: ManifestInfo;
  permissions: PermissionAnalysis;
  components: ComponentAnalysis;
  resources: ResourceAnalysis;
  native_libs: NativeLibAnalysis;
  dex: DexAnalysis;
  certificate: CertificateAnalysis;
  security: SecurityAnalysis;
  ai_summary: AISummary | null;
  plugins: PluginResult[];
}

/// 单个插件的分析结果
export interface PluginResult {
  plugin_id: string;
  plugin_name: string;
  /// 插件返回的任意 JSON 数据
  data: any;
  /// UI schema JSON（声明式视图描述）
  ui_schema: any;
  /// 分析错误（如有）
  error: string | null;
  /// 分析耗时（毫秒）
  duration_ms: number;
  /// 侧边栏 tab 显示名（来自 manifest.ui_tab.label），None 时 fallback 到 plugin_name
  ui_tab_label?: string | null;
  /// 侧边栏图标名（lucide-react 图标，如 "ShieldCheck"），None 时用默认图标
  ui_tab_icon?: string | null;
  /// 侧边栏排序权重（来自 manifest.ui_tab.order，越小越靠前）
  ui_tab_order?: number | null;
}

/// 插件管理摘要（来自 list_plugins 命令）
export interface PluginSummary {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  enabled: boolean;
  load_error: string | null;
  capabilities: string[];
}

/// UI schema 支持的 section 类型
export type UiSection =
  | { type: "table"; data_key: string; columns: { key: string; label: string; width?: number }[] }
  | { type: "cards"; data_key: string; card: { title_key: string; body_key: string } }
  | { type: "stat_grid"; data_key: string; metrics: { key: string; label: string; unit?: string }[] }
  | { type: "markdown"; data_key: string }
  | { type: "chart_bar"; data_key: string; x_key: string; y_key: string };

export interface PluginUiSchema {
  title: string;
  sections: UiSection[];
}

export interface OverviewInfo {
  app_name: string;
  package_name: string;
  version_name: string;
  version_code: string;
  min_sdk: string;
  target_sdk: string;
  compile_sdk: string;
  apk_size: number;
  estimated_install_size: number;
  abis: string[];
  languages: string[];
  densities: string[];
  signature_version: string;
  debuggable: boolean;
  allow_backup: boolean;
  extract_native_libs: boolean;
  uses_cleartext_traffic: boolean;
  instant_app: boolean;
  split_apk: boolean;
  bundle_info: string | null;
  app_icon_base64: string | null;
}

export interface ManifestInfo {
  package: string;
  version_code: string;
  version_name: string;
  min_sdk: number;
  target_sdk: number;
  compile_sdk: number;
  debuggable: boolean;
  allow_backup: boolean;
  extract_native_libs: boolean;
  uses_cleartext_traffic: boolean;
  instant_app: boolean;
  activities: Component[];
  services: Component[];
  receivers: Component[];
  providers: Component[];
  permissions_declared: string[];
  uses_features: Feature[];
  queries: Query[];
  meta_data: MetaData[];
  launch_activity: string | null;
  raw_xml: string;
}

export interface Component {
  name: string;
  exported: boolean;
  enabled: boolean;
  permission: string | null;
  process: string | null;
  intent_filters: IntentFilter[];
  meta_data: MetaData[];
}

export interface IntentFilter {
  actions: string[];
  categories: string[];
  data_schemes: string[];
  data_hosts: string[];
  data_paths: string[];
  data_mime_types: string[];
}

export interface Feature {
  name: string;
  required: boolean;
  version: number;
}

export interface Query {
  package: string | null;
  intent: IntentFilter | null;
}

export interface MetaData {
  name: string;
  value: string | null;
  resource: string | null;
}

export interface PermissionAnalysis {
  permissions: PermissionInfo[];
  summary: PermissionSummary;
}

export interface PermissionInfo {
  name: string;
  protection_level: string;
  description: string;
  risk_level: string;
  recommended_usage: string;
  category: string;
}

export interface PermissionSummary {
  total: number;
  normal: number;
  dangerous: number;
  signature: number;
  special: number;
  unknown: number;
}

export interface ComponentAnalysis {
  stats: ComponentStats;
  activities: Component[];
  services: Component[];
  receivers: Component[];
  providers: Component[];
  exported_components: ExportedComponent[];
}

export interface ComponentStats {
  activities: number;
  services: number;
  receivers: number;
  providers: number;
  exported: number;
  with_intent_filters: number;
}

export interface ExportedComponent {
  name: string;
  component_type: string;
  permission: string | null;
  has_intent_filter: boolean;
}

export interface ResourceAnalysis {
  summary: ResourceSummary;
  by_type: ResourceTypeGroup[];
  largest_resources: ResourceEntry[];
  duplicate_resources: DuplicateResource[];
  image_stats: ImageStats;
}

export interface ResourceSummary {
  total: number;
  total_size: number;
  types: number;
}

export interface ResourceTypeGroup {
  type_name: string;
  count: number;
  total_size: number;
  entries: ResourceEntry[];
}

export interface ResourceEntry {
  name: string;
  path: string;
  size: number;
  resource_type: string;
  compression: string | null;
}

export interface DuplicateResource {
  name: string;
  paths: string[];
  total_size: number;
}

export interface ImageStats {
  total_images: number;
  total_size: number;
  by_format: FormatStat[];
  largest_images: ResourceEntry[];
}

export interface FormatStat {
  format: string;
  count: number;
  total_size: number;
}

export interface NativeLibAnalysis {
  libraries: NativeLib[];
  by_abi: AbiGroup[];
  summary: NativeLibSummary;
}

export interface NativeLib {
  file_name: string;
  path: string;
  abi: string;
  architecture: string;
  size: number;
  compressed_size: number;
  compression: string;
  export_symbols: string[];
}

export interface AbiGroup {
  abi: string;
  count: number;
  total_size: number;
  libraries: NativeLib[];
}

export interface NativeLibSummary {
  total: number;
  total_size: number;
  abis: string[];
}

export interface DexAnalysis {
  dex_files: DexFile[];
  summary: DexSummary;
  packages: PackageInfo[];
  largest_packages: PackageInfo[];
  largest_classes: ClassInfo[];
}

export interface DexFile {
  name: string;
  size: number;
  class_count: number;
  method_count: number;
  field_count: number;
}

export interface DexSummary {
  total_dex_files: number;
  total_classes: number;
  total_methods: number;
  total_fields: number;
  total_size: number;
}

export interface PackageInfo {
  name: string;
  class_count: number;
  method_count: number;
  field_count: number;
}

export interface ClassInfo {
  name: string;
  method_count: number;
  field_count: number;
  dex_file: string;
}

export interface CertificateAnalysis {
  signers: SignerInfo[];
  signature_scheme: string;
  is_debug_certificate: boolean;
  is_expired: boolean;
  has_v1: boolean;
  has_v2: boolean;
  has_v3: boolean;
}

export interface SignerInfo {
  subject: string;
  issuer: string;
  serial_number: string;
  sha1: string;
  sha256: string;
  md5: string;
  not_before: string;
  not_after: string;
  public_key_algorithm: string;
  signature_algorithm: string;
  is_expired: boolean;
  validity_days: number;
}

export interface SecurityAnalysis {
  score: number;
  issues: SecurityIssue[];
  recommendations: string[];
}

export interface SecurityIssue {
  severity: string;
  category: string;
  title: string;
  description: string;
  recommendation: string;
}

export interface AISummary {
  overview: string;
  app_type: string;
  tech_stack: TechStackEntry[];
  architecture_guess: string;
  potential_risks: string[];
  performance_suggestions: string[];
  packaging_suggestions: string[];
  permission_review: string;
}

export interface TechStackEntry {
  name: string;
  confidence: string;
  evidence: string[];
}

export interface ProgressUpdate {
  stage: string;
  message: string;
  percent: number;
}

export interface SearchResult {
  category: string;
  title: string;
  detail: string;
}

export interface RecentFile {
  path: string;
  name: string;
  size: number;
  last_opened: string;
}

export type NavSection =
  | "overview"
  | "manifest"
  | "permissions"
  | "components"
  | "resources"
  | "native_libs"
  | "dex"
  | "certificate"
  | "security"
  | "ai_summary"
  | "plugins"
  | string; // 允许插件动态 tab id
