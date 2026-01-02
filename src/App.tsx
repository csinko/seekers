import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";

interface Credentials {
  orgId: string;
  sessionKey: string;
}

interface UsageData {
  fiveHour: {
    utilization: number;
    resetsAt: string;
  } | null;
  sevenDay: {
    utilization: number;
    resetsAt: string;
  } | null;
}

interface AppSettings {
  menuBarDisplay: "session" | "weekly" | "both" | "higher";
  showPercentSymbol: boolean;
  progressStyle: "circles" | "blocks" | "bar" | "dots";
  progressLength: 5 | 8 | 10;
  refreshInterval: 0 | 5 | 15 | 30 | 60;
  notifySession: number;
  notifyWeekly: number;
}

type Tab = "account" | "appearance" | "about";

const defaultSettings: AppSettings = {
  menuBarDisplay: "session",
  showPercentSymbol: true,
  progressStyle: "circles",
  progressLength: 10,
  refreshInterval: 15,
  notifySession: 80,
  notifyWeekly: 80,
};

function App() {
  const [tab, setTab] = useState<Tab>("account");
  const [credentials, setCredentials] = useState<Credentials>({
    orgId: "",
    sessionKey: "",
  });
  const [loading, setLoading] = useState(true);
  const [usage, setUsage] = useState<UsageData | null>(null);
  const [status, setStatus] = useState<"idle" | "saving" | "saved">("idle");
  const [settings, setSettings] = useState<AppSettings>(defaultSettings);
  const [version, setVersion] = useState<string>("");

  useEffect(() => {
    loadCredentials();
    loadSettings();
    getVersion().then(setVersion);
    const unlisten = listen<UsageData>("usage-updated", (event) => {
      setUsage(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  async function loadSettings() {
    try {
      const s = await invoke<AppSettings>("get_settings");
      setSettings(s);
    } catch (e) {
      console.error("Failed to load settings:", e);
    }
  }

  async function updateSettings(newSettings: AppSettings) {
    setSettings(newSettings);
    try {
      await invoke("save_settings", { newSettings });
    } catch (e) {
      console.error("Failed to save settings:", e);
    }
  }

  const [credentialsDirty, setCredentialsDirty] = useState(false);

  async function saveCredentials() {
    if (!credentials.orgId || !credentials.sessionKey) return;
    if (!credentialsDirty) return;
    
    setCredentialsDirty(false);
    setStatus("saving");
    try {
      await invoke("save_credentials", {
        orgId: credentials.orgId,
        sessionKey: credentials.sessionKey,
      });
      await invoke("refresh_usage");
      setStatus("saved");
      setTimeout(() => setStatus("idle"), 1500);
    } catch (e) {
      console.error("Failed to save:", e);
      setStatus("idle");
    }
  }

  useEffect(() => {
    if (!credentialsDirty) return;
    const timeout = setTimeout(saveCredentials, 2000);
    return () => clearTimeout(timeout);
  }, [credentials.orgId, credentials.sessionKey, credentialsDirty]);

  async function loadCredentials() {
    try {
      const creds = await invoke<Credentials>("get_credentials");
      setCredentials(creds);
    } catch (e) {
      console.error("Failed to load credentials:", e);
    } finally {
      setLoading(false);
    }
  }

  function formatResetTime(isoString: string): string {
    const date = new Date(isoString);
    const now = new Date();
    const diffMs = date.getTime() - now.getTime();
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffMins = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));

    if (diffHours > 24) {
      return date.toLocaleDateString("en-US", {
        weekday: "short",
        month: "short",
        day: "numeric",
      });
    } else if (diffHours > 0) {
      return `${diffHours}h ${diffMins}m`;
    } else if (diffMins > 0) {
      return `${diffMins}m`;
    }
    return "soon";
  }

  function getBarColor(pct: number): string {
    if (pct > 80) return "bg-rose-500";
    if (pct > 50) return "bg-amber-400";
    return "bg-emerald-400";
  }

  function getProgressPreview(style: AppSettings["progressStyle"], length: number, pct: number = 30): string {
    const filled = Math.round((pct / 100) * length);
    const empty = length - filled;
    const chars = {
      circles: { filled: "●", empty: "○" },
      blocks: { filled: "▰", empty: "▱" },
      bar: { filled: "█", empty: "░" },
      dots: { filled: "⬤", empty: "○" },
    };
    return chars[style].filled.repeat(filled) + chars[style].empty.repeat(empty);
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="w-5 h-5 border-2 border-gray-200 border-t-gray-500 rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <div className="select-none">
      <div className="h-8 drag-region" />
      
      <div className="px-4 pb-5">
        {/* Tab Bar */}
        <div className="flex justify-center mb-6">
          <div className="flex gap-1 p-1 bg-black/5 dark:bg-white/10 rounded-lg">
            {(["account", "appearance", "about"] as Tab[]).map((t) => (
              <button
                key={t}
                onClick={() => setTab(t)}
                className={`px-4 py-1.5 text-[13px] font-medium rounded-md transition-all duration-150
                  ${tab === t 
                    ? "bg-white dark:bg-white/20 text-gray-900 dark:text-white shadow-sm" 
                    : "text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
                  }`}
              >
                {t.charAt(0).toUpperCase() + t.slice(1)}
              </button>
            ))}
          </div>
        </div>

        {tab === "account" && (
          <div className="space-y-5">
            {/* Usage Section */}
            {usage && (
              <Section title="Usage">
                <div className="space-y-4">
                  {usage.fiveHour && (
                    <UsageBar
                      label="Session"
                      pct={usage.fiveHour.utilization}
                      resetTime={formatResetTime(usage.fiveHour.resetsAt)}
                      color={getBarColor(usage.fiveHour.utilization)}
                    />
                  )}
                  {usage.sevenDay && (
                    <UsageBar
                      label="Weekly"
                      pct={usage.sevenDay.utilization}
                      resetTime={formatResetTime(usage.sevenDay.resetsAt)}
                      color={getBarColor(usage.sevenDay.utilization)}
                    />
                  )}
                </div>
              </Section>
            )}

            {/* Credentials Section */}
            <Section 
              title="Credentials" 
              badge={status === "saving" ? "Saving..." : status === "saved" ? "Saved" : undefined}
              badgeColor={status === "saved" ? "text-emerald-500" : "text-gray-400"}
            >
              <div className="space-y-3">
                <Input
                  label="Organization ID"
                  value={credentials.orgId}
                  onChange={(v) => { setCredentials({ ...credentials, orgId: v }); setCredentialsDirty(true); }}
                  onBlur={saveCredentials}
                  placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
                  mono
                />
                <Input
                  label="Session Key"
                  value={credentials.sessionKey}
                  onChange={(v) => { setCredentials({ ...credentials, sessionKey: v }); setCredentialsDirty(true); }}
                  onBlur={saveCredentials}
                  placeholder="sk-ant-sid01-..."
                  type="password"
                  mono
                />
              </div>
            </Section>

            {/* Setup Guide */}
            <Section title="Setup">
              <ol className="text-[12px] text-gray-500 dark:text-gray-400 space-y-1 list-decimal list-inside leading-relaxed">
                <li>Log into <span className="text-gray-600 dark:text-gray-300">claude.ai</span></li>
                <li>Open DevTools <span className="text-[11px] text-gray-400">(Cmd+Option+I)</span></li>
                <li>Go to Application → Cookies</li>
                <li>Copy <code className="px-1 py-0.5 bg-black/5 dark:bg-white/10 rounded text-[11px]">sessionKey</code></li>
                <li>Get Org ID from URL <span className="text-[11px] text-gray-400">(/organizations/...)</span></li>
              </ol>
            </Section>
          </div>
        )}

        {tab === "appearance" && (
          <div className="space-y-5">
            <Section title="Menu Bar">
              <div className="space-y-3">
                <Row label="Display">
                  <Select
                    value={settings.menuBarDisplay}
                    onChange={(v) => updateSettings({ ...settings, menuBarDisplay: v as AppSettings["menuBarDisplay"] })}
                    options={[
                      { value: "session", label: "Session only" },
                      { value: "weekly", label: "Weekly only" },
                      { value: "both", label: "Both (7/18)" },
                      { value: "higher", label: "Higher value" },
                    ]}
                  />
                </Row>
                <Row label="Show % symbol">
                  <Toggle
                    checked={settings.showPercentSymbol}
                    onChange={(v) => updateSettings({ ...settings, showPercentSymbol: v })}
                  />
                </Row>
              </div>
            </Section>

            <Section title="Progress Indicator">
              <div className="space-y-3">
                <Row label="Style">
                  <Select
                    value={settings.progressStyle}
                    onChange={(v) => updateSettings({ ...settings, progressStyle: v as AppSettings["progressStyle"] })}
                    options={[
                      { value: "circles", label: "Circles" },
                      { value: "blocks", label: "Blocks" },
                      { value: "bar", label: "Bar" },
                      { value: "dots", label: "Dots" },
                    ]}
                  />
                </Row>
                <Row label="Length">
                  <Select
                    value={String(settings.progressLength)}
                    onChange={(v) => updateSettings({ ...settings, progressLength: Number(v) as AppSettings["progressLength"] })}
                    options={[
                      { value: "5", label: "Short" },
                      { value: "8", label: "Medium" },
                      { value: "10", label: "Long" },
                    ]}
                  />
                </Row>
                <div className="flex items-center justify-center py-2 bg-black/[0.03] dark:bg-white/[0.06] rounded-lg">
                  <span className="font-mono text-[14px] tracking-wide text-gray-600 dark:text-gray-300">
                    {getProgressPreview(settings.progressStyle, settings.progressLength, 30)}
                  </span>
                </div>
              </div>
            </Section>

            <Section title="Refresh">
              <Row label="Auto-refresh">
                <Select
                  value={String(settings.refreshInterval)}
                  onChange={(v) => updateSettings({ ...settings, refreshInterval: Number(v) as AppSettings["refreshInterval"] })}
                  options={[
                    { value: "0", label: "Manual" },
                    { value: "5", label: "5 min" },
                    { value: "15", label: "15 min" },
                    { value: "30", label: "30 min" },
                    { value: "60", label: "1 hour" },
                  ]}
                />
              </Row>
            </Section>

            <Section title="Notifications">
              <div className="space-y-3">
                <Row label="Session warning">
                  <Select
                    value={String(settings.notifySession)}
                    onChange={(v) => updateSettings({ ...settings, notifySession: Number(v) })}
                    options={[
                      { value: "0", label: "Off" },
                      { value: "50", label: "50%" },
                      { value: "70", label: "70%" },
                      { value: "80", label: "80%" },
                      { value: "90", label: "90%" },
                    ]}
                  />
                </Row>
                <Row label="Weekly warning">
                  <Select
                    value={String(settings.notifyWeekly)}
                    onChange={(v) => updateSettings({ ...settings, notifyWeekly: Number(v) })}
                    options={[
                      { value: "0", label: "Off" },
                      { value: "50", label: "50%" },
                      { value: "70", label: "70%" },
                      { value: "80", label: "80%" },
                      { value: "90", label: "90%" },
                    ]}
                  />
                </Row>
                <button
                  onClick={() => invoke("test_notification")}
                  className="w-full py-2 text-[13px] text-gray-500 dark:text-gray-400 
                    bg-black/[0.03] dark:bg-white/[0.06] hover:bg-black/[0.06] dark:hover:bg-white/[0.1] 
                    rounded-lg transition-colors"
                >
                  Test Notification
                </button>
              </div>
            </Section>
          </div>
        )}

        {tab === "about" && (
          <div className="flex flex-col items-center justify-center py-10">
            <div className="w-16 h-16 mb-4 rounded-2xl bg-gradient-to-br from-violet-500 to-purple-600 
              flex items-center justify-center shadow-lg">
              <span className="text-3xl">📊</span>
            </div>
            <h1 className="text-[17px] font-semibold text-gray-900 dark:text-white">Seekers</h1>
            <p className="text-[12px] text-gray-400 mt-1">Version {version}</p>
            <p className="text-[13px] text-gray-500 dark:text-gray-400 mt-6 text-center leading-relaxed">
              Track your Claude usage<br />directly from the menu bar
            </p>
            <a 
              href="https://claude.ai" 
              className="mt-6 px-4 py-1.5 text-[13px] text-white bg-gray-900 dark:bg-white dark:text-gray-900 
                rounded-full hover:opacity-80 transition-opacity"
              target="_blank"
              rel="noopener noreferrer"
            >
              Open Claude
            </a>
          </div>
        )}
      </div>
    </div>
  );
}

// Components

function Section({ title, badge, badgeColor, children }: { 
  title: string; 
  badge?: string;
  badgeColor?: string;
  children: React.ReactNode 
}) {
  return (
    <div>
      <div className="flex items-center gap-2 mb-2.5">
        <span className="text-[11px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider">
          {title}
        </span>
        {badge && <span className={`text-[11px] ${badgeColor}`}>{badge}</span>}
      </div>
      {children}
    </div>
  );
}

function Row({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between">
      <span className="text-[13px] text-gray-600 dark:text-gray-300">{label}</span>
      {children}
    </div>
  );
}

function Input({ label, value, onChange, onBlur, placeholder, type = "text", mono }: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  onBlur?: () => void;
  placeholder?: string;
  type?: "text" | "password";
  mono?: boolean;
}) {
  return (
    <div>
      <label className="block text-[12px] text-gray-500 dark:text-gray-400 mb-1.5">{label}</label>
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onBlur={onBlur}
        placeholder={placeholder}
        className={`w-full px-3 py-2 text-[13px] rounded-lg
          bg-black/[0.03] dark:bg-white/[0.06]
          border border-transparent
          focus:outline-none focus:border-gray-300 dark:focus:border-gray-600
          placeholder:text-gray-400 dark:placeholder:text-gray-500
          transition-colors
          ${mono ? "font-mono" : ""}`}
      />
    </div>
  );
}

function Select({ value, onChange, options }: {
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
}) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="pl-3 pr-8 py-1.5 text-[13px] rounded-lg
        bg-black/[0.03] dark:bg-white/[0.06]
        border border-transparent
        focus:outline-none focus:border-gray-300 dark:focus:border-gray-600
        transition-colors cursor-pointer"
    >
      {options.map((opt) => (
        <option key={opt.value} value={opt.value}>{opt.label}</option>
      ))}
    </select>
  );
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (value: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={`relative w-10 h-6 rounded-full transition-colors duration-200
        ${checked ? "bg-emerald-500" : "bg-gray-300 dark:bg-gray-600"}`}
    >
      <div
        className={`absolute top-1 w-4 h-4 bg-white rounded-full shadow transition-transform duration-200
          ${checked ? "translate-x-5" : "translate-x-1"}`}
      />
    </button>
  );
}

function UsageBar({ label, pct, resetTime, color }: { 
  label: string; 
  pct: number; 
  resetTime: string;
  color: string;
}) {
  return (
    <div>
      <div className="flex items-baseline justify-between mb-1.5">
        <span className="text-[13px] font-medium text-gray-700 dark:text-gray-200">{label}</span>
        <div className="flex items-baseline gap-1.5">
          <span className="text-[15px] font-semibold tabular-nums text-gray-900 dark:text-white">
            {pct.toFixed(0)}%
          </span>
          <span className="text-[11px] text-gray-400">· {resetTime}</span>
        </div>
      </div>
      <div className="h-2 bg-black/[0.06] dark:bg-white/[0.1] rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full transition-all duration-500 ${color}`}
          style={{ width: `${Math.min(pct, 100)}%` }}
        />
      </div>
    </div>
  );
}

export default App;
