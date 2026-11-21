import React from "react";
import {
  LayoutDashboard,
  Smartphone,
  MonitorPlay,
  SlidersHorizontal,
  Wrench,
  ScrollText,
  Settings,
  Info,
  Compass,
  type LucideIcon,
} from "lucide-react";
import { BrandMark } from "./BrandMark";
import type { NavPage } from "../types/scrdeck";

interface SidebarProps {
  activePage: NavPage;
  onSelectPage: (page: NavPage) => void;
  activeSessionCount: number;
}

const NAV_ITEMS: { id: NavPage; label: string; icon: LucideIcon }[] = [
  { id: "overview", label: "Overview", icon: LayoutDashboard },
  { id: "devices", label: "Devices", icon: Smartphone },
  { id: "mirroring", label: "Mirroring", icon: MonitorPlay },
  { id: "presets", label: "Presets", icon: SlidersHorizontal },
  { id: "adb-tools", label: "ADB Tools", icon: Wrench },
  { id: "activity", label: "Activity & Logs", icon: ScrollText },
  { id: "settings", label: "Settings", icon: Settings },
  { id: "about", label: "About ScrDeck", icon: Info },
];

export const Sidebar: React.FC<SidebarProps> = ({
  activePage,
  onSelectPage,
  activeSessionCount,
}) => {
  return (
    <aside
      aria-label="Primary navigation"
      className="flex h-full w-16 md:w-56 shrink-0 flex-col border-r border-border-default bg-surface select-none"
    >
      <div className="flex h-14 items-center gap-2.5 border-b border-border-default px-4">
        <BrandMark size={24} className="shrink-0 text-text-primary" />
        <div className="hidden md:flex flex-col leading-tight">
          <span className="text-sm font-semibold tracking-tight text-text-primary">
            ScrDeck
          </span>
          <span className="text-[11px] text-text-muted">
            Powered by scrcpy
          </span>
        </div>
      </div>

      <nav className="flex-1 space-y-1 p-2 overflow-y-auto">
        {NAV_ITEMS.map((item) => {
          const Icon = item.icon;
          const isActive = activePage === item.id;
          return (
            <button
              key={item.id}
              type="button"
              onClick={() => onSelectPage(item.id)}
              title={item.label}
              aria-current={isActive ? "page" : undefined}
              className={`flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-[13px] font-medium transition-colors ${
                isActive
                  ? "bg-elevated text-text-primary border border-border-default"
                  : "text-text-secondary hover:bg-subtle hover:text-text-primary border border-transparent"
              }`}
            >
              <span
                className={`shrink-0 ${
                  isActive ? "text-accent-lime" : "text-text-secondary"
                }`}
              >
                <Icon size={18} />
              </span>
              <span className="hidden md:inline truncate">{item.label}</span>
              {item.id === "mirroring" && activeSessionCount > 0 && (
                <span className="hidden md:inline-flex ml-auto items-center rounded-full bg-status-success/20 px-2 py-0.5 text-[11px] font-medium text-status-success tabular-nums">
                  {activeSessionCount}
                </span>
              )}
            </button>
          );
        })}
      </nav>

      <div className="border-t border-border-default p-2">
        <button
          type="button"
          onClick={() => onSelectPage("setup")}
          title="Setup Assistant"
          aria-current={activePage === "setup" ? "page" : undefined}
          className={`flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left text-[13px] font-medium transition-colors ${
            activePage === "setup"
              ? "bg-elevated text-text-primary border border-border-default"
              : "text-text-secondary hover:bg-subtle hover:text-text-primary border border-transparent"
          }`}
        >
          <Compass size={18} className="shrink-0" />
          <span className="hidden md:inline truncate">Setup Assistant</span>
        </button>
      </div>
    </aside>
  );
};
