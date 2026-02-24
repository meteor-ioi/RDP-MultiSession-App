import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  ShieldCheck,
  ShieldAlert,
  Activity,
  Settings as SettingsIcon,
  Terminal,
  Users,
  Power,
  X,
  ChevronRight,
  Info
} from "lucide-react";

// --- Sub-components ---

const StatusRing = ({ isActive }: { isActive: boolean }) => (
  <div className="relative flex items-center justify-center w-32 h-32">
    <motion.div
      animate={{
        scale: isActive ? [1, 1.05, 1] : 1,
        opacity: isActive ? [0.5, 0.8, 0.5] : 0.3,
      }}
      transition={{ repeat: Infinity, duration: 3, ease: "easeInOut" }}
      className={`absolute inset-0 rounded-full blur-2xl ${isActive ? "bg-vercel-blue" : "bg-gray-300"
        }`}
    />
    <div className={`relative z-10 flex items-center justify-center w-24 h-24 rounded-full border-2 transition-colors duration-500 bg-white ${isActive ? "border-vercel-blue shadow-lg shadow-vercel-blue/20" : "border-gray-200"
      }`}>
      {isActive ? (
        <ShieldCheck className="w-10 h-10 text-vercel-blue" />
      ) : (
        <ShieldAlert className="w-10 h-10 text-gray-400" />
      )}
    </div>
  </div>
);

const SessionCard = ({ count }: { count: number }) => (
  <div className="flex items-center gap-3 px-4 py-2 bg-gray-50 border border-gray-100 rounded-full">
    <Users className="w-4 h-4 text-gray-500" />
    <span className="text-sm font-medium text-gray-700">{count} 活跃会话数</span>
  </div>
);

const Drawer = ({ isOpen, onClose, title, children }: { isOpen: boolean; onClose: () => void; title: string; children: React.ReactNode }) => (
  <AnimatePresence>
    {isOpen && (
      <>
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="absolute inset-0 bg-black/20 backdrop-blur-sm z-[60]"
        />
        <motion.div
          initial={{ y: "100%" }}
          animate={{ y: 0 }}
          exit={{ y: "100%" }}
          transition={{ type: "spring", damping: 25, stiffness: 200 }}
          className="absolute inset-x-0 bottom-0 max-h-[90%] bg-white rounded-t-2xl shadow-2xl z-[70] overflow-hidden flex flex-col"
        >
          <div className="flex items-center justify-between px-6 py-4 border-b border-gray-100">
            <h3 className="text-sm font-bold uppercase tracking-wider text-black">{title}</h3>
            <button onClick={onClose} className="p-1 hover:bg-gray-100 rounded-full text-gray-400 hover:text-black">
              <X className="w-5 h-5" />
            </button>
          </div>
          <div className="flex-1 overflow-y-auto p-6">
            {children}
          </div>
        </motion.div>
      </>
    )}
  </AnimatePresence>
);

const ToggleRow = ({ label, description, isEnabled, onToggle }: { label: string; description: string; isEnabled: boolean; onToggle: () => void }) => (
  <div className="flex items-center justify-between py-3 border-b border-gray-50 last:border-0">
    <div className="space-y-0.5 pr-4">
      <div className="text-sm font-medium text-black">{label}</div>
      <div className="text-xs text-gray-500">{description}</div>
    </div>
    <button
      onClick={onToggle}
      className={`relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none ${isEnabled ? "bg-black" : "bg-gray-200"}`}
    >
      <span
        aria-hidden="true"
        className={`pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out ${isEnabled ? "translate-x-5" : "translate-x-0"}`}
      />
    </button>
  </div>
);

