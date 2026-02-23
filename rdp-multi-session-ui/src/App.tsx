import { useState } from "react";
import { motion } from "framer-motion";
import {
  ShieldCheck,
  ShieldAlert,
  Activity,
  Settings,
  Terminal,
  Users,
  Power
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
    <span className="text-sm font-medium text-gray-700">{count} Active Sessions</span>
  </div>
);

function App() {
  const [isActive, setIsActive] = useState(false);
  const [isPatching, setIsPatching] = useState(false);
  const [osBuild] = useState("Windows 11 (Build 26100)");

  const togglePatch = async () => {
    setIsPatching(true);
    // Simulate system work
    await new Promise((r) => setTimeout(r, 1500));
    setIsActive(!isActive);
    setIsPatching(false);
  };

  return (
    <div className="flex flex-col h-screen max-w-sm mx-auto bg-white overflow-hidden border-x border-gray-100 shadow-2xl">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-4 border-b border-gray-50 bg-white/80 backdrop-blur-md sticky top-0 z-50">
        <div className="flex items-center gap-2">
          <div className="w-2 h-2 rounded-full bg-black animate-pulse" />
          <h1 className="text-xs font-bold tracking-widest uppercase text-black">RDP Enabler</h1>
        </div>
        <div className="flex gap-3">
          <button className="p-1 hover:bg-gray-100 rounded-md transition-colors">
            <Settings className="w-4 h-4 text-gray-400" />
          </button>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 flex flex-col items-center justify-center px-8 gap-8">
        <StatusRing isActive={isActive} />

        <div className="text-center space-y-2">
          <h2 className="text-2xl font-semibold tracking-tight">
            {isActive ? "Multi-Session Active" : "Restrictions Enabled"}
          </h2>
          <p className="text-sm text-gray-500 font-medium">{osBuild}</p>
        </div>

        <SessionCard count={isActive ? 2 : 1} />

        {/* Vercel Style Toggle Button */}
        <button
          onClick={togglePatch}
          disabled={isPatching}
          className={`group relative flex items-center justify-center w-full py-4 rounded-xl font-medium transition-all duration-300 overflow-hidden ${isPatching ? "bg-gray-100 cursor-not-allowed" :
            isActive ? "bg-black text-white hover:bg-gray-900" : "bg-vercel-blue text-white hover:opacity-90"
            }`}
        >
          {isPatching && (
            <motion.div
              initial={{ x: "-100%" }}
              animate={{ x: "100%" }}
              transition={{ repeat: Infinity, duration: 1, ease: "linear" }}
              className="absolute inset-0 bg-white/20"
            />
          )}
          <span className="relative z-10 flex items-center gap-2">
            {isPatching ? "Processing..." : (
              <>
                <Power className="w-4 h-4" />
                {isActive ? "Disable Multi-Session" : "Enable Multi-Session"}
              </>
            )}
          </span>
        </button>
      </main>

      {/* Footer Details */}
      <footer className="px-8 py-6 bg-gray-50 border-t border-gray-100">
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-1">
            <span className="text-[10px] uppercase tracking-wider text-gray-400 font-bold">Persistence</span>
            <div className="flex items-center gap-1.5">
              <div className={`w-1.5 h-1.5 rounded-full ${isActive ? "bg-green-500" : "bg-gray-300"}`} />
              <span className="text-xs text-gray-600 font-medium">Automatic</span>
            </div>
          </div>
          <div className="space-y-1">
            <span className="text-[10px] uppercase tracking-wider text-gray-400 font-bold">Protection</span>
            <div className="flex items-center gap-1.5">
              <Activity className="w-3 h-3 text-gray-400" />
              <span className="text-xs text-gray-600 font-medium">Disabled</span>
            </div>
          </div>
        </div>

        <div className="mt-6 pt-4 border-t border-gray-200/50 flex items-center justify-between">
          <button className="flex items-center gap-1.5 text-[11px] text-gray-400 hover:text-black transition-colors font-semibold">
            <Terminal className="w-3 h-3" />
            VIEW RAW LOGS
          </button>
          <span className="text-[10px] text-gray-300">v1.2.0-stable</span>
        </div>
      </footer>
    </div>
  );
}

export default App;