function App() {
  const [isActive, setIsActive] = useState(false);
  const [isPatching, setIsPatching] = useState(false);
  const [osBuild] = useState("Windows 11 (Build 26100)");

  // UI State
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [isLogsOpen, setIsLogsOpen] = useState(false);

  // Settings State
  const [persistence, setPersistence] = useState(true);
  const [defender, setDefender] = useState(true);
  const [autoUpdate, setAutoUpdate] = useState(false);

  const togglePatch = async () => {
    setIsPatching(true);
    await new Promise((r) => setTimeout(r, 1500));
    setIsActive(!isActive);
    setIsPatching(false);
  };

  const logs = [
    { time: "13:42:01", msg: "初始化 RDP 并发管理引擎...", type: "info" },
    { time: "13:42:02", msg: "获取系统版本: Windows 11 Build 26100", type: "info" },
    { time: "13:42:02", msg: "检测 termsrv.dll 状态: 正常", type: "success" },
    { time: "13:42:03", msg: "持久化守护进程已就绪。", type: "info" },
    { time: "13:45:10", msg: "等待用户指令...", type: "wait" },
  ];

  return (
    <div className="relative flex flex-col h-screen max-w-sm mx-auto bg-white overflow-hidden border-x border-gray-100 shadow-2xl">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-4 border-b border-gray-50 bg-white/80 backdrop-blur-md sticky top-0 z-50">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-black animate-pulse" />
          <h1 className="text-xs font-bold tracking-widest uppercase text-black">RDP 并发管理器</h1>
        </div>
        <div className="flex gap-3">
          <button
            onClick={() => setIsSettingsOpen(true)}
            className="p-2 hover:bg-gray-100 rounded-lg transition-colors group"
          >
            <SettingsIcon className="w-4 h-4 text-gray-400 group-hover:text-black transition-colors" />
          </button>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 flex flex-col items-center justify-center px-8 gap-8">
        <StatusRing isActive={isActive} />

        <div className="text-center space-y-2">
          <h2 className="text-2xl font-semibold tracking-tight">
            {isActive ? "多用户并发已开启" : "未开启多用户"}
          </h2>
          <p className="text-sm text-gray-500 font-medium">{osBuild}</p>
        </div>

        <SessionCard count={isActive ? 2 : 1} />

        {/* 主操作按钮 */}
        <button
          onClick={togglePatch}
          disabled={isPatching}
          className={`group relative flex items-center justify-center w-full py-4 rounded-xl font-medium transition-all duration-300 overflow-hidden ${isPatching ? "bg-gray-100 cursor-not-allowed" :
            isActive ? "bg-black text-white hover:bg-gray-900" : "bg-vercel-blue text-white hover:opacity-90 shadow-lg shadow-vercel-blue/20"
            }`}
        >
          {isPatching && (
            <motion.div
              initial={{ x: "-100%" }}
              animate={{ x: "100%" }}
              transition={{ repeat: Infinity, duration: 1.5, ease: "linear" }}
              className="absolute inset-0 bg-white/20"
            />
          )}
          <span className="relative z-10 flex items-center gap-2 uppercase tracking-wide text-xs">
            {isPatching ? "处理中..." : (
              <>
                <Power className="w-4 h-4" />
                {isActive ? "关闭多用户并发" : "开启多用户并发"}
              </>
            )}
          </span>
        </button>
      </main>

      {/* Footer Details */}
      <footer className="px-8 py-6 bg-gray-50 border-t border-gray-100">
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-1">
            <span className="text-[10px] uppercase tracking-wider text-gray-400 font-bold">持久化守护</span>
            <div className="flex items-center gap-1.5">
              <div className={`w-1.5 h-1.5 rounded-full ${persistence ? "bg-green-500" : "bg-gray-300"}`} />
              <span className="text-xs text-gray-600 font-medium">{persistence ? "自动修复" : "已断开"}</span>
            </div>
          </div>
          <div className="space-y-1">
            <span className="text-[10px] uppercase tracking-wider text-gray-400 font-bold">系统文件保护</span>
            <div className="flex items-center gap-1.5">
              <Activity className={`w-3 h-3 ${defender ? "text-vercel-blue" : "text-gray-400"}`} />
              <span className="text-xs text-gray-600 font-medium">{defender ? "按需接管" : "原生限制"}</span>
            </div>
          </div>
        </div>

        <div className="mt-6 pt-4 border-t border-gray-200/50 flex items-center justify-between">
          <button
            onClick={() => setIsLogsOpen(true)}
            className="flex items-center gap-1.5 text-[11px] text-gray-400 hover:text-black transition-colors font-semibold uppercase tracking-tighter"
          >
            <Terminal className="w-3 h-3" />
            查看运行日志
          </button>
          <span className="text-[10px] text-gray-300">v1.2.0-stable</span>
        </div>
      </footer>

      {/* Settings Drawer */}
      <Drawer isOpen={isSettingsOpen} onClose={() => setIsSettingsOpen(false)} title="高级配置">
        <div className="space-y-2">
          <ToggleRow
            label="开机持久化"
            description="在系统更新后自动尝试重新应用补丁。"
            isEnabled={persistence}
            onToggle={() => setPersistence(!persistence)}
          />
          <ToggleRow
            label="Defender 排除"
            description="自动将 termsrv.dll 加入安全白名单。"
            isEnabled={defender}
            onToggle={() => setDefender(!defender)}
          />
          <ToggleRow
            label="自动检查更新"
            description="当补丁包特征码更新时自动下载。"
            isEnabled={autoUpdate}
            onToggle={() => setAutoUpdate(!autoUpdate)}
          />

          <div className="mt-8 pt-6 border-t border-gray-100">
            <div className="flex items-center gap-2 text-vercel-blue mb-2">
              <Info className="w-4 h-4" />
              <span className="text-xs font-bold uppercase">备份路径</span>
            </div>
            <div className="flex items-center gap-2 bg-gray-50 border border-gray-100 px-3 py-2 rounded-lg group hover:border-vercel-blue/30 transition-colors">
              <span className="text-xs text-gray-500 font-mono truncate flex-1">C:\RDP_Backups\termsrv_backup.dll</span>
              <ChevronRight className="w-3 h-3 text-gray-300" />
            </div>
          </div>
        </div>
      </Drawer>

      {/* Logs Drawer */}
      <Drawer isOpen={isLogsOpen} onClose={() => setIsLogsOpen(false)} title="系统执行日志">
        <div className="bg-black rounded-xl p-4 font-mono text-[11px] leading-relaxed space-y-2 shadow-inner min-h-[300px]">
          {logs.map((log, i) => (
            <div key={i} className="flex gap-3">
              <span className="text-gray-600 shrink-0">{log.time}</span>
              <span className={
                log.type === "success" ? "text-green-400" :
                  log.type === "info" ? "text-gray-300" :
                    log.type === "wait" ? "text-yellow-400 animate-pulse" : "text-gray-300"
              }>
                {log.type === "info" && <span className="mr-1 text-vercel-blue">●</span>}
                {log.msg}
              </span>
            </div>
          ))}
          <div className="pt-2 flex items-center gap-2 text-gray-500 italic">
            <div className="w-1 h-3 bg-vercel-blue animate-pulse" />
            等待系统指令...
          </div>
        </div>
        <button
          className="w-full mt-4 py-3 bg-gray-100 hover:bg-black hover:text-white rounded-xl text-xs font-bold transition-all uppercase"
          onClick={() => alert("日志已保存至本地桌面。")}
        >
          导出原始日志
        </button>
      </Drawer>
    </div>
  );
}

export default App;
